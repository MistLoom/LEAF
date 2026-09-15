use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{Notifier, OutboundError, OutboundEvent};

/// 仅收集事件、不做订阅过滤；用于单测与场景断言。
pub struct MemoryOutbound {
    events: RwLock<Vec<OutboundEvent>>,
}

impl MemoryOutbound {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
        }
    }

    pub fn shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    pub async fn events(&self) -> Vec<OutboundEvent> {
        self.events.read().await.clone()
    }

    pub async fn clear(&self) {
        self.events.write().await.clear();
    }
}

impl Default for MemoryOutbound {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Notifier for MemoryOutbound {
    async fn emit(&self, event: OutboundEvent) -> Result<(), OutboundError> {
        self.events.write().await.push(event);
        Ok(())
    }
}
