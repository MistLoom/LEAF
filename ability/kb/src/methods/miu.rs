//! MIU 落库与维护：单工具单动作

use std::sync::Arc;

use crate::core::{KbError, KbId, MiuDraft, MiuId};
use crate::store::KbHub;
use ability_support::Error;
use serde::Deserialize;
use serde_json::{json, Value};

fn map_err(e: KbError) -> Error {
    Error::failed(e.to_string())
}

async fn store(hub: &KbHub, kb_id: &str) -> Result<Arc<dyn crate::store::KbStore>, Error> {
    let id = KbId::new(kb_id).map_err(|e| Error::invalid_args(e.to_string()))?;
    hub.get(&id).await.map_err(map_err)
}

pub(crate) struct GetTool {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct GetArgs {
    kb_id: String,
    miu_id: String,
}
impl GetTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: GetArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let mid = MiuId::new(a.miu_id).map_err(|e| Error::invalid_args(e.to_string()))?;
        let miu = kb.get_miu(&mid).await.map_err(map_err)?;
        serde_json::to_value(miu).map_err(|e| Error::failed(e.to_string()))
    }
}

pub(crate) struct PutTool {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct PutArgs {
    kb_id: String,
    draft: MiuDraft,
}
impl PutTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: PutArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let miu = kb.put_miu(a.draft).await.map_err(map_err)?;
        Ok(json!({ "miu": miu }))
    }
}

pub(crate) struct PutManyTool {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct PutManyArgs {
    kb_id: String,
    drafts: Vec<MiuDraft>,
}
impl PutManyTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: PutManyArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let mius = kb.put_many(a.drafts).await.map_err(map_err)?;
        Ok(json!({ "mius": mius }))
    }
}

pub(crate) struct PatchTool {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct PatchArgs {
    kb_id: String,
    miu_id: String,
    patch: Value,
}
impl PatchTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: PatchArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let mid = MiuId::new(a.miu_id).map_err(|e| Error::invalid_args(e.to_string()))?;
        let miu = kb.patch_miu(&mid, a.patch).await.map_err(map_err)?;
        Ok(json!({ "miu": miu }))
    }
}

pub(crate) struct MarkStaleTool {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct IdArgs {
    kb_id: String,
    miu_id: String,
}
impl MarkStaleTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: IdArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let mid = MiuId::new(a.miu_id).map_err(|e| Error::invalid_args(e.to_string()))?;
        let miu = kb.mark_stale(&mid).await.map_err(map_err)?;
        Ok(json!({ "miu": miu }))
    }
}

pub(crate) struct DeleteTool {
    pub(crate) hub: Arc<KbHub>,
}
impl DeleteTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: IdArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let mid = MiuId::new(a.miu_id).map_err(|e| Error::invalid_args(e.to_string()))?;
        kb.delete_miu(&mid).await.map_err(map_err)?;
        Ok(json!({ "ok": true }))
    }
}

pub(crate) struct LinkTool {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct LinkArgs {
    kb_id: String,
    miu_id: String,
    prev_id: Option<String>,
    next_id: Option<String>,
}
impl LinkTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: LinkArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let mid = MiuId::new(a.miu_id).map_err(|e| Error::invalid_args(e.to_string()))?;
        let prev = a
            .prev_id
            .map(MiuId::new)
            .transpose()
            .map_err(|e| Error::invalid_args(e.to_string()))?;
        let next = a
            .next_id
            .map(MiuId::new)
            .transpose()
            .map_err(|e| Error::invalid_args(e.to_string()))?;
        let miu = kb.link_context(&mid, prev, next).await.map_err(map_err)?;
        Ok(json!({ "miu": miu }))
    }
}
