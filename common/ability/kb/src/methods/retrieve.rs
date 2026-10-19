//! 检索原语：不含问答生成

use std::sync::Arc;

use crate::core::{KbError, KbId, MiuId, ScopeId};
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

fn scopes(v: &[String]) -> Result<Vec<ScopeId>, Error> {
    v.iter()
        .map(|s| ScopeId::new(s.clone()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| Error::invalid_args(e.to_string()))
}

#[derive(Deserialize)]
pub(crate) struct ByIdArgs {
    kb_id: String,
    miu_id: String,
}

pub(crate) struct ById {
    pub(crate) hub: Arc<KbHub>,
}
impl ById {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: ByIdArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let mid = MiuId::new(a.miu_id).map_err(|e| Error::invalid_args(e.to_string()))?;
        let miu = kb.get_miu(&mid).await.map_err(map_err)?;
        Ok(json!({ "miu": miu }))
    }
}

#[derive(Deserialize)]
pub(crate) struct QueryArgs {
    kb_id: String,
    query: String,
    #[serde(default)]
    scope_ids: Vec<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize {
    10
}

pub(crate) struct Lexical {
    pub(crate) hub: Arc<KbHub>,
}
impl Lexical {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: QueryArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let sc = scopes(&a.scope_ids)?;
        let hits = kb
            .search_lexical(&a.query, Some(sc.as_slice()), a.limit)
            .await
            .map_err(map_err)?;
        Ok(json!({ "hits": hits }))
    }
}

pub(crate) struct Symbol {
    pub(crate) hub: Arc<KbHub>,
}
impl Symbol {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: QueryArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let sc = scopes(&a.scope_ids)?;
        let hits = kb
            .search_symbol(&a.query, Some(sc.as_slice()), a.limit)
            .await
            .map_err(map_err)?;
        Ok(json!({ "hits": hits }))
    }
}

pub(crate) struct Vector {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct VectorArgs {
    kb_id: String,
    vector: Vec<f32>,
    #[serde(default)]
    scope_ids: Vec<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}
impl Vector {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: VectorArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let sc = scopes(&a.scope_ids)?;
        let hits = kb
            .search_vector(&a.vector, Some(sc.as_slice()), a.limit)
            .await
            .map_err(map_err)?;
        Ok(json!({ "hits": hits }))
    }
}

pub(crate) struct Hybrid {
    pub(crate) hub: Arc<KbHub>,
}
impl Hybrid {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: QueryArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let sc = scopes(&a.scope_ids)?;
        let mut lexical = kb
            .search_lexical(&a.query, Some(sc.as_slice()), a.limit)
            .await
            .map_err(map_err)?;
        let symbol = kb
            .search_symbol(&a.query, Some(sc.as_slice()), a.limit)
            .await
            .map_err(map_err)?;
        // 符号命中抬分后与词法合并去重
        for h in symbol {
            if let Some(existing) = lexical.iter_mut().find(|x| x.miu.id == h.miu.id) {
                existing.score = existing.score.max(h.score + 0.5);
            } else {
                lexical.push(h);
            }
        }
        lexical.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        lexical.truncate(a.limit.max(1));
        Ok(json!({ "hits": lexical }))
    }
}

pub(crate) struct Expand {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct ExpandArgs {
    kb_id: String,
    miu_id: String,
}
impl Expand {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: ExpandArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let kb = store(&self.hub, &a.kb_id).await?;
        let mid = MiuId::new(a.miu_id).map_err(|e| Error::invalid_args(e.to_string()))?;
        let miu = kb.get_miu(&mid).await.map_err(map_err)?;
        let mut chain = vec![miu.clone()];
        if let Some(prev) = &miu.metadata.prev_id {
            if let Ok(p) = kb.get_miu(prev).await {
                chain.insert(0, p);
            }
        }
        if let Some(next) = &miu.metadata.next_id {
            if let Ok(n) = kb.get_miu(next).await {
                chain.push(n);
            }
        }
        Ok(json!({ "chain": chain }))
    }
}

/// 以文本查询相关结果集（词符号融合去重）
pub(crate) struct Related {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct TextQueryArgs {
    kb_id: String,
    text: String,
    #[serde(default)]
    scope_ids: Vec<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}
impl Related {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: TextQueryArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        if a.text.trim().is_empty() {
            return Err(Error::invalid_args("text 为空"));
        }
        let kb = store(&self.hub, &a.kb_id).await?;
        let sc = scopes(&a.scope_ids)?;
        let hits = merge_channels(&kb, &a.text, &sc, a.limit).await?;
        Ok(json!({
            "text": a.text,
            "hits": hits,
            "count": hits.len()
        }))
    }
}

/// 多源检索：source_type/source_id 分组返回
#[derive(Clone)]
pub(crate) struct MultiSource {
    pub(crate) hub: Arc<KbHub>,
}
impl MultiSource {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: TextQueryArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        if a.text.trim().is_empty() {
            return Err(Error::invalid_args("text 为空"));
        }
        let kb = store(&self.hub, &a.kb_id).await?;
        let sc = scopes(&a.scope_ids)?;
        let hits = merge_channels(&kb, &a.text, &sc, a.limit.max(20)).await?;

        use std::collections::BTreeMap;
        let mut by_source: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        for h in &hits {
            let key = match &h.miu.metadata.source {
                Some(s) => format!("{}:{}", s.source_type, s.source_id),
                None => "_unknown".into(),
            };
            by_source
                .entry(key)
                .or_default()
                .push(serde_json::to_value(h).map_err(|e| Error::failed(e.to_string()))?);
        }
        let source_count = by_source.len();
        Ok(json!({
            "text": a.text,
            "source_count": source_count,
            "by_source": by_source,
            "hits": hits
        }))
    }
}

async fn merge_channels(
    kb: &Arc<dyn crate::store::KbStore>,
    text: &str,
    sc: &[ScopeId],
    limit: usize,
) -> Result<Vec<crate::store::SearchHit>, Error> {
    let mut lexical: Vec<crate::store::SearchHit> = Vec::new();
    for term in query_terms(text) {
        let hits = kb
            .search_lexical(&term, Some(sc), limit)
            .await
            .map_err(map_err)?;
        merge_hits(&mut lexical, hits, 0.0);
        let symbol = kb
            .search_symbol(&term, Some(sc), limit)
            .await
            .map_err(map_err)?;
        merge_hits(&mut lexical, symbol, 0.5);
    }
    for h in &mut lexical {
        let j = token_jaccard(text, &h.miu.raw_summary);
        let cjk = cjk_overlap(text, &h.miu.raw_summary);
        h.score = 0.4 * h.score + 0.3 * j + 0.3 * cjk;
    }
    lexical.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    lexical.truncate(limit.max(1));
    Ok(lexical)
}

fn merge_hits(
    into: &mut Vec<crate::store::SearchHit>,
    from: Vec<crate::store::SearchHit>,
    symbol_boost: f64,
) {
    for h in from {
        if let Some(existing) = into.iter_mut().find(|x| x.miu.id == h.miu.id) {
            existing.score = existing.score.max(h.score + symbol_boost);
        } else {
            into.push(crate::store::SearchHit {
                score: h.score + symbol_boost,
                miu: h.miu,
            });
        }
    }
}

fn query_terms(text: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let push = |terms: &mut Vec<String>, seen: &mut std::collections::HashSet<String>, t: String| {
        if t.chars().count() < 2 {
            return;
        }
        if seen.insert(t.clone()) {
            terms.push(t);
        }
    };

    for t in text.split_whitespace() {
        push(&mut terms, &mut seen, t.to_string());
    }
    for t in text.split(|c: char| !c.is_ascii_alphanumeric()) {
        if !t.is_empty() {
            push(&mut terms, &mut seen, t.to_string());
        }
    }
    // 中文 bigram，覆盖无空格文本
    let cjk: Vec<char> = text.chars().filter(|c| is_cjk(*c)).collect();
    for w in cjk.windows(2) {
        push(&mut terms, &mut seen, format!("{}{}", w[0], w[1]));
    }
    if terms.is_empty() {
        push(
            &mut terms,
            &mut seen,
            text.chars().take(64).collect::<String>(),
        );
    }
    terms.into_iter().take(24).collect()
}

fn is_cjk(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c)
}

fn cjk_overlap(a: &str, b: &str) -> f64 {
    let ga: std::collections::HashSet<String> = a
        .chars()
        .filter(|c| is_cjk(*c))
        .collect::<Vec<_>>()
        .windows(2)
        .map(|w| format!("{}{}", w[0], w[1]))
        .collect();
    let gb: std::collections::HashSet<String> = b
        .chars()
        .filter(|c| is_cjk(*c))
        .collect::<Vec<_>>()
        .windows(2)
        .map(|w| format!("{}{}", w[0], w[1]))
        .collect();
    if ga.is_empty() || gb.is_empty() {
        return 0.0;
    }
    let inter = ga.intersection(&gb).count() as f64;
    let union = ga.union(&gb).count() as f64;
    if union > 0.0 {
        inter / union
    } else {
        0.0
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
