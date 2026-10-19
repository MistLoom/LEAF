use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{MiuId, ScopeId, SourceRef};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SymbolForm {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposition: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MiuMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_id: Option<MiuId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_id: Option<MiuId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<ScopeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
    /// 内容指纹，用于重复估计与增量定位。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MiuStatus {
    Active,
    Stale,
}

impl Default for MiuStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// 写出前草案。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MiuDraft {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<MiuId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vec<f32>>,
    #[serde(default)]
    pub symbol: SymbolForm,
    #[serde(default)]
    pub metadata: MiuMetadata,
    pub raw_summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

/// 已落库 MIU。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Miu {
    pub id: MiuId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vec<f32>>,
    pub symbol: SymbolForm,
    pub metadata: MiuMetadata,
    pub raw_summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    pub status: MiuStatus,
    pub created_at: u64,
    pub updated_at: u64,
}
