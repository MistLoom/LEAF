//! 多方法分发

pub(crate) mod assess;
pub(crate) mod clue;
pub(crate) mod hook;
pub(crate) mod miu;
pub(crate) mod retrieve;
pub(crate) mod scope;
pub(crate) mod stats;

use std::sync::Arc;

use ability_support::Error;
use serde_json::{json, Value};

use crate::core::{PassthroughBuild, ScoreGate, TextBoundaryDecompose};
use crate::store::KbHub;

pub(crate) struct KbMethods {
    hub: Arc<KbHub>,
    kb_create: scope::KbCreate,
    kb_open: scope::KbOpen,
    kb_drop: scope::KbDrop,
    scope_create: scope::ScopeCreate,
    bind_source: scope::ScopeBindSource,
    scope_list: scope::ScopeList,
    decompose: clue::DecomposeTool,
    gate: clue::GateTool,
    build: clue::BuildTool,
    miu_get: miu::GetTool,
    miu_put: miu::PutTool,
    miu_put_many: miu::PutManyTool,
    miu_patch: miu::PatchTool,
    miu_stale: miu::MarkStaleTool,
    miu_delete: miu::DeleteTool,
    miu_link: miu::LinkTool,
    by_id: retrieve::ById,
    lexical: retrieve::Lexical,
    symbol: retrieve::Symbol,
    vector: retrieve::Vector,
    hybrid: retrieve::Hybrid,
    expand: retrieve::Expand,
    related: retrieve::Related,
    multi: retrieve::MultiSource,
    dim: stats::DimTool,
    dup: stats::DupTool,
    freshness: stats::FreshnessTool,
    coverage: stats::CoverageTool,
    subscribe: hook::Subscribe,
    unsubscribe: hook::Unsubscribe,
    hook_list: hook::List,
    verity: assess::VerityTool,
}

impl KbMethods {
    pub fn new(hub: Arc<KbHub>) -> Self {
        let multi = retrieve::MultiSource { hub: hub.clone() };
        Self {
            kb_create: scope::KbCreate { hub: hub.clone() },
            kb_open: scope::KbOpen { hub: hub.clone() },
            kb_drop: scope::KbDrop { hub: hub.clone() },
            scope_create: scope::ScopeCreate { hub: hub.clone() },
            bind_source: scope::ScopeBindSource { hub: hub.clone() },
            scope_list: scope::ScopeList { hub: hub.clone() },
            decompose: clue::DecomposeTool {
                engine: TextBoundaryDecompose,
            },
            gate: clue::GateTool {
                hub: hub.clone(),
                engine: ScoreGate,
            },
            build: clue::BuildTool {
                engine: PassthroughBuild,
            },
            miu_get: miu::GetTool { hub: hub.clone() },
            miu_put: miu::PutTool { hub: hub.clone() },
            miu_put_many: miu::PutManyTool { hub: hub.clone() },
            miu_patch: miu::PatchTool { hub: hub.clone() },
            miu_stale: miu::MarkStaleTool { hub: hub.clone() },
            miu_delete: miu::DeleteTool { hub: hub.clone() },
            miu_link: miu::LinkTool { hub: hub.clone() },
            by_id: retrieve::ById { hub: hub.clone() },
            lexical: retrieve::Lexical { hub: hub.clone() },
            symbol: retrieve::Symbol { hub: hub.clone() },
            vector: retrieve::Vector { hub: hub.clone() },
            hybrid: retrieve::Hybrid { hub: hub.clone() },
            expand: retrieve::Expand { hub: hub.clone() },
            related: retrieve::Related { hub: hub.clone() },
            multi: multi.clone(),
            dim: stats::DimTool { hub: hub.clone() },
            dup: stats::DupTool { hub: hub.clone() },
            freshness: stats::FreshnessTool { hub: hub.clone() },
            coverage: stats::CoverageTool { hub: hub.clone() },
            subscribe: hook::Subscribe { hub: hub.clone() },
            unsubscribe: hook::Unsubscribe { hub: hub.clone() },
            hook_list: hook::List { hub: hub.clone() },
            verity: assess::VerityTool { multi },
            hub,
        }
    }

    pub async fn invoke(&self, method: &str, args: Value) -> Result<Value, Error> {
        match method {
            "scope.kb_create" => self.kb_create.invoke(args).await,
            "scope.kb_open" => self.kb_open.invoke(args).await,
            "scope.kb_drop" => self.kb_drop.invoke(args).await,
            "scope.create" => self.scope_create.invoke(args).await,
            "scope.bind_source" => self.bind_source.invoke(args).await,
            "scope.list" => self.scope_list.invoke(args).await,
            "clue.decompose" => self.decompose.invoke(args).await,
            "clue.gate" => self.gate.invoke(args).await,
            "clue.build" => self.build.invoke(args).await,
            "miu.get" => self.miu_get.invoke(args).await,
            "miu.put" => self.miu_put.invoke(args).await,
            "miu.put_many" => self.miu_put_many.invoke(args).await,
            "miu.patch" => self.miu_patch.invoke(args).await,
            "miu.mark_stale" => self.miu_stale.invoke(args).await,
            "miu.delete" => self.miu_delete.invoke(args).await,
            "miu.link_context" => self.miu_link.invoke(args).await,
            "retrieve.by_id" => self.by_id.invoke(args).await,
            "retrieve.lexical" => self.lexical.invoke(args).await,
            "retrieve.symbol" => self.symbol.invoke(args).await,
            "retrieve.vector" => self.vector.invoke(args).await,
            "retrieve.hybrid" => self.hybrid.invoke(args).await,
            "retrieve.expand_context" => self.expand.invoke(args).await,
            "retrieve.related" => self.related.invoke(args).await,
            "retrieve.multi_source" => self.multi.invoke(args).await,
            "stats.dim" => self.dim.invoke(args).await,
            "stats.dup_prob" => self.dup.invoke(args).await,
            "stats.freshness" => self.freshness.invoke(args).await,
            "stats.coverage" => self.coverage.invoke(args).await,
            "hook.subscribe" => self.subscribe.invoke(args).await,
            "hook.unsubscribe" => self.unsubscribe.invoke(args).await,
            "hook.list" => self.hook_list.invoke(args).await,
            "hook.deliveries" => {
                let d = self.hub.outbound().deliveries().await;
                Ok(json!({ "deliveries": d }))
            }
            "assess.verity" => self.verity.invoke(args).await,
            other => Err(Error::not_found(other)),
        }
    }
}
