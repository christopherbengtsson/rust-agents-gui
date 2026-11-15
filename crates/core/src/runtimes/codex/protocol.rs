use crate::runtimes::{Authentication, BillingRoute, SetupEvidence, SetupFailure, SetupProblem};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::time::{Instant, timeout_at};

pub(super) const MINIMUM_VERSION: semver::Version = semver::Version::new(0, 169, 0);
pub(super) const MAX_FRAME: usize = 256 * 1024;

pub(in crate::runtimes) enum LoginEvent {
    Pending,
    BrowserOpened(bool),
    CleanupUncertain,
}

pub(super) fn browser_url(value: &str, fixture: bool) -> bool {
    if value.len() > 8192 || value.chars().any(char::is_control) {
        return false;
    }
    url::Url::parse(value).is_ok_and(|url| {
        url.username().is_empty()
            && url.password().is_none()
            && url.host_str().is_some()
            && (url.scheme() == "https"
                || (fixture
                    && url.scheme() == "http"
                    && matches!(url.host_str(), Some("127.0.0.1" | "localhost"))))
    })
}

/// Frame progress lives outside the cancellable read future, so cancellation or
/// a reopen command cannot discard a partial JSON line.
struct Frames<R> {
    read: BufReader<R>,
    bytes: Vec<u8>,
}

fn login_completion(
    frame: &Value,
    id: &str,
) -> Result<Option<crate::runtimes::LoginOutcome>, SetupFailure> {
    if frame.get("id").is_some() || frame.get("result").is_some() || frame.get("error").is_some() {
        return Err(SetupProblem::IncompatibleProtocol.into());
    }
    let method = frame
        .get("method")
        .and_then(Value::as_str)
        .ok_or(SetupProblem::IncompatibleProtocol)?;
    if method != "account/login/completed" {
        return Ok(None);
    }
    let params = frame
        .get("params")
        .ok_or(SetupProblem::IncompatibleProtocol)?;
    if params.get("loginId").and_then(Value::as_str) != Some(id) {
        return Ok(None);
    }
    match params.get("success").and_then(Value::as_bool) {
        Some(true) => Ok(Some(crate::runtimes::LoginOutcome::Completed)),
        Some(false) => Ok(Some(crate::runtimes::LoginOutcome::Failed)),
        None => Err(SetupProblem::IncompatibleProtocol.into()),
    }
}
impl<R: AsyncRead + Unpin> Frames<R> {
    fn new(read: R) -> Self {
        Self {
            read: BufReader::new(read),
            bytes: Vec::new(),
        }
    }
    async fn next(&mut self) -> Result<Value, SetupFailure> {
        loop {
            let available = self
                .read
                .fill_buf()
                .await
                .map_err(|error| SetupFailure::io("provider stdout read", error))?;
            if available.is_empty() {
                return Err(SetupFailure::at(
                    SetupProblem::StatusReadFailed,
                    "EOF before complete provider frame",
                ));
            }
            let count = available
                .iter()
                .position(|byte| *byte == b'\n')
                .map(|n| n + 1)
                .unwrap_or(available.len());
            if self.bytes.len() + count > MAX_FRAME {
                return Err(SetupFailure::at(
                    SetupProblem::IncompatibleProtocol,
                    "Provider frame exceeded 262144 bytes",
                ));
            }
            self.bytes.extend_from_slice(&available[..count]);
            self.read.consume(count);
            if self.bytes.last() == Some(&b'\n') {
                let value = serde_json::from_slice(&self.bytes).map_err(|error| {
                    SetupFailure::at(
                        SetupProblem::IncompatibleProtocol,
                        format!(
                            "Provider JSON {:?} at line {}, column {}",
                            error.classify(),
                            error.line(),
                            error.column()
                        ),
                    )
                })?;
                self.bytes.clear();
                return Ok(value);
            }
        }
    }
}

pub(super) async fn login<
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
    F: std::future::Future<Output = bool>,
>(
    read: R,
    mut write: W,
    cancel: &tokio_util::sync::CancellationToken,
    mut reopen: tokio::sync::mpsc::Receiver<()>,
    event: &impl Fn(LoginEvent),
    mut open: impl FnMut(String) -> F,
) -> Result<crate::runtimes::LoginOutcome, SetupFailure> {
    use crate::runtimes::LoginOutcome;
    let mut frames = Frames::new(read);
    let start = async {
        initialize(&mut frames, &mut write).await?;
        send(
            &mut write,
            json!({"id":2,"method":"account/login/start","params":{"type":"chatgpt"}}),
        )
        .await?;
        let mut early = Vec::new();
        loop {
            let frame = frames.next().await?;
            if frame.get("id").is_none() && frame.get("method").and_then(Value::as_str).is_some() {
                if early.len() >= 64 {
                    return Err(SetupProblem::IncompatibleProtocol.into());
                }
                early.push(frame);
                continue;
            }
            let result = reply(frame, 2).map_err(SetupFailure::from)?;
            let id = optional_text(&result, "loginId")
                .map_err(SetupFailure::from)?
                .filter(|id| !id.is_empty())
                .ok_or(SetupProblem::IncompatibleProtocol)?;
            let url = result
                .get("authUrl")
                .and_then(Value::as_str)
                .filter(|url| url.len() <= 8192 && !url.chars().any(char::is_control))
                .ok_or(SetupProblem::IncompatibleProtocol)?
                .to_owned();
            if result.get("type").and_then(Value::as_str) != Some("chatgpt") {
                return Err(SetupProblem::IncompatibleProtocol.into());
            }
            return Ok::<_, SetupFailure>((id, url, early));
        }
    };
    let Some(started) = cancel
        .run_until_cancelled(timeout_at(
            Instant::now() + std::time::Duration::from_secs(15),
            start,
        ))
        .await
    else {
        return Ok(LoginOutcome::Cancelled);
    };
    let (id, url, early) = started.map_err(|_| {
        SetupFailure::at(
            SetupProblem::StatusReadFailed,
            "Browser sign-in startup exceeded 15 seconds",
        )
    })??;
    for frame in early {
        if let Some(outcome) = login_completion(&frame, &id)? {
            return Ok(outcome);
        }
    }
    if cancel.is_cancelled() {
        return Ok(LoginOutcome::Cancelled);
    }
    event(LoginEvent::Pending);
    event(LoginEvent::BrowserOpened(open(url.clone()).await));
    let deadline = Instant::now() + std::time::Duration::from_secs(10 * 60);
    let mut notifications = 0usize;
    loop {
        // Ready control messages must not starve the timer/read branch.
        if Instant::now() >= deadline {
            return Err(SetupFailure::at(
                SetupProblem::StatusReadFailed,
                "Browser sign-in exceeded its 10-minute deadline",
            ));
        }
        enum Next {
            Cancel,
            Reopen,
            Frame(Result<Value, SetupFailure>),
        }
        let next = {
            let cancelled = std::pin::pin!(cancel.cancelled());
            let progress = std::pin::pin!(async {
                let opening = std::pin::pin!(async {
                    if reopen.is_closed() {
                        std::future::pending::<()>().await;
                    }
                    let _ = reopen.recv().await;
                });
                let reading = std::pin::pin!(timeout_at(deadline, frames.next()));
                match futures_util::future::select(opening, reading).await {
                    futures_util::future::Either::Left(_) => Next::Reopen,
                    futures_util::future::Either::Right((value, _)) => Next::Frame(
                        value
                            .map_err(|_| {
                                SetupFailure::at(
                                    SetupProblem::StatusReadFailed,
                                    "Browser sign-in exceeded its 10-minute deadline",
                                )
                            })
                            .and_then(|value| value),
                    ),
                }
            });
            match futures_util::future::select(cancelled, progress).await {
                futures_util::future::Either::Left(_) => Next::Cancel,
                futures_util::future::Either::Right((value, _)) => value,
            }
        };
        match next {
            Next::Reopen => {
                event(LoginEvent::BrowserOpened(open(url.clone()).await));
            }
            Next::Cancel => {
                let mut completed = false;
                // Exact native login identity; ack is not physical cleanup proof.
                let cancellation = async {
                    send(
                        &mut write,
                        json!({"id":3,"method":"account/login/cancel","params":{"loginId":id}}),
                    )
                    .await?;
                    for _ in 0..64 {
                        let value = frames.next().await?;
                        if value.get("id").is_some() {
                            let result = reply(value, 3).map_err(SetupFailure::from)?;
                            return match result.get("status").and_then(Value::as_str) {
                                Some("canceled" | "notFound") => Ok(()),
                                _ => Err(SetupProblem::IncompatibleProtocol.into()),
                            };
                        }
                        if login_completion(&value, &id)? == Some(LoginOutcome::Completed) {
                            completed = true;
                        }
                    }
                    Err::<(), SetupFailure>(SetupProblem::IncompatibleProtocol.into())
                };
                // Closing the connection and group cleanup follow even when the
                // cancel reply is lost. Never replay the protocol operation.
                let _ = timeout_at(
                    Instant::now() + std::time::Duration::from_secs(2),
                    cancellation,
                )
                .await;
                return Ok(if completed {
                    LoginOutcome::Completed
                } else {
                    LoginOutcome::Cancelled
                });
            }
            Next::Frame(value) => {
                let frame = value?;
                notifications += 1;
                if notifications > 1024 {
                    return Err(SetupProblem::IncompatibleProtocol.into());
                }
                if let Some(outcome) = login_completion(&frame, &id)? {
                    return Ok(outcome);
                }
            }
        }
    }
}

pub(super) fn version(bytes: &[u8], successful_exit: bool) -> Result<String, SetupProblem> {
    if !successful_exit {
        return Err(SetupProblem::StatusReadFailed);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| SetupProblem::IncompatibleProtocol)?;
    let text = text
        .strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .ok_or(SetupProblem::IncompatibleProtocol)?;
    let detected = text
        .strip_prefix("codex-cli ")
        .filter(|value| value.len() <= 128)
        .and_then(|value| semver::Version::parse(value).ok())
        .ok_or(SetupProblem::IncompatibleProtocol)?;
    if detected < MINIMUM_VERSION || !detected.pre.is_empty() {
        return Err(SetupProblem::IncompatibleProtocol);
    }
    Ok(detected.to_string())
}

fn reply(value: Value, id: u32) -> Result<Value, SetupProblem> {
    if value.get("id").and_then(Value::as_u64) != Some(u64::from(id))
        || value.get("method").is_some()
    {
        return Err(SetupProblem::IncompatibleProtocol);
    }
    if let Some(error) = value.get("error") {
        if value.get("result").is_some() {
            return Err(SetupProblem::IncompatibleProtocol);
        }
        return Err(match error.get("code").and_then(Value::as_i64) {
            Some(-32600 | -32601) => SetupProblem::IncompatibleProtocol,
            Some(_) => SetupProblem::StatusReadFailed,
            None => SetupProblem::IncompatibleProtocol,
        });
    }
    value
        .get("result")
        .cloned()
        .ok_or(SetupProblem::IncompatibleProtocol)
}

fn optional_text(value: &Value, key: &str) -> Result<Option<String>, SetupProblem> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) if text.len() <= 512 && !text.chars().any(char::is_control) => {
            Ok(Some(text.clone()))
        }
        _ => Err(SetupProblem::StatusReadFailed),
    }
}

fn setup(
    account: &Value,
    config: &Value,
    observed_version: &str,
) -> Result<SetupEvidence, SetupProblem> {
    let requires = account
        .get("requiresOpenaiAuth")
        .and_then(Value::as_bool)
        .ok_or(SetupProblem::StatusReadFailed)?;
    let config = config
        .get("config")
        .filter(|v| v.is_object())
        .ok_or(SetupProblem::StatusReadFailed)?;
    let model_provider = optional_text(config, "model_provider")?;
    let model = optional_text(config, "model")?;
    let current = account
        .get("account")
        .ok_or(SetupProblem::StatusReadFailed)?;
    let (authentication, mut billing, email, plan) = if current.is_null() {
        (
            if requires {
                Authentication::Unavailable
            } else {
                Authentication::NotRequired
            },
            BillingRoute::Unknown,
            None,
            None,
        )
    } else {
        match current.get("type").and_then(Value::as_str) {
            Some("chatgpt") => (
                Authentication::Chatgpt,
                BillingRoute::Chatgpt,
                optional_text(current, "email")?,
                optional_text(current, "planType")?,
            ),
            Some("apiKey") => (Authentication::ApiKey, BillingRoute::ApiKey, None, None),
            Some("amazonBedrock" | "gcp") => {
                (Authentication::Unknown, BillingRoute::Unknown, None, None)
            }
            _ => return Err(SetupProblem::IncompatibleProtocol),
        }
    };
    // Account metadata does not establish where a custom provider bills requests.
    if model_provider
        .as_deref()
        .is_some_and(|provider| provider != "openai")
    {
        billing = BillingRoute::Unknown;
    }
    Ok(SetupEvidence {
        version: observed_version.into(),
        authentication,
        billing,
        account: email,
        plan,
        model_provider,
        model,
    })
}

async fn send<S: AsyncWrite + Unpin>(stream: &mut S, value: Value) -> Result<(), SetupFailure> {
    let mut bytes = serde_json::to_vec(&value).expect("JSON Value serialization is infallible");
    bytes.push(b'\n');
    stream
        .write_all(&bytes)
        .await
        .map_err(|error| SetupFailure::io("provider stdin write", error))?;
    stream
        .flush()
        .await
        .map_err(|error| SetupFailure::io("provider stdin flush", error))
}
async fn receive<S: AsyncRead + Unpin>(
    stream: &mut Frames<S>,
    id: u32,
) -> Result<Value, SetupFailure> {
    let mut notifications = 0;
    loop {
        let value = stream.next().await?;
        if value.get("id").is_none()
            && value.get("method").and_then(Value::as_str).is_some()
            && value.get("result").is_none()
            && value.get("error").is_none()
        {
            notifications += 1;
            if notifications > 64 {
                return Err(SetupFailure::at(
                    SetupProblem::IncompatibleProtocol,
                    "Notification budget exceeded while awaiting setup response",
                ));
            }
            continue;
        }
        let rpc_code = value
            .get("error")
            .and_then(|error| error.get("code"))
            .and_then(Value::as_i64);
        return reply(value,id).map_err(|problem|SetupFailure::at(problem,format!("Setup response {id} rejected; RPC code {rpc_code:?}; expected exact response ID and no server request")));
    }
}
// Shared by readiness and browser login on each connection. Keep one Frames
// owner so coalesced replies and notifications survive the handshake.
async fn initialize<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    read: &mut Frames<R>,
    write: &mut W,
) -> Result<(), SetupFailure> {
    send(write, json!({"id":1,"method":"initialize","params":{"clientInfo":{"name":"agent_workspace","version":env!("CARGO_PKG_VERSION")},"capabilities":{"explicitGatewayOauth":true}}})).await?;
    if !receive(read, 1).await?.is_object() {
        return Err(SetupFailure::at(
            SetupProblem::IncompatibleProtocol,
            "initialize result must be an object",
        ));
    }
    send(write, json!({"method":"initialized","params":{}})).await?;
    // ID 4 is reserved for this probe; readiness and login use IDs 2/3.
    // Older servers may ignore the capability, so initialize alone is insufficient.
    send(write, json!({"id":4,"method":"account/gatewayOAuth/read"})).await?;
    let probe = receive(read, 4).await?;
    if ["providerId", "providerName"]
        .iter()
        .any(|key| probe.get(key).and_then(Value::as_str).is_none())
        || probe.get("required").and_then(Value::as_bool).is_none()
    {
        return Err(SetupFailure::at(
            SetupProblem::IncompatibleProtocol,
            "Required gateway capability response fields are invalid",
        ));
    }
    // Successful read proves support even when gateway credentials are not ready.
    // Authentication still comes from authoritative account/configuration reads.
    Ok(())
}

pub(super) async fn inspect<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    read: R,
    mut write: W,
    deadline: Instant,
    observed_version: &str,
) -> Result<SetupEvidence, SetupFailure> {
    timeout_at(deadline, async {
        let mut read = Frames::new(read);
        initialize(&mut read, &mut write).await?;
        send(
            &mut write,
            json!({"id":2,"method":"account/read","params":{"refreshToken":false}}),
        )
        .await?;
        let account = receive(&mut read, 2).await?;
        send(
            &mut write,
            json!({"id":3,"method":"config/read","params":{"includeLayers":false}}),
        )
        .await?;
        let config = receive(&mut read, 3).await?;
        setup(&account, &config, observed_version).map_err(|problem| {
            SetupFailure::at(
                problem,
                "Invalid or unsupported account/configuration fields",
            )
        })
    })
    .await
    .map_err(|_| {
        SetupFailure::at(
            SetupProblem::StatusReadFailed,
            "Readiness protocol exceeded its absolute 15-second deadline",
        )
    })?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtimes::{Authentication, BillingRoute};
    use serde_json::json;

    #[test]
    fn browser_login_matches_exact_completion_and_never_exports_provider_errors() {
        run(async {
            let frames = b"{\"id\":1,\"result\":{}}\n{\"id\":4,\"result\":{\"providerId\":\"openai\",\"providerName\":\"OpenAI\",\"required\":false}}\n{\"id\":2,\"result\":{\"type\":\"chatgpt\",\"loginId\":\"native-1\",\"authUrl\":\"https://auth.openai.com/authorize?state=synthetic\"}}\n{\"method\":\"account/login/completed\",\"params\":{\"loginId\":\"other\",\"success\":false}}\n{\"method\":\"account/login/completed\",\"params\":{\"loginId\":\"native-1\",\"success\":true}}\n";
            let mut commands = Vec::new();
            let (_send, receive) = tokio::sync::mpsc::channel(1);
            let opens = std::cell::Cell::new(0);
            let result = login(
                frames.as_slice(),
                &mut commands,
                &tokio_util::sync::CancellationToken::new(),
                receive,
                &|_| {},
                |_| {
                    opens.set(opens.get() + 1);
                    std::future::ready(true)
                },
            )
            .await
            .unwrap();
            assert_eq!(result, crate::runtimes::LoginOutcome::Completed);
            assert_eq!(opens.get(), 1);
            let sent: Vec<Value> = String::from_utf8(commands)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            assert_eq!(
                sent.iter()
                    .filter_map(|value| value["method"].as_str())
                    .collect::<Vec<_>>(),
                [
                    "initialize",
                    "initialized",
                    "account/gatewayOAuth/read",
                    "account/login/start"
                ]
            );
            assert_eq!(sent[3]["params"], json!({"type":"chatgpt"}));
        });
    }

    #[test]
    fn browser_open_failure_reopens_the_same_url_without_another_login() {
        run(async {
            let frames = b"{\"id\":1,\"result\":{}}\n{\"id\":4,\"result\":{\"providerId\":\"openai\",\"providerName\":\"OpenAI\",\"required\":false}}\n{\"id\":2,\"result\":{\"type\":\"chatgpt\",\"loginId\":\"native-1\",\"authUrl\":\"https://auth.openai.com/authorize?state=synthetic\"}}\n{\"method\":\"account/login/completed\",\"params\":{\"loginId\":\"native-1\",\"success\":false,\"error\":\"secret-do-not-export\"}}\n";
            let mut commands = Vec::new();
            let (send_reopen, receive) = tokio::sync::mpsc::channel(1);
            let opens = std::cell::RefCell::new(Vec::new());
            let events = std::cell::RefCell::new(Vec::new());
            let result = login(
                frames.as_slice(),
                &mut commands,
                &tokio_util::sync::CancellationToken::new(),
                receive,
                &|event| {
                    if let LoginEvent::BrowserOpened(success) = event {
                        events.borrow_mut().push(success);
                    }
                },
                |url| {
                    opens.borrow_mut().push(url);
                    if opens.borrow().len() == 1 {
                        send_reopen.try_send(()).unwrap();
                    }
                    std::future::ready(opens.borrow().len() > 1)
                },
            )
            .await
            .unwrap();
            assert_eq!(result, crate::runtimes::LoginOutcome::Failed);
            assert_eq!(*events.borrow(), [false, true]);
            assert_eq!(opens.borrow()[0], opens.borrow()[1]);
            let sent = String::from_utf8(commands).unwrap();
            assert_eq!(sent.matches("account/login/start").count(), 1);
            assert!(!sent.contains("secret-do-not-export"));
        });
    }

    #[test]
    fn browser_cancel_during_setup_has_no_effect_and_pending_cancel_uses_native_identity() {
        run(async {
            let cancel = tokio_util::sync::CancellationToken::new();
            cancel.cancel();
            let (_send, receive) = tokio::sync::mpsc::channel(1);
            let mut commands = Vec::new();
            assert_eq!(
                login(&b""[..], &mut commands, &cancel, receive, &|_| {}, |_| {
                    std::future::ready(false)
                })
                .await
                .unwrap(),
                crate::runtimes::LoginOutcome::Cancelled
            );
            assert!(commands.is_empty());
            let cancel = tokio_util::sync::CancellationToken::new();
            let frames = b"{\"id\":1,\"result\":{}}\n{\"id\":4,\"result\":{\"providerId\":\"openai\",\"providerName\":\"OpenAI\",\"required\":false}}\n{\"id\":2,\"result\":{\"type\":\"chatgpt\",\"loginId\":\"native-1\",\"authUrl\":\"https://auth.openai.com/authorize\"}}\n{\"id\":3,\"result\":{\"status\":\"canceled\"}}\n";
            let (_send, receive) = tokio::sync::mpsc::channel(1);
            assert_eq!(
                login(
                    frames.as_slice(),
                    &mut commands,
                    &cancel,
                    receive,
                    &|_| {},
                    |_| {
                        cancel.cancel();
                        std::future::ready(true)
                    }
                )
                .await
                .unwrap(),
                crate::runtimes::LoginOutcome::Cancelled
            );
            let sent: Vec<Value> = String::from_utf8(commands)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            assert_eq!(
                sent.last().unwrap()["params"],
                json!({"loginId":"native-1"})
            );
            assert_eq!(sent.last().unwrap()["method"], "account/login/cancel");
        });
    }

    #[test]
    fn browser_timeout_uses_controlled_time_and_urls_reject_non_browser_schemes() {
        run(async {
            let (mut send_frames, read) = tokio::io::duplex(2048);
            send_frames.write_all(b"{\"id\":1,\"result\":{}}\n{\"id\":4,\"result\":{\"providerId\":\"openai\",\"providerName\":\"OpenAI\",\"required\":false}}\n{\"id\":2,\"result\":{\"type\":\"chatgpt\",\"loginId\":\"native-1\",\"authUrl\":\"https://auth.openai.com/authorize\"}}\n").await.unwrap();
            let (_send, receive) = tokio::sync::mpsc::channel(1);
            let start = Instant::now();
            let failure = login(
                read,
                Vec::new(),
                &tokio_util::sync::CancellationToken::new(),
                receive,
                &|_| {},
                |_| std::future::ready(true),
            )
            .await
            .unwrap_err();
            assert_eq!(failure.problem, SetupProblem::StatusReadFailed);
            assert_eq!(Instant::now() - start, std::time::Duration::from_secs(600));
        });
        for url in [
            "javascript:alert(1)",
            "file:///tmp/test",
            "https://user:secret@example.test",
            "http://example.test",
            "https://example.test/\n",
        ] {
            assert!(!browser_url(url, false));
        }
        assert!(browser_url(
            "https://auth.openai.com/authorize?state=synthetic",
            false
        ));
        assert!(browser_url("http://127.0.0.1:1234/fixture", true));
        assert!(!browser_url("http://127.0.0.1:1234/fixture", false));
    }

    #[test]
    fn early_completion_and_success_racing_cancel_preserve_exact_outcome() {
        run(async {
            for early in [true, false] {
                let initialized = "{\"id\":1,\"result\":{}}\n{\"id\":4,\"result\":{\"providerId\":\"openai\",\"providerName\":\"OpenAI\",\"required\":false}}\n";
                let started = "{\"id\":2,\"result\":{\"type\":\"chatgpt\",\"loginId\":\"native-1\",\"authUrl\":\"https://auth.openai.com/authorize\"}}\n";
                let completed = "{\"method\":\"account/login/completed\",\"params\":{\"loginId\":\"native-1\",\"success\":true}}\n";
                let frames = if early {
                    format!("{initialized}{completed}{started}")
                } else {
                    format!(
                        "{initialized}{started}{completed}{{\"id\":3,\"result\":{{\"status\":\"notFound\"}}}}\n"
                    )
                };
                let cancel = tokio_util::sync::CancellationToken::new();
                let (_send, receive) = tokio::sync::mpsc::channel(1);
                assert_eq!(
                    login(
                        frames.as_bytes(),
                        Vec::new(),
                        &cancel,
                        receive,
                        &|_| {},
                        |_| {
                            if !early {
                                cancel.cancel();
                            }
                            std::future::ready(true)
                        }
                    )
                    .await
                    .unwrap(),
                    crate::runtimes::LoginOutcome::Completed
                );
            }
        });
    }

    #[test]
    fn cancelled_frame_wait_retains_partial_bytes() {
        run(async {
            let (mut writer, reader) = tokio::io::duplex(128);
            let mut frames = Frames::new(reader);
            writer.write_all(b"{\"id\":").await.unwrap();
            {
                let mut next = std::pin::pin!(frames.next());
                std::future::poll_fn(|cx| {
                    assert!(std::future::Future::poll(next.as_mut(), cx).is_pending());
                    std::task::Poll::Ready(())
                })
                .await;
            }
            writer.write_all(b"1,\"result\":{}}\n").await.unwrap();
            assert_eq!(frames.next().await.unwrap(), json!({"id":1,"result":{}}));
        });
    }

    #[test]
    fn repeated_reopen_cannot_starve_the_absolute_login_deadline() {
        run(async {
            let (mut writer, read) = tokio::io::duplex(2048);
            writer.write_all(b"{\"id\":1,\"result\":{}}\n{\"id\":4,\"result\":{\"providerId\":\"openai\",\"providerName\":\"OpenAI\",\"required\":false}}\n{\"id\":2,\"result\":{\"type\":\"chatgpt\",\"loginId\":\"native-1\",\"authUrl\":\"https://auth.openai.com/authorize\"}}\n").await.unwrap();
            let (send, receive) = tokio::sync::mpsc::channel(1);
            let opens = std::cell::Cell::new(0);
            let failure = login(
                read,
                Vec::new(),
                &tokio_util::sync::CancellationToken::new(),
                receive,
                &|_| {},
                |_| {
                    let count = opens.get() + 1;
                    opens.set(count);
                    send.try_send(()).unwrap();
                    async move {
                        assert!(count <= 2, "expired login must not reopen again");
                        if count == 2 {
                            tokio::time::advance(std::time::Duration::from_secs(601)).await;
                        }
                        true
                    }
                },
            )
            .await
            .unwrap_err();
            assert_eq!(failure.problem, SetupProblem::StatusReadFailed);
            assert_eq!(opens.get(), 2);
        });
    }

    fn run(future: impl std::future::Future<Output = ()>) {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .start_paused(true)
            .build()
            .unwrap()
            .block_on(future);
    }

    #[test]
    fn bounded_frames_reject_malformed_truncated_and_oversized_messages() {
        run(async {
            assert_eq!(
                version(b"codex-cli 0.169.0\n", false),
                Err(SetupProblem::StatusReadFailed)
            );
            for bytes in [
                b"invalid\n".to_vec(),
                b"{\"id\":1,\"result\":{}}".to_vec(),
                vec![b'x'; MAX_FRAME + 1],
                b"{\"id\":9,\"result\":{}}\n".to_vec(),
            ] {
                assert!(
                    receive(&mut Frames::new(bytes.as_slice()), 1)
                        .await
                        .is_err()
                );
            }
            let bytes=b"{\"method\":\"account/updated\",\"params\":{}}\n{\"id\":1,\"result\":{}}\n{\"id\":2,\"result\":{}}\n";
            let mut stream = Frames::new(bytes.as_slice());
            assert_eq!(receive(&mut stream, 1).await.unwrap(), json!({}));
            assert_eq!(receive(&mut stream, 2).await.unwrap(), json!({}));
        });
    }

    #[test]
    fn readiness_proves_gateway_capability_before_reading_authentication() {
        run(async {
            let frames = b"{\"id\":1,\"result\":{}}\n{\"id\":4,\"error\":{\"code\":-32601,\"message\":\"private provider error\"}}\n";
            let mut sent = Vec::new();
            let failure = inspect(
                frames.as_slice(),
                &mut sent,
                Instant::now() + std::time::Duration::from_secs(15),
                "0.170.0",
            )
            .await
            .unwrap_err();
            assert_eq!(failure.problem, SetupProblem::IncompatibleProtocol);
            let commands: Vec<Value> = String::from_utf8(sent)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            assert_eq!(
                commands
                    .iter()
                    .map(|v| v["method"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                ["initialize", "initialized", "account/gatewayOAuth/read"]
            );
            assert!(!failure.diagnostic.contains("private provider error"));
        });
    }

    #[test]
    fn setup_commands_are_read_only_and_deadline_is_absolute() {
        run(async {
            let frames=b"{\"id\":1,\"result\":{}}\n{\"id\":4,\"result\":{\"providerId\":\"openai\",\"providerName\":\"OpenAI\",\"required\":false}}\n{\"id\":2,\"result\":{\"account\":null,\"requiresOpenaiAuth\":true}}\n{\"id\":3,\"result\":{\"config\":{}}}\n";
            let mut sent = Vec::new();
            let evidence = inspect(
                frames.as_slice(),
                &mut sent,
                Instant::now() + std::time::Duration::from_secs(15),
                "0.170.0",
            )
            .await
            .unwrap();
            assert_eq!(evidence.authentication, Authentication::Unavailable);
            assert_eq!(evidence.version, "0.170.0");
            let commands: Vec<Value> = String::from_utf8(sent)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            assert_eq!(
                commands
                    .iter()
                    .map(|v| v["method"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                [
                    "initialize",
                    "initialized",
                    "account/gatewayOAuth/read",
                    "account/read",
                    "config/read"
                ]
            );
            assert_eq!(commands[3]["params"], json!({"refreshToken":false}));
            assert_eq!(commands[4]["params"], json!({"includeLayers":false}));
            let (_writer, read) = tokio::io::duplex(32);
            let mut sent = Vec::new();
            let deadline = Instant::now() + std::time::Duration::from_secs(15);
            assert_eq!(
                inspect(read, &mut sent, deadline, "0.169.0")
                    .await
                    .unwrap_err()
                    .problem,
                SetupProblem::StatusReadFailed
            );
            assert_eq!(Instant::now(), deadline);
        });
    }

    #[test]
    fn required_probe_failures_block_both_readiness_and_browser_effects() {
        run(async {
            for (response, expected) in [
                (
                    json!({"id":4,"error":{"code":-32600,"message":"secret"}}),
                    SetupProblem::IncompatibleProtocol,
                ),
                (
                    json!({"id":4,"error":{"code":-32601,"message":"secret"}}),
                    SetupProblem::IncompatibleProtocol,
                ),
                (
                    json!({"id":4,"error":{"code":-32603,"message":"secret"}}),
                    SetupProblem::StatusReadFailed,
                ),
                (
                    json!({"id":4,"result":{}}),
                    SetupProblem::IncompatibleProtocol,
                ),
                (
                    json!({"id":4,"result":{"providerId":"openai","providerName":"OpenAI","required":"false"}}),
                    SetupProblem::IncompatibleProtocol,
                ),
                (
                    json!({"id":4,"result":{"providerId":null,"providerName":"OpenAI","required":false}}),
                    SetupProblem::IncompatibleProtocol,
                ),
                (
                    json!({"id":4,"result":{"providerId":"openai","providerName":[],"required":false}}),
                    SetupProblem::IncompatibleProtocol,
                ),
                (
                    json!({"id":99,"result":{"providerId":"openai","providerName":"OpenAI","required":false}}),
                    SetupProblem::IncompatibleProtocol,
                ),
                (
                    json!({"id":4,"method":"item/commandExecution/requestApproval","params":{}}),
                    SetupProblem::IncompatibleProtocol,
                ),
            ] {
                let frames = format!("{{\"id\":1,\"result\":{{}}}}\n{response}\n");
                for browser in [false, true] {
                    let mut sent = Vec::new();
                    let opens = std::cell::Cell::new(0);
                    let failure = if browser {
                        let (_send, receive) = tokio::sync::mpsc::channel(1);
                        login(
                            frames.as_bytes(),
                            &mut sent,
                            &tokio_util::sync::CancellationToken::new(),
                            receive,
                            &|_| {},
                            |_| {
                                opens.set(opens.get() + 1);
                                std::future::ready(true)
                            },
                        )
                        .await
                        .unwrap_err()
                    } else {
                        inspect(
                            frames.as_bytes(),
                            &mut sent,
                            Instant::now() + std::time::Duration::from_secs(15),
                            "0.170.0",
                        )
                        .await
                        .unwrap_err()
                    };
                    assert_eq!(failure.problem, expected);
                    assert!(!failure.diagnostic.contains("secret"));
                    assert_eq!(opens.get(), 0);
                    let commands: Vec<Value> = String::from_utf8(sent)
                        .unwrap()
                        .lines()
                        .map(|line| serde_json::from_str(line).unwrap())
                        .collect();
                    assert_eq!(
                        commands
                            .iter()
                            .map(|v| v["method"].as_str().unwrap())
                            .collect::<Vec<_>>(),
                        ["initialize", "initialized", "account/gatewayOAuth/read"]
                    );
                }
            }
        });
    }

    #[test]
    fn probe_accepts_additive_fields_without_treating_support_as_authentication() {
        run(async {
            let frames = [
                json!({"id":1,"result":{"futureField":42}}),
                json!({"method":"future/information","params":{}}),
                json!({"id":4,"result":{"providerId":"x".repeat(513),"providerName":"Gateway\nDisplay","required":true,"status":"notReady","futureField":true}}),
                json!({"id":2,"result":{"account":null,"requiresOpenaiAuth":true,"futureField":true}}),
                json!({"id":3,"result":{"config":{},"futureField":true}}),
            ].iter().map(Value::to_string).collect::<Vec<_>>().join("\n") + "\n";
            let mut sent = Vec::new();
            let evidence = inspect(
                frames.as_bytes(),
                &mut sent,
                Instant::now() + std::time::Duration::from_secs(15),
                "1.0.0+build.7",
            )
            .await
            .unwrap();
            assert_eq!(evidence.version, "1.0.0+build.7");
            assert_eq!(evidence.authentication, Authentication::Unavailable);
            assert_eq!(evidence.billing, BillingRoute::Unknown);
            let initialize: Value =
                serde_json::from_str(String::from_utf8(sent).unwrap().lines().next().unwrap())
                    .unwrap();
            assert_eq!(
                initialize["params"]["capabilities"],
                json!({"explicitGatewayOauth":true})
            );
        });
    }

    #[test]
    fn capability_probe_keeps_absolute_deadlines_and_cancelled_login_has_no_effect() {
        run(async {
            for browser in [false, true] {
                let (mut writer, reader) = tokio::io::duplex(128);
                writer
                    .write_all(b"{\"id\":1,\"result\":{}}\n")
                    .await
                    .unwrap();
                let mut sent = Vec::new();
                let start = Instant::now();
                let failure = if browser {
                    let (_send, receive) = tokio::sync::mpsc::channel(1);
                    login(
                        reader,
                        &mut sent,
                        &tokio_util::sync::CancellationToken::new(),
                        receive,
                        &|_| {},
                        |_| async { panic!("probe timeout cannot open a browser") },
                    )
                    .await
                    .unwrap_err()
                } else {
                    inspect(
                        reader,
                        &mut sent,
                        start + std::time::Duration::from_secs(15),
                        "0.170.0",
                    )
                    .await
                    .unwrap_err()
                };
                assert_eq!(failure.problem, SetupProblem::StatusReadFailed);
                assert_eq!(Instant::now() - start, std::time::Duration::from_secs(15));
                let sent = String::from_utf8(sent).unwrap();
                assert!(sent.contains("account/gatewayOAuth/read"));
                assert!(!sent.contains("account/read"));
                assert!(!sent.contains("account/login/start"));
                drop(writer);
            }
            let (mut writer, reader) = tokio::io::duplex(128);
            writer
                .write_all(b"{\"id\":1,\"result\":{}}\n")
                .await
                .unwrap();
            let cancel = tokio_util::sync::CancellationToken::new();
            let (_send, receive) = tokio::sync::mpsc::channel(1);
            let mut sent = Vec::new();
            let mut attempt = Box::pin(login(
                reader,
                &mut sent,
                &cancel,
                receive,
                &|_| {},
                |_| async { panic!("cancelled probe cannot open a browser") },
            ));
            std::future::poll_fn(|cx| {
                assert!(std::future::Future::poll(attempt.as_mut(), cx).is_pending());
                std::task::Poll::Ready(())
            })
            .await;
            cancel.cancel();
            assert_eq!(
                attempt.await.unwrap(),
                crate::runtimes::LoginOutcome::Cancelled
            );
            let sent = String::from_utf8(sent).unwrap();
            assert!(sent.contains("account/gatewayOAuth/read"));
            assert!(!sent.contains("account/login/start"));
        });
    }

    #[test]
    fn unsupported_versions_and_malformed_output_are_rejected() {
        for bytes in [
            b"codex-cli 0.168.9\n".as_slice(),
            b"codex-cli 0.170.0-alpha.1\n",
            b"codex-cli 1.0.0-nightly+build.7\n",
            b"codex-cli 0.169\n",
            b"codex-cli 00.169.0\n",
            b"codex-cli 0.170.0\nextra\n",
            b"codex-cli 0.170.0 \n",
            b"codex-cli 0.170.0",
            b"codex-cli 0.170.0\0\n",
            b"codex-cli \xff\n",
        ] {
            assert_eq!(
                version(bytes, true),
                Err(SetupProblem::IncompatibleProtocol)
            );
        }
        assert_eq!(
            version(b"codex-cli 0.170.0+build.7\r\n", true).unwrap(),
            "0.170.0+build.7"
        );
    }

    #[test]
    fn stable_releases_after_the_baseline_do_not_need_an_allowlist() {
        for bytes in [
            b"codex-cli 0.169.0\n".as_slice(),
            b"codex-cli 0.170.0\r\n",
            b"codex-cli 1.0.0\n",
            b"codex-cli 0.169.0+build.7\n",
        ] {
            assert!(version(bytes, true).is_ok(), "{bytes:?}");
        }
    }

    #[test]
    fn version_and_configuration_failures_do_not_invent_authentication_or_billing() {
        assert!(version(b"codex-cli 0.169.0\n", true).is_ok());
        for bytes in [
            b"codex-cli 0.167.1\n".as_slice(),
            b"",
            b"codex-cli nightly\n",
        ] {
            assert_eq!(
                version(bytes, true),
                Err(SetupProblem::IncompatibleProtocol)
            );
        }
        let signed_out = json!({"account":null,"requiresOpenaiAuth":false});
        let evidence = setup(
            &signed_out,
            &json!({"config":{"model_provider":"custom"}}),
            "0.169.0",
        )
        .unwrap();
        assert_eq!(evidence.authentication, Authentication::NotRequired);
        assert_eq!(evidence.billing, BillingRoute::Unknown);
        assert!(
            setup(
                &json!({"account":{"type":"apiKey"}}),
                &json!({"config":{}}),
                "0.169.0"
            )
            .is_err()
        );
        assert!(setup(&signed_out, &json!({}), "0.169.0").is_err());
    }

    #[test]
    fn rpc_failure_diagnostics_keep_codes_and_discard_provider_messages() {
        run(async {
            let bytes =
                b"{\"id\":3,\"error\":{\"code\":-32603,\"message\":\"do-not-export-secret\"}}\n";
            let failure = receive(&mut Frames::new(bytes.as_slice()), 3)
                .await
                .unwrap_err();
            assert_eq!(failure.problem, SetupProblem::StatusReadFailed);
            assert!(failure.diagnostic.contains("-32603"));
            assert!(!failure.diagnostic.contains("do-not-export-secret"));
        });
    }

    #[test]
    fn existing_authentication_is_evidence_not_a_login_or_billing_choice() {
        let config = json!({"config":{"model_provider":"openai","model":"gpt-5"},"origins":{},"layers":null});
        let evidence = setup(&json!({"account":{"type":"chatgpt","email":"fixture@example.test","planType":"pro","workspaceName":null},"userId":null,"requiresOpenaiAuth":true}), &config, "0.169.0").unwrap();
        assert_eq!(evidence.authentication, Authentication::Chatgpt);
        assert_eq!(evidence.billing, BillingRoute::Chatgpt);
        let api = setup(
            &json!({"account":{"type":"apiKey"},"userId":null,"requiresOpenaiAuth":true}),
            &config,
            "0.169.0",
        )
        .unwrap();
        assert_eq!(api.authentication, Authentication::ApiKey);
        assert_eq!(api.billing, BillingRoute::ApiKey);
    }

    #[test]
    fn protocol_never_accepts_an_unrelated_reply_or_a_server_request() {
        assert!(reply(json!({"id":99,"result":{}}), 2).is_err());
        assert!(
            reply(
                json!({"id":2,"method":"item/commandExecution/requestApproval","params":{}}),
                2
            )
            .is_err()
        );
        assert!(reply(json!({"id":2,"result":{},"error":{}}), 2).is_err());
        assert!(reply(json!({"id":2,"error":{"code":-1,"message":"secret"}}), 2).is_err());
        assert!(reply(json!({"id":2,"result":{}}), 2).is_ok());
    }
}
