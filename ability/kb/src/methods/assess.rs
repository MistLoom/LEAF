//! 真实度评估：只消费多源检索证据，不做入库与生成

use std::sync::Arc;

use crate::methods::retrieve::MultiSource;
use ability_support::Error;
use serde::Deserialize;
use serde_json::{json, Value};

/// 对给定文本做真实度评估；crate 内调retrieve.multi_source
pub(crate) struct VerityTool {
    pub(crate) multi: MultiSource,
}

#[derive(Deserialize)]
pub(crate) struct VerityArgs {
    kb_id: String,
    text: String,
    #[serde(default)]
    scope_ids: Vec<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    20
}

impl VerityTool {
    pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: VerityArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        if a.text.trim().is_empty() {
            return Err(Error::invalid_args("text 为空"));
        }

        let multi = self
            .multi
            .invoke(json!({
                "kb_id": a.kb_id,
                "text": a.text,
                "scope_ids": a.scope_ids,
                "limit": a.limit
            }))
            .await?;

        let source_count = multi
            .get("source_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as f64;
        let hits = multi
            .get("hits")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut overlaps = Vec::new();
        let mut scores = Vec::new();
        let mut evidence = Vec::new();
        for h in hits.iter().take(a.limit) {
            let summary = h
                .pointer("/miu/raw_summary")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let score = h.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let j = token_jaccard(&a.text, summary);
            overlaps.push(j);
            scores.push(score);
            evidence.push(json!({
                "miu_id": h.pointer("/miu/id"),
                "source": h.pointer("/miu/metadata/source"),
                "overlap": j,
                "score": score,
                "snippet": summary.chars().take(120).collect::<String>()
            }));
        }

        let mean_overlap = mean(&overlaps);
        let mean_score = mean(&scores);
        // 源多样性：2+ 源给满，单源减半
        let diversity = if source_count <= 0.0 {
            0.0
        } else if source_count >= 2.0 {
            1.0
        } else {
            0.45
        };

        let verity = (0.4 * diversity + 0.4 * mean_overlap + 0.2 * mean_score.clamp(0.0, 1.0))
            .clamp(0.0, 1.0);

        let stance = if hits.is_empty() {
            "insufficient"
        } else if verity >= 0.65 && source_count >= 2.0 {
            "supported"
        } else if verity >= 0.4 {
            "weak"
        } else {
            "insufficient"
        };

        Ok(json!({
            "text": a.text,
            "verity": verity,
            "stance": stance,
            "source_count": source_count as u64,
            "hit_count": hits.len(),
            "mean_overlap": mean_overlap,
            "mean_score": mean_score,
            "evidence": evidence,
            "note": "工程启发式估计，非形式化真值证明。",
        }))
    }
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

fn token_jaccard(a: &str, b: &str) -> f64 {
    let ta: std::collections::HashSet<_> = a.split_whitespace().collect();
    let tb: std::collections::HashSet<_> = b.split_whitespace().collect();
    if ta.is_empty() || tb.is_empty() {
        return 0.0;
    }
    let inter = ta.intersection(&tb).count() as f64;
    let union = ta.union(&tb).count() as f64;
    if union > 0.0 {
        inter / union
    } else {
        0.0
    }
}
