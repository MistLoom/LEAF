use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::core::{KbError, KbId, ScenarioConfig};
use crate::outbound::{OutboundHub, SharedOutbound};
use crate::store::io::StoreIo;

use super::local::LocalKb;
use super::store::KbStore;

/// 多知识库注册与打开入口。
pub struct KbHub {
    io: StoreIo,
    outbound: SharedOutbound,
    open: RwLock<HashMap<String, Arc<LocalKb>>>,
}

impl KbHub {
    pub fn new() -> Self {
        Self {
            io: StoreIo::new(),
            outbound: Arc::new(OutboundHub::new()),
            open: RwLock::new(HashMap::new()),
        }
    }

    pub async fn bind_ctx(&self, ctx: Arc<dyn ability_support::AbilityCtx>) {
        self.io.bind(ctx).await;
    }

    pub fn outbound(&self) -> SharedOutbound {
        self.outbound.clone()
    }

    pub async fn create_kb(
        &self,
        name: impl Into<String>,
        config: ScenarioConfig,
    ) -> Result<Arc<LocalKb>, KbError> {
        let kb = LocalKb::create(self.io.clone(), name, config, self.outbound.clone()).await?;
        let id = kb.id().to_string();
        let arc = Arc::new(kb);
        self.open.write().await.insert(id, arc.clone());
        Ok(arc)
    }

    pub async fn open_kb(&self, kb_id: &KbId) -> Result<Arc<LocalKb>, KbError> {
        {
            let guard = self.open.read().await;
            if let Some(kb) = guard.get(kb_id.as_str()) {
                return Ok(kb.clone());
            }
        }
        let kb = LocalKb::open(self.io.clone(), kb_id, self.outbound.clone()).await?;
        let arc = Arc::new(kb);
        self.open
            .write()
            .await
            .insert(kb_id.to_string(), arc.clone());
        Ok(arc)
    }

    pub async fn drop_kb(&self, kb_id: &KbId) -> Result<(), KbError> {
        self.open.write().await.remove(kb_id.as_str());
        self.io.purge_prefix(&format!("{}/", kb_id.as_str())).await?;
        Ok(())
    }

    pub async fn get(&self, kb_id: &KbId) -> Result<Arc<dyn KbStore>, KbError> {
        let kb = self.open_kb(kb_id).await?;
        Ok(kb as Arc<dyn KbStore>)
    }
}
