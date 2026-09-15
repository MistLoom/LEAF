use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 摄取来源引用，支撑按源增量定位。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub source_type: String,
    pub source_id: String,
}

/// 文本线索。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextClue {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
    /// 粒度等提示，不替代解构结果。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<Value>,
}

/// JSON 线索（已部分结构化）。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonClue {
    pub payload: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// 对齐摄取阶段的原始记录。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawRecord {
    pub id: String,
    pub source_type: String,
    pub timestamp: u64,
    pub raw_content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_metadata: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Clue {
    Text(TextClue),
    Json(JsonClue),
    Raw(RawRecord),
}

impl Clue {
    pub fn text_content(&self) -> String {
        match self {
            Clue::Text(t) => t.text.clone(),
            Clue::Json(j) => j.payload.to_string(),
            Clue::Raw(r) => r.raw_content.clone(),
        }
    }

    pub fn source(&self) -> Option<SourceRef> {
        match self {
            Clue::Text(t) => t.source.clone(),
            Clue::Json(j) => j.source.clone(),
            Clue::Raw(r) => Some(SourceRef {
                source_type: r.source_type.clone(),
                source_id: r.id.clone(),
            }),
        }
    }
}
