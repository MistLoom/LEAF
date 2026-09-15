//! 库与 Scope：每Tool 仅一个动作

use std::sync::Arc;

use crate::core::{KbError, KbId, ScenarioConfig, ScopeId, SourceRef};
use crate::store::{KbHub, KbStore};
use ability_support::Error;
use serde::Deserialize;
use serde_json::{json, Value};

fn map_err(e: KbError) -> Error {
    Error::failed(e.to_string())
}

fn parse_kb_id(s: &str) -> Result<KbId, Error> {
    KbId::new(s).map_err(|e| Error::invalid_args(e.to_string()))
}

pub(crate) struct KbCreate {
    pub(crate) hub: Arc<KbHub>,
}

#[derive(Deserialize)]
pub(crate) struct KbCreateArgs {
    name: String,
    #[serde(default)]
    config: ScenarioConfig,
}

impl KbCreate {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: KbCreateArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = self
            .hub
            .create_kb(a.name, a.config)
            .await
            .map_err(map_err)?;
        Ok(json!({ "kb_id": kb.id().as_str() }))
    }
}

pub(crate) struct KbOpen {
    pub(crate) hub: Arc<KbHub>,
}

#[derive(Deserialize)]
pub(crate) struct KbOpenArgs {
    kb_id: String,
}

impl KbOpen {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: KbOpenArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let id = parse_kb_id(&a.kb_id)?;
        let kb = self.hub.open_kb(&id).await.map_err(map_err)?;
        let caps = kb.capabilities();
        Ok(json!({
            "kb_id": kb.id().as_str(),
            "capabilities": caps,
            "config": kb.scenario_config().await
        }))
    }
}

pub(crate) struct KbDrop {
    pub(crate) hub: Arc<KbHub>,
}

#[derive(Deserialize)]
pub(crate) struct KbDropArgs {
    kb_id: String,
    confirm: bool,
}

impl KbDrop {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: KbDropArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        if !a.confirm {
            return Err(Error::invalid_args("confirm 必须true"));
        }
        let id = parse_kb_id(&a.kb_id)?;
        self.hub.drop_kb(&id).await.map_err(map_err)?;
        Ok(json!({ "ok": true }))
    }
}

pub(crate) struct ScopeCreate {
    pub(crate) hub: Arc<KbHub>,
}

#[derive(Deserialize)]
pub(crate) struct ScopeCreateArgs {
    kb_id: String,
    name: String,
    parent: Option<String>,
}

impl ScopeCreate {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: ScopeCreateArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = self
            .hub
            .get(&parse_kb_id(&a.kb_id)?)
            .await
            .map_err(map_err)?;
        let parent = match a.parent {
            Some(p) => Some(ScopeId::new(p).map_err(|e| Error::invalid_args(e.to_string()))?),
            None => None,
        };
        let sid = kb
            .create_scope(&a.name, parent.as_ref())
            .await
            .map_err(map_err)?;
        Ok(json!({ "scope_id": sid.as_str() }))
    }
}

pub(crate) struct ScopeBindSource {
    pub(crate) hub: Arc<KbHub>,
}

#[derive(Deserialize)]
pub(crate) struct BindArgs {
    kb_id: String,
    scope_id: String,
    source_type: String,
    source_id: String,
}

impl ScopeBindSource {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: BindArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = self
            .hub
            .get(&parse_kb_id(&a.kb_id)?)
            .await
            .map_err(map_err)?;
        let scope = ScopeId::new(a.scope_id).map_err(|e| Error::invalid_args(e.to_string()))?;
        kb.bind_source(
            &scope,
            SourceRef {
                source_type: a.source_type,
                source_id: a.source_id,
            },
        )
        .await
        .map_err(map_err)?;
        Ok(json!({ "ok": true }))
    }
}

pub(crate) struct ScopeList {
    pub(crate) hub: Arc<KbHub>,
}

#[derive(Deserialize)]
pub(crate) struct ListArgs {
    kb_id: String,
}

impl ScopeList {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: ListArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = self
            .hub
            .get(&parse_kb_id(&a.kb_id)?)
            .await
            .map_err(map_err)?;
        let scopes = kb.list_scopes().await.map_err(map_err)?;
        let list: Vec<_> = scopes
            .into_iter()
            .map(|(id, name)| json!({ "scope_id": id.as_str(), "name": name }))
            .collect();
        Ok(json!({ "scopes": list }))
    }
}