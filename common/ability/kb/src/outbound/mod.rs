//! 通用出站通知：与具体业务域解耦。

mod error;
mod hub;
mod memory;

pub use error::OutboundError;
pub use hub::{OutboundHub, SharedOutbound, Subscription};
pub use memory::MemoryOutbound;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 通用出站事件信封。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutboundEvent {
    pub topic: String,
    pub payload: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    pub ts: u64,
}

#[async_trait]
pub trait Notifier: Send + Sync {
    async fn emit(&self, event: OutboundEvent) -> Result<(), OutboundError>;
}
