//! Native action results and bounded delivery resources. No application peer transport.
use crate::workspaces::{ErrorCode, RepositoryError};
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    io,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
mod admission;
pub(crate) mod subscription;
pub(crate) mod tasks;
pub(crate) use admission::ConnectionSlots;
pub use subscription::{FeedAck, FeedBatch, FeedEvent};

/// Evidence about this exchange only, never the outcome of an earlier attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Delivery {
    NotSent,
    Rejected,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum TransportCode {
    Busy,
    NotConnected,
    IncarnationChanged,
    ReplyUnavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct TransportFailure {
    pub code: TransportCode,
    pub delivery: Delivery,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RequestFailure {
    Transport {
        error: TransportFailure,
    },
    /// Local input rejection, before any transport access.
    Rejected {
        error: RepositoryError,
    },
}
impl RequestFailure {
    pub fn transport(code: TransportCode, delivery: Delivery, message: impl Into<String>) -> Self {
        Self::Transport {
            error: transport_failure(code, delivery, message),
        }
    }
}
fn transport_failure(
    code: TransportCode,
    delivery: Delivery,
    message: impl Into<String>,
) -> TransportFailure {
    TransportFailure {
        code,
        delivery,
        message: RepositoryError::new(ErrorCode::Connection, message).message,
    }
}

/// Serving identity is returned even when the domain rejected the request.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct OwnedReply<T> {
    pub incarnation: String,
    pub result: Result<T, RepositoryError>,
}

fn unavailable(message: impl std::fmt::Display) -> RequestFailure {
    RequestFailure::transport(
        TransportCode::ReplyUnavailable,
        Delivery::Unknown,
        format!(
            "Native reply unavailable: {message}. Refresh recorded action status; no action has been replayed."
        ),
    )
}
#[cfg(test)]
mod tests;

#[cfg(feature = "codegen")]
pub(crate) fn export_contracts(exports: &mut crate::codegen::Exports) {
    exports.add::<Delivery>();
    exports.add::<TransportCode>();
    exports.add::<TransportFailure>();
    exports.add::<RequestFailure>();
    exports.owned_replies();
    subscription::export_contracts(exports);
}
