//! Pinned Codex metadata protocol.
use super::{ErrorCode, Result, RuntimeError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub(super) const MAX_FRAME: usize = 256 * 1024;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum RequestId {
    String(String),
    Integer(i64),
}

pub(super) fn failure() -> RuntimeError {
    RuntimeError::new(
        ErrorCode::Protocol,
        "Codex returned an incompatible or unexpected metadata message. Install the supported version and check setup again.",
    )
}

#[derive(Default)]
pub(super) struct Frames {
    pending: Vec<u8>,
}
impl Frames {
    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<Value>> {
        let mut frames = Vec::new();
        for &byte in bytes {
            if self.pending.len() >= MAX_FRAME {
                return Err(RuntimeError::new(
                    ErrorCode::OutputLimit,
                    "Codex exceeded the setup frame limit. Close unexpected startup tools and check its configuration externally.",
                ));
            }
            if byte == b'\n' {
                if !self.pending.is_empty() {
                    frames.push(serde_json::from_slice(&self.pending).map_err(|_| failure())?);
                }
                self.pending.clear();
            } else {
                self.pending.push(byte);
            }
        }
        Ok(frames)
    }
}

pub(super) fn request(id: i64, method: &str, params: Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(&json!({"id":id,"method":method,"params":params}))
        .expect("metadata parameters serialize");
    bytes.push(b'\n');
    bytes
}
pub(super) fn initialized() -> Vec<u8> {
    b"{\"method\":\"initialized\",\"params\":{}}\n".to_vec()
}

/// Unexpected server requests fail closed. They never reach approval code.
pub(super) fn response(frame: Value, expected: i64) -> Result<Option<Value>> {
    let object = frame.as_object().ok_or_else(failure)?;
    if object.contains_key("method") {
        if object.contains_key("id") {
            return Err(failure());
        }
        if object.get("method").and_then(Value::as_str).is_none()
            || object.contains_key("result")
            || object.contains_key("error")
        {
            return Err(failure());
        }
        return Ok(None);
    }
    let id: RequestId = serde_json::from_value(object.get("id").cloned().ok_or_else(failure)?)
        .map_err(|_| failure())?;
    if id != RequestId::Integer(expected) {
        return Err(failure());
    }
    match (object.get("result"), object.get("error")) {
        (Some(value), None) => Ok(Some(value.clone())),
        (None, Some(error)) if error.get("code").and_then(Value::as_i64).is_some() => {
            Err(failure())
        }
        _ => Err(failure()),
    }
}

pub(super) fn initialize_params() -> Value {
    json!({"clientInfo":{"name":"agent_workspace","title":"Agent Workspace","version":env!("CARGO_PKG_VERSION")},"capabilities":{"experimentalApi":false,"requestAttestation":false}})
}
pub(super) fn validate_initialize(value: &Value) -> Result<()> {
    for key in ["userAgent", "codexHome", "platformFamily", "platformOs"] {
        if value.get(key).and_then(Value::as_str).is_none() {
            return Err(failure());
        }
    }
    Ok(())
}

pub(super) struct Metadata {
    pub authentication: String,
    pub provider: Option<String>,
    pub billing: String,
}
pub(super) fn metadata(account: &Value, configuration: &Value) -> Result<Metadata> {
    let requires = account
        .get("requiresOpenaiAuth")
        .and_then(Value::as_bool)
        .ok_or_else(failure)?;
    let account = account.get("account").ok_or_else(failure)?;
    let kind = if account.is_null() {
        None
    } else {
        Some(
            account
                .get("type")
                .and_then(Value::as_str)
                .ok_or_else(failure)?,
        )
    };
    if requires && kind.is_none() {
        return Err(RuntimeError::new(
            ErrorCode::Authentication,
            "Codex authentication is unavailable. Sign in with the official Codex CLI outside this application, then check setup again.",
        ));
    }
    let config = configuration
        .get("config")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            RuntimeError::new(
                ErrorCode::Configuration,
                "Codex configuration could not be read. Review it with the official CLI.",
            )
        })?;
    let provider = match config.get("model_provider") {
        Some(Value::String(p)) if p.len() <= 128 && !p.chars().any(char::is_control) => {
            Some(p.clone())
        }
        Some(Value::Null) | None => None,
        _ => return Err(failure()),
    };
    let authentication = match kind {
        Some("chatgpt") => "Existing ChatGPT account",
        Some("apiKey") => "Existing API-key configuration",
        Some("amazonBedrock") => "Existing Amazon Bedrock configuration",
        Some("gcp") => "Existing GCP configuration",
        None if !requires => "Not required by this provider; access unverified",
        _ => "Unknown",
    }
    .to_owned();
    // Account evidence alone does not establish a custom provider's billing route.
    let billing = match (provider.as_deref(), kind) {
        (Some("openai"), Some("chatgpt")) => "ChatGPT account",
        (Some("openai"), Some("apiKey")) => "OpenAI API",
        _ => "Unknown",
    }
    .to_owned();
    Ok(Metadata {
        authentication,
        provider,
        billing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_identifier_types_and_unexpected_requests_fail_closed() {
        for frame in [
            json!({"id":"1","result":{}}),
            json!({"id":2,"result":{}}),
            json!({"id":1.5,"result":{}}),
            json!({"id":1,"method":"item/commandExecution/requestApproval","params":{}}),
        ] {
            assert!(response(frame, 1).is_err());
        }
        assert!(response(json!({"id":1,"result":{}}), 1).unwrap().is_some());
        assert_eq!(
            serde_json::from_str::<RequestId>("9223372036854775807").unwrap(),
            RequestId::Integer(i64::MAX)
        );
    }
    #[test]
    fn chunked_frames_enforce_bounds_and_reject_malformed_json() {
        let mut frames = Frames::default();
        assert!(frames.push(b"{\"id\":").unwrap().is_empty());
        assert_eq!(frames.push(b"1,\"result\":{}}\n").unwrap().len(), 1);
        assert!(frames.push(b"bad\n").is_err());
        assert!(Frames::default().push(&vec![b'x'; MAX_FRAME + 1]).is_err());
    }
    #[test]
    fn authentication_and_billing_require_evidence_without_revealing_account_details() {
        let config = json!({"config":{"model_provider":"openai","secret":"do-not-retain"}});
        assert_eq!(
            metadata(&json!({"account":null,"requiresOpenaiAuth":true}), &config)
                .err()
                .unwrap()
                .code,
            ErrorCode::Authentication
        );
        let m=metadata(&json!({"account":{"type":"chatgpt","email":"private@example.test"},"requiresOpenaiAuth":true}),&config).unwrap();
        assert_eq!(m.billing, "ChatGPT account");
        assert_eq!(m.authentication, "Existing ChatGPT account");
        assert_eq!(
            metadata(
                &json!({"account":{"type":"apiKey"},"requiresOpenaiAuth":false}),
                &json!({"config":{"model_provider":"custom"}})
            )
            .unwrap()
            .billing,
            "Unknown"
        );
    }
}
