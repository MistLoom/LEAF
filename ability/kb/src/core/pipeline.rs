use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{Clue, KbError, MiuDraft, MiuMetadata, SourceRef, SymbolForm};

/// 语义单元：解构产物，尚未构建为草案。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticUnit {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
    #[serde(default)]
    pub ordinal: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum GateDecision {
    Pass { score: f64 },
    Reject { score: f64, reason: String },
}

#[async_trait]
pub trait Decompose: Send + Sync {
    async fn decompose(&self, clue: &Clue) -> Result<Vec<SemanticUnit>, KbError>;
}

#[async_trait]
pub trait Gate: Send + Sync {
    async fn evaluate(&self, unit: &SemanticUnit, threshold: f64) -> Result<GateDecision, KbError>;
}

#[async_trait]
pub trait Build: Send + Sync {
    async fn build(&self, units: &[SemanticUnit]) -> Result<Vec<MiuDraft>, KbError>;
}

/// 按空行与 Markdown 标题切分（无 LLM 的本地默认策略）。
pub struct TextBoundaryDecompose;

#[async_trait]
impl Decompose for TextBoundaryDecompose {
    async fn decompose(&self, clue: &Clue) -> Result<Vec<SemanticUnit>, KbError> {
        let source = clue.source();
        let text = clue.text_content();
        if text.trim().is_empty() {
            return Err(KbError::InvalidInput("线索文本为空".into()));
        }

        let mut units = Vec::new();
        let mut buf = String::new();
        let mut section = String::from("root");
        let mut ordinal = 0usize;

        let flush = |buf: &mut String,
                     section: &str,
                     ordinal: &mut usize,
                     source: &Option<SourceRef>,
                     units: &mut Vec<SemanticUnit>| {
            let trimmed = buf.trim();
            if trimmed.is_empty() {
                buf.clear();
                return;
            }
            units.push(SemanticUnit {
                text: trimmed.to_string(),
                doc_path: Some(section.to_string()),
                source: source.clone(),
                ordinal: *ordinal,
            });
            *ordinal += 1;
            buf.clear();
        };

        for line in text.lines() {
            let heading = parse_heading(line);
            if let Some(title) = heading {
                flush(&mut buf, &section, &mut ordinal, &source, &mut units);
                section = title;
                continue;
            }
            if line.trim().is_empty() {
                flush(&mut buf, &section, &mut ordinal, &source, &mut units);
                continue;
            }
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(line);
        }
        flush(&mut buf, &section, &mut ordinal, &source, &mut units);

        if units.is_empty() {
            return Err(KbError::InvalidInput("解构结果为空".into()));
        }
        Ok(units)
    }
}

fn parse_heading(line: &str) -> Option<String> {
    let t = line.trim();
    if !t.starts_with('#') {
        return None;
    }
    let title = t.trim_start_matches('#').trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

/// 基于长度的简单门控。
pub struct ScoreGate;

#[async_trait]
impl Gate for ScoreGate {
    async fn evaluate(&self, unit: &SemanticUnit, threshold: f64) -> Result<GateDecision, KbError> {
        let len = unit.text.chars().count() as f64;
        // 过短降权：< 8 字视为低分
        let score = (len / 64.0).clamp(0.0, 1.0);
        if score < threshold {
            Ok(GateDecision::Reject {
                score,
                reason: format!("score {score} < threshold {threshold}"),
            })
        } else {
            Ok(GateDecision::Pass { score })
        }
    }
}

/// 将语义单元转为草案，并填充上下文链占位（由调用方或 link_context 固化 id）。
pub struct PassthroughBuild;

#[async_trait]
impl Build for PassthroughBuild {
    async fn build(&self, units: &[SemanticUnit]) -> Result<Vec<MiuDraft>, KbError> {
        if units.is_empty() {
            return Err(KbError::InvalidInput("语义单元为空".into()));
        }
        let mut drafts = Vec::with_capacity(units.len());
        for unit in units {
            let proposition = unit
                .text
                .lines()
                .next()
                .unwrap_or(unit.text.as_str())
                .chars()
                .take(200)
                .collect::<String>();
            drafts.push(MiuDraft {
                id: None,
                vector: None,
                symbol: SymbolForm {
                    proposition: Some(proposition),
                    entities: Vec::new(),
                    relations: Vec::new(),
                },
                metadata: MiuMetadata {
                    doc_path: unit.doc_path.clone(),
                    prev_id: None,
                    next_id: None,
                    modality: Some("text".into()),
                    scopes: Vec::new(),
                    source: unit.source.clone(),
                    fingerprint: Some(fingerprint(&unit.text)),
                    extra: None,
                },
                raw_summary: unit.text.clone(),
                score: None,
            });
        }
        Ok(drafts)
    }
}

fn fingerprint(text: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let normalized: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut hasher = DefaultHasher::new();
    normalized.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
