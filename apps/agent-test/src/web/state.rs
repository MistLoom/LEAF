//! Web 共享状态。

use std::sync::Arc;

use tokio::sync::broadcast;

use crate::runtime::Runtime;

pub struct WebState {
    pub rt: Runtime,
    pub spawn_profile: frame_core::AbilityExpose,
    pub events_tx: broadcast::Sender<String>,
    pub restored_count: usize,
}

impl WebState {
    pub fn gateway(&self) -> &Arc<dyn frame_core::Gateway> {
        &self.rt.gateway
    }
}
