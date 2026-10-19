//! 线索管道前半段：decompose / gate / build，均不写库

use std::sync::Arc;

use crate::core::{
    Build, Clue, Decompose, Gate, KbError, PassthroughBuild, ScoreGate, SemanticUnit,
    TextBoundaryDecompose,
};
use crate::store::KbHub;
use ability_support::Error;
use serde::Deserialize;
use serde_json::{json, Value};

fn map_err(e: KbError) -> Error {
    Error::failed(e.to_string())
}

pub(crate) struct DecomposeTool {
    pub(crate) engine: TextBoundaryDecompose,
}

#[derive(Deserialize)]
pub(crate) struct DecomposeArgs {
    clue: Clue,
}

impl DecomposeTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: DecomposeArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let units = self.engine.decompose(&a.clue).await.map_err(map_err)?;
        Ok(json!({ "units": units }))
    }
}

pub(crate) struct GateTool {
    pub(crate) hub: Arc<KbHub>,
    pub(crate) engine: ScoreGate,
}

#[derive(Deserialize)]
pub(crate) struct GateArgs {
    unit: SemanticUnit,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kb_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    threshold: Option<f64>,
}

impl GateTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: GateArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let threshold = if let Some(t) = a.threshold {
            t
        } else if let Some(kid) = a.kb_id {
            let id = crate::core::KbId::new(kid).map_err(|e| Error::invalid_args(e.to_string()))?;
            let kb = self.hub.get(&id).await.map_err(map_err)?;
            kb.scenario_config().await.gate_threshold
        } else {
            0.0
        };
        let decision = self
            .engine
            .evaluate(&a.unit, threshold)
            .await
            .map_err(map_err)?;
        serde_json::to_value(decision).map_err(|e| Error::failed(e.to_string()))
    }
}

pub(crate) struct BuildTool {
    pub(crate) engine: PassthroughBuild,
}

#[derive(Deserialize)]
pub(crate) struct BuildArgs {
    units: Vec<SemanticUnit>,
    #[serde(default)]
    scope_ids: Vec<String>,
}

impl BuildTool {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: BuildArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let mut drafts = self.engine.build(&a.units).await.map_err(map_err)?;
        if !a.scope_ids.is_empty() {
            let scopes: Result<Vec<_>, _> = a
                .scope_ids
                .iter()
                .map(|s| crate::core::ScopeId::new(s.clone()))
                .collect();
            let scopes = scopes.map_err(|e| Error::invalid_args(e.to_string()))?;
            for d in &mut drafts {
                d.metadata.scopes = scopes.clone();
            }
        }
        Ok(json!({ "drafts": drafts }))
    }
}
