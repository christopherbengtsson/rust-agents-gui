use crate::runtimes::{Authentication, BillingRoute, SetupEvidence, SetupFailure, SetupProblem};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::time::{Instant, timeout_at};

pub(super) const VERSION: &str = "0.169.0";
pub(super) const MAX_FRAME: usize = 256 * 1024;

pub(super) fn version(bytes: &[u8], successful_exit: bool) -> Result<(), SetupProblem> {
    if !successful_exit {
        return Err(SetupProblem::StatusReadFailed);
    }
    if bytes == b"codex-cli 0.169.0\n" || bytes == b"codex-cli 0.169.0\r\n" {
        Ok(())
    } else {
        Err(SetupProblem::IncompatibleProtocol)
    }
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

fn setup(account: &Value, config: &Value) -> Result<SetupEvidence, SetupProblem> {
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
        version: VERSION.into(),
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
    stream: &mut BufReader<S>,
    id: u32,
) -> Result<Value, SetupFailure> {
    let mut bytes = Vec::new();
    let mut notifications = 0;
    loop {
        let available = stream
            .fill_buf()
            .await
            .map_err(|error| SetupFailure::io("provider stdout read", error))?;
        if available.is_empty() {
            return Err(SetupFailure::at(
                SetupProblem::StatusReadFailed,
                format!("EOF before complete response {id}"),
            ));
        }
        let count = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|n| n + 1)
            .unwrap_or(available.len());
        if bytes.len() + count > MAX_FRAME {
            return Err(SetupFailure::at(
                SetupProblem::IncompatibleProtocol,
                "Provider frame exceeded 262144 bytes",
            ));
        }
        bytes.extend_from_slice(&available[..count]);
        stream.consume(count);
        if bytes.last() == Some(&b'\n') {
            let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
                SetupFailure::at(
                    SetupProblem::IncompatibleProtocol,
                    format!(
                        "Provider JSON {:?} error at line {}, column {}",
                        error.classify(),
                        error.line(),
                        error.column()
                    ),
                )
            })?;
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
                bytes.clear();
                continue;
            }
            let rpc_code = value
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(Value::as_i64);
            return reply(value,id).map_err(|problem|SetupFailure::at(problem,format!("Setup response {id} rejected; RPC code {rpc_code:?}; expected exact response ID and no server request")));
        }
    }
}
pub(super) async fn inspect<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    read: R,
    mut write: W,
    deadline: Instant,
) -> Result<SetupEvidence, SetupFailure> {
    timeout_at(deadline,async {
        let mut read=BufReader::new(read);
        send(&mut write,json!({"id":1,"method":"initialize","params":{"clientInfo":{"name":"agent_workspace","version":env!("CARGO_PKG_VERSION")},"capabilities":{"explicitGatewayOauth":true}}})).await?;
        let initialized=receive(&mut read,1).await?;
        if !initialized.is_object() {return Err(SetupFailure::at(SetupProblem::IncompatibleProtocol,"initialize result must be an object"));}
        send(&mut write,json!({"method":"initialized","params":{}})).await?;
        send(&mut write,json!({"id":2,"method":"account/read","params":{"refreshToken":false}})).await?;
        let account=receive(&mut read,2).await?;
        send(&mut write,json!({"id":3,"method":"config/read","params":{"includeLayers":false}})).await?;
        let config=receive(&mut read,3).await?;
        setup(&account,&config).map_err(|problem|SetupFailure::at(problem,"Invalid or unsupported account/configuration fields"))
    }).await.map_err(|_|SetupFailure::at(SetupProblem::StatusReadFailed,"Readiness protocol exceeded its absolute 15-second deadline"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtimes::{Authentication, BillingRoute};
    use serde_json::json;

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
                    receive(&mut BufReader::new(bytes.as_slice()), 1)
                        .await
                        .is_err()
                );
            }
            let bytes=b"{\"method\":\"account/updated\",\"params\":{}}\n{\"id\":1,\"result\":{}}\n{\"id\":2,\"result\":{}}\n";
            let mut stream = BufReader::new(bytes.as_slice());
            assert_eq!(receive(&mut stream, 1).await.unwrap(), json!({}));
            assert_eq!(receive(&mut stream, 2).await.unwrap(), json!({}));
        });
    }

    #[test]
    fn setup_commands_are_read_only_and_deadline_is_absolute() {
        run(async {
            let frames=b"{\"id\":1,\"result\":{}}\n{\"id\":2,\"result\":{\"account\":null,\"requiresOpenaiAuth\":true}}\n{\"id\":3,\"result\":{\"config\":{}}}\n";
            let mut sent = Vec::new();
            let evidence = inspect(
                frames.as_slice(),
                &mut sent,
                Instant::now() + std::time::Duration::from_secs(15),
            )
            .await
            .unwrap();
            assert_eq!(evidence.authentication, Authentication::Unavailable);
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
                ["initialize", "initialized", "account/read", "config/read"]
            );
            assert_eq!(commands[2]["params"], json!({"refreshToken":false}));
            assert_eq!(commands[3]["params"], json!({"includeLayers":false}));
            let (_writer, read) = tokio::io::duplex(32);
            let mut sent = Vec::new();
            let deadline = Instant::now() + std::time::Duration::from_secs(15);
            assert_eq!(
                inspect(read, &mut sent, deadline)
                    .await
                    .unwrap_err()
                    .problem,
                SetupProblem::StatusReadFailed
            );
            assert_eq!(Instant::now(), deadline);
        });
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
        let evidence = setup(&signed_out, &json!({"config":{"model_provider":"custom"}})).unwrap();
        assert_eq!(evidence.authentication, Authentication::NotRequired);
        assert_eq!(evidence.billing, BillingRoute::Unknown);
        assert!(setup(&json!({"account":{"type":"apiKey"}}), &json!({"config":{}})).is_err());
        assert!(setup(&signed_out, &json!({})).is_err());
    }

    #[test]
    fn rpc_failure_diagnostics_keep_codes_and_discard_provider_messages() {
        run(async {
            let bytes =
                b"{\"id\":3,\"error\":{\"code\":-32603,\"message\":\"do-not-export-secret\"}}\n";
            let failure = receive(&mut BufReader::new(bytes.as_slice()), 3)
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
        let evidence = setup(&json!({"account":{"type":"chatgpt","email":"fixture@example.test","planType":"pro","workspaceName":null},"userId":null,"requiresOpenaiAuth":true}), &config).unwrap();
        assert_eq!(evidence.authentication, Authentication::Chatgpt);
        assert_eq!(evidence.billing, BillingRoute::Chatgpt);
        let api = setup(
            &json!({"account":{"type":"apiKey"},"userId":null,"requiresOpenaiAuth":true}),
            &config,
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
