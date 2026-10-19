use std::sync::Arc;

use ability_store::{GetArgs, ListArgs, PutArgs};
use ability_store_persist::PersistExt;
use ability_support::AbilityCtx;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::core::KbError;

const OWNER: &str = "kb";

#[derive(Clone)]
pub(crate) struct StoreIo {
    ctx: Arc<RwLock<Option<Arc<dyn AbilityCtx>>>>,
}

impl StoreIo {
    pub fn new() -> Self {
        Self {
            ctx: Arc::new(RwLock::new(None)),
        }
    }

    async fn ctx(&self) -> Result<Arc<dyn AbilityCtx>, KbError> {
        self.ctx
            .read()
            .await
            .clone()
            .ok_or_else(|| KbError::Store("store.persist 未绑定".into()))
    }

    pub async fn bind(&self, ctx: Arc<dyn AbilityCtx>) {
        *self.ctx.write().await = Some(ctx);
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, KbError> {
        let ctx = self.ctx().await?;
        let out = ctx
            .persist()
            .get(GetArgs {
                owner: Some(OWNER.into()),
                space: None,
                key: key.into(),
                codec: Some("json".into()),
            })
            .await
            .map_err(|e| KbError::Store(e.to_string()))?;
        match out.value {
            Some(v) => serde_json::from_value(v)
                .map_err(|e| KbError::Store(e.to_string()))
                .map(Some),
            None => Ok(None),
        }
    }

    pub async fn put_json<T: Serialize>(&self, key: &str, value: &T) -> Result<(), KbError> {
        let ctx = self.ctx().await?;
        let v = serde_json::to_value(value).map_err(|e| KbError::Store(e.to_string()))?;
        ctx.persist()
            .put(PutArgs {
                owner: Some(OWNER.into()),
                space: None,
                key: key.into(),
                value: v,
                codec: Some("json".into()),
            })
            .await
            .map_err(|e| KbError::Store(e.to_string()))?;
        Ok(())
    }

    pub async fn purge_prefix(&self, prefix: &str) -> Result<(), KbError> {
        let ctx = self.ctx().await?;
        ctx.persist()
            .purge(ListArgs {
                owner: Some(OWNER.into()),
                space: None,
                prefix: prefix.into(),
            })
            .await
            .map_err(|e| KbError::Store(e.to_string()))?;
        Ok(())
    }
}
