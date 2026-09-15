//! 统计域：不改库

use std::sync::Arc;

use crate::core::{KbError, KbId, ScopeId};
use crate::store::{DimQuery, KbHub};
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

pub(crate) struct DimTool {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct DimArgs {
    kb_id: String,
    dimension: String,
    #[serde(default)]
    scope_ids: Vec<String>,
}
impl DimTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: DimArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let scope_ids = if a.scope_ids.is_empty() {
            None
        } else {
            Some(
                a.scope_ids
                    .into_iter()
                    .map(ScopeId::new)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| Error::invalid_args(e.to_string()))?,
            )
        };
        let stat = kb
            .stats_dim(DimQuery {
                dimension: a.dimension,
                scope_ids,
            })
            .await
            .map_err(map_err)?;
        serde_json::to_value(stat).map_err(|e| Error::failed(e.to_string()))
    }
}

pub(crate) struct DupTool {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct DupArgs {
    kb_id: String,
    text: String,
    fingerprint: Option<String>,
    #[serde(default)]
    scope_ids: Vec<String>,
}
impl DupTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: DupArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let sc: Vec<ScopeId> = a
            .scope_ids
            .into_iter()
            .map(ScopeId::new)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Error::invalid_args(e.to_string()))?;
        let est = kb
            .dup_prob(
                &a.text,
                a.fingerprint.as_deref(),
                if sc.is_empty() { None } else { Some(sc.as_slice()) },
            )
            .await
            .map_err(map_err)?;
        serde_json::to_value(est).map_err(|e| Error::failed(e.to_string()))
    }
}

pub(crate) struct FreshnessTool {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct KbOnly {
    kb_id: String,
}
impl FreshnessTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: KbOnly =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        // status 维度近似新鲜度分
        let stat = kb
            .stats_dim(DimQuery {
                dimension: "status".into(),
                scope_ids: None,
            })
            .await
            .map_err(map_err)?;
        Ok(json!({ "freshness": stat }))
    }
}

pub(crate) struct CoverageTool {
    pub(crate) hub: Arc<KbHub>,
}
impl CoverageTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: KbOnly =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let by_scope = kb
            .stats_dim(DimQuery {
                dimension: "scope".into(),
                scope_ids: None,
            })
            .await
            .map_err(map_err)?;
        let by_source = kb
            .stats_dim(DimQuery {
                dimension: "source_type".into(),
                scope_ids: None,
            })
            .await
            .map_err(map_err)?;
        Ok(json!({ "by_scope": by_scope, "by_source_type": by_source }))
    }
}
