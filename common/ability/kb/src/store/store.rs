use async_trait::async_trait;
use crate::core::{
    KbError, KbId, Miu, MiuDraft, MiuId, ScopeId, SourceRef, ScenarioConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KbCapabilities {
    pub lexical: bool,
    pub vector: bool,
    pub symbol: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchHit {
    pub miu: Miu,
    pub score: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DimQuery {
    pub dimension: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_ids: Option<Vec<ScopeId>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DimBucket {
    pub key: String,
    pub count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_score: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DimStat {
    pub dimension: String,
    pub buckets: Vec<DimBucket>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DupEstimate {
    pub p_dup: f64,
    pub evidence: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nearest_miu_id: Option<MiuId>,
}

#[async_trait]
pub trait KbStore: Send + Sync {
    fn id(&self) -> &KbId;
    fn capabilities(&self) -> KbCapabilities;
    async fn scenario_config(&self) -> ScenarioConfig;

    async fn create_scope(
        &self,
        name: &str,
        parent: Option<&ScopeId>,
    ) -> Result<ScopeId, KbError>;

    async fn list_scopes(&self) -> Result<Vec<(ScopeId, String)>, KbError>;

    async fn bind_source(&self, scope_id: &ScopeId, source: SourceRef) -> Result<(), KbError>;

    async fn put_miu(&self, draft: MiuDraft) -> Result<Miu, KbError>;

    async fn put_many(&self, drafts: Vec<MiuDraft>) -> Result<Vec<Miu>, KbError>;

    async fn get_miu(&self, id: &MiuId) -> Result<Miu, KbError>;

    async fn patch_miu(&self, id: &MiuId, patch: serde_json::Value) -> Result<Miu, KbError>;

    async fn mark_stale(&self, id: &MiuId) -> Result<Miu, KbError>;

    async fn delete_miu(&self, id: &MiuId) -> Result<(), KbError>;

    async fn link_context(
        &self,
        id: &MiuId,
        prev: Option<MiuId>,
        next: Option<MiuId>,
    ) -> Result<Miu, KbError>;

    async fn search_lexical(
        &self,
        query: &str,
        scope_ids: Option<&[ScopeId]>,
        limit: usize,
    ) -> Result<Vec<SearchHit>, KbError>;

    async fn search_symbol(
        &self,
        query: &str,
        scope_ids: Option<&[ScopeId]>,
        limit: usize,
    ) -> Result<Vec<SearchHit>, KbError>;

    async fn search_vector(
        &self,
        _vector: &[f32],
        _scope_ids: Option<&[ScopeId]>,
        _limit: usize,
    ) -> Result<Vec<SearchHit>, KbError> {
        Err(KbError::CapabilityUnavailable("vector".into()))
    }

    async fn stats_dim(&self, query: DimQuery) -> Result<DimStat, KbError>;

    async fn dup_prob(
        &self,
        text: &str,
        fingerprint: Option<&str>,
        scope_ids: Option<&[ScopeId]>,
    ) -> Result<DupEstimate, KbError>;
}
