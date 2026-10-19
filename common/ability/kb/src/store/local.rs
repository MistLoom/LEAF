use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use crate::core::{
    KbError, KbId, Miu, MiuDraft, MiuId, MiuStatus, ScopeId, SourceRef, ScenarioConfig,
};
use crate::outbound::{Notifier, OutboundEvent};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::store::io::StoreIo;
use crate::store::{
    DimBucket, DimQuery, DimStat, DupEstimate, KbCapabilities, KbStore, SearchHit,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ScopeRecord {
    id: ScopeId,
    name: String,
    parent: Option<ScopeId>,
    sources: Vec<SourceRef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct KbMeta {
    id: KbId,
    name: String,
    config: ScenarioConfig,
}

struct Inner {
    meta: KbMeta,
    scopes: HashMap<ScopeId, ScopeRecord>,
    mius: HashMap<MiuId, Miu>,
}

/// 经 persist 落盘的知识库。
pub struct LocalKb {
    io: StoreIo,
    kb_id: KbId,
    inner: RwLock<Inner>,
    notifier: Arc<dyn Notifier>,
}

impl LocalKb {
    pub async fn create(
        io: StoreIo,
        name: impl Into<String>,
        config: ScenarioConfig,
        notifier: Arc<dyn Notifier>,
    ) -> Result<Self, KbError> {
        let id = KbId::generate();
        let meta = KbMeta {
            id: id.clone(),
            name: name.into(),
            config,
        };
        let kb = Self {
            io,
            kb_id: id,
            inner: RwLock::new(Inner {
                meta,
                scopes: HashMap::new(),
                mius: HashMap::new(),
            }),
            notifier,
        };
        kb.persist().await?;
        Ok(kb)
    }

    pub async fn open(
        io: StoreIo,
        kb_id: &KbId,
        notifier: Arc<dyn Notifier>,
    ) -> Result<Self, KbError> {
        let prefix = kb_id.as_str();
        let meta: KbMeta = io
            .get_json(&format!("{prefix}/meta.json"))
            .await?
            .ok_or_else(|| KbError::KbNotFound(kb_id.to_string()))?;
        let scopes: HashMap<ScopeId, ScopeRecord> = io
            .get_json(&format!("{prefix}/scopes.json"))
            .await?
            .unwrap_or_default();
        let mius: HashMap<MiuId, Miu> = io
            .get_json(&format!("{prefix}/mius.json"))
            .await?
            .unwrap_or_default();
        Ok(Self {
            io,
            kb_id: meta.id.clone(),
            inner: RwLock::new(Inner { meta, scopes, mius }),
            notifier,
        })
    }

    pub async fn name(&self) -> String {
        self.inner.read().await.meta.name.clone()
    }

    async fn persist(&self) -> Result<(), KbError> {
        let guard = self.inner.read().await;
        let prefix = self.kb_id.as_str();
        self.io
            .put_json(&format!("{prefix}/meta.json"), &guard.meta)
            .await?;
        self.io
            .put_json(&format!("{prefix}/scopes.json"), &guard.scopes)
            .await?;
        self.io
            .put_json(&format!("{prefix}/mius.json"), &guard.mius)
            .await?;
        Ok(())
    }

    async fn emit(&self, topic: &str, payload: serde_json::Value) -> Result<(), KbError> {
        let event = OutboundEvent {
            topic: topic.into(),
            payload,
            trace_id: Some(Uuid::new_v4().to_string()),
            ts: now_secs(),
        };
        self.notifier
            .emit(event)
            .await
            .map_err(|e| KbError::Store(e.to_string()))
    }

    fn in_scopes(miu: &Miu, scope_ids: Option<&[ScopeId]>) -> bool {
        match scope_ids {
            None | Some([]) => true,
            Some(ids) => ids.iter().any(|s| miu.metadata.scopes.contains(s)),
        }
    }
}

#[async_trait]
impl KbStore for LocalKb {
    fn id(&self) -> &KbId {
        &self.kb_id
    }

    fn capabilities(&self) -> KbCapabilities {
        KbCapabilities {
            lexical: true,
            vector: false,
            symbol: true,
        }
    }

    async fn scenario_config(&self) -> ScenarioConfig {
        self.inner.read().await.meta.config.clone()
    }

    async fn create_scope(
        &self,
        name: &str,
        parent: Option<&ScopeId>,
    ) -> Result<ScopeId, KbError> {
        let mut guard = self.inner.write().await;
        if let Some(p) = parent {
            if !guard.scopes.contains_key(p) {
                return Err(KbError::ScopeNotFound(p.to_string()));
            }
        }
        let id = ScopeId::generate();
        guard.scopes.insert(
            id.clone(),
            ScopeRecord {
                id: id.clone(),
                name: name.into(),
                parent: parent.cloned(),
                sources: Vec::new(),
            },
        );
        drop(guard);
        self.persist().await?;
        Ok(id)
    }

    async fn list_scopes(&self) -> Result<Vec<(ScopeId, String)>, KbError> {
        let guard = self.inner.read().await;
        Ok(guard
            .scopes
            .values()
            .map(|s| (s.id.clone(), s.name.clone()))
            .collect())
    }

    async fn bind_source(&self, scope_id: &ScopeId, source: SourceRef) -> Result<(), KbError> {
        let mut guard = self.inner.write().await;
        let scope = guard
            .scopes
            .get_mut(scope_id)
            .ok_or_else(|| KbError::ScopeNotFound(scope_id.to_string()))?;
        if !scope.sources.iter().any(|s| s == &source) {
            scope.sources.push(source);
        }
        drop(guard);
        self.persist().await
    }

    async fn put_miu(&self, draft: MiuDraft) -> Result<Miu, KbError> {
        let now = now_secs();
        let id = draft.id.clone().unwrap_or_else(MiuId::generate);
        let is_update = {
            let guard = self.inner.read().await;
            guard.mius.contains_key(&id)
        };
        let created_at = if is_update {
            self.inner
                .read()
                .await
                .mius
                .get(&id)
                .map(|m| m.created_at)
                .unwrap_or(now)
        } else {
            now
        };
        let miu = Miu {
            id: id.clone(),
            vector: draft.vector,
            symbol: draft.symbol,
            metadata: draft.metadata,
            raw_summary: draft.raw_summary,
            score: draft.score,
            status: MiuStatus::Active,
            created_at,
            updated_at: now,
        };
        {
            let mut guard = self.inner.write().await;
            guard.mius.insert(id, miu.clone());
        }
        self.persist().await?;
        let topic = if is_update {
            "kb.miu.updated"
        } else {
            "kb.miu.ingested"
        };
        let op = if is_update { "updated" } else { "ingested" };
        self.emit(
            topic,
            json!({
                "kb_id": self.kb_id.as_str(),
                "miu_id": miu.id.as_str(),
                "op": op
            }),
        )
        .await?;
        Ok(miu)
    }

    async fn put_many(&self, drafts: Vec<MiuDraft>) -> Result<Vec<Miu>, KbError> {
        let mut out = Vec::with_capacity(drafts.len());
        for d in drafts {
            out.push(self.put_miu(d).await?);
        }
        Ok(out)
    }

    async fn get_miu(&self, id: &MiuId) -> Result<Miu, KbError> {
        let guard = self.inner.read().await;
        guard
            .mius
            .get(id)
            .cloned()
            .ok_or_else(|| KbError::MiuNotFound(id.to_string()))
    }

    async fn patch_miu(&self, id: &MiuId, patch: serde_json::Value) -> Result<Miu, KbError> {
        let mut guard = self.inner.write().await;
        let miu = guard
            .mius
            .get_mut(id)
            .ok_or_else(|| KbError::MiuNotFound(id.to_string()))?;
        if let Some(s) = patch.get("raw_summary").and_then(|v| v.as_str()) {
            miu.raw_summary = s.to_string();
        }
        if let Some(prop) = patch
            .pointer("/symbol/proposition")
            .and_then(|v| v.as_str())
        {
            miu.symbol.proposition = Some(prop.to_string());
        }
        if let Some(scopes) = patch.get("scopes").and_then(|v| v.as_array()) {
            miu.metadata.scopes = scopes
                .iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| ScopeId::new(s).ok())
                .collect();
        }
        miu.updated_at = now_secs();
        let cloned = miu.clone();
        drop(guard);
        self.persist().await?;
        self.emit(
            "kb.miu.updated",
            json!({
                "kb_id": self.kb_id.as_str(),
                "miu_id": cloned.id.as_str(),
                "op": "updated"
            }),
        )
        .await?;
        Ok(cloned)
    }

    async fn mark_stale(&self, id: &MiuId) -> Result<Miu, KbError> {
        let mut guard = self.inner.write().await;
        let miu = guard
            .mius
            .get_mut(id)
            .ok_or_else(|| KbError::MiuNotFound(id.to_string()))?;
        miu.status = MiuStatus::Stale;
        miu.updated_at = now_secs();
        let cloned = miu.clone();
        drop(guard);
        self.persist().await?;
        Ok(cloned)
    }

    async fn delete_miu(&self, id: &MiuId) -> Result<(), KbError> {
        {
            let mut guard = self.inner.write().await;
            if guard.mius.remove(id).is_none() {
                return Err(KbError::MiuNotFound(id.to_string()));
            }
        }
        self.persist().await?;
        self.emit(
            "kb.miu.deleted",
            json!({
                "kb_id": self.kb_id.as_str(),
                "miu_id": id.as_str(),
                "op": "deleted"
            }),
        )
        .await?;
        Ok(())
    }

    async fn link_context(
        &self,
        id: &MiuId,
        prev: Option<MiuId>,
        next: Option<MiuId>,
    ) -> Result<Miu, KbError> {
        let mut guard = self.inner.write().await;
        let miu = guard
            .mius
            .get_mut(id)
            .ok_or_else(|| KbError::MiuNotFound(id.to_string()))?;
        miu.metadata.prev_id = prev;
        miu.metadata.next_id = next;
        miu.updated_at = now_secs();
        let cloned = miu.clone();
        drop(guard);
        self.persist().await?;
        Ok(cloned)
    }

    async fn search_lexical(
        &self,
        query: &str,
        scope_ids: Option<&[ScopeId]>,
        limit: usize,
    ) -> Result<Vec<SearchHit>, KbError> {
        let q = query.to_lowercase();
        let guard = self.inner.read().await;
        let mut hits: Vec<SearchHit> = guard
            .mius
            .values()
            .filter(|m| m.status == MiuStatus::Active)
            .filter(|m| Self::in_scopes(m, scope_ids))
            .filter_map(|m| {
                let text = m.raw_summary.to_lowercase();
                if !text.contains(&q) {
                    return None;
                }
                let score = q.len() as f64 / text.len().max(1) as f64;
                Some(SearchHit {
                    miu: m.clone(),
                    score,
                })
            })
            .collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(limit.max(1));
        Ok(hits)
    }

    async fn search_symbol(
        &self,
        query: &str,
        scope_ids: Option<&[ScopeId]>,
        limit: usize,
    ) -> Result<Vec<SearchHit>, KbError> {
        let q = query.to_lowercase();
        let guard = self.inner.read().await;
        let mut hits: Vec<SearchHit> = guard
            .mius
            .values()
            .filter(|m| m.status == MiuStatus::Active)
            .filter(|m| Self::in_scopes(m, scope_ids))
            .filter_map(|m| {
                let prop = m
                    .symbol
                    .proposition
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase();
                if prop.contains(&q) {
                    Some(SearchHit {
                        miu: m.clone(),
                        score: 1.0,
                    })
                } else {
                    None
                }
            })
            .collect();
        hits.truncate(limit.max(1));
        Ok(hits)
    }

    async fn stats_dim(&self, query: DimQuery) -> Result<DimStat, KbError> {
        let guard = self.inner.read().await;
        let mius: Vec<_> = guard
            .mius
            .values()
            .filter(|m| Self::in_scopes(m, query.scope_ids.as_deref()))
            .cloned()
            .collect();
        let mut map: HashMap<String, (usize, f64)> = HashMap::new();
        for m in &mius {
            let keys: Vec<String> = match query.dimension.as_str() {
                "scope" => {
                    if m.metadata.scopes.is_empty() {
                        vec!["_none".into()]
                    } else {
                        m.metadata.scopes.iter().map(|s| s.to_string()).collect()
                    }
                }
                "source_type" => vec![m
                    .metadata
                    .source
                    .as_ref()
                    .map(|s| s.source_type.clone())
                    .unwrap_or_else(|| "_none".into())],
                "modality" => vec![m
                    .metadata
                    .modality
                    .clone()
                    .unwrap_or_else(|| "_none".into())],
                "status" => {
                    let s = match m.status {
                        MiuStatus::Active => "active",
                        MiuStatus::Stale => "stale",
                    };
                    vec![s.into()]
                }
                other => {
                    return Err(KbError::InvalidInput(format!("未知维度: {other}")));
                }
            };
            for k in keys {
                let entry = map.entry(k).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += m.score.unwrap_or(0.0);
            }
        }
        let buckets = map
            .into_iter()
            .map(|(key, (count, sum))| DimBucket {
                avg_score: if count > 0 {
                    Some(sum / count as f64)
                } else {
                    None
                },
                key,
                count,
            })
            .collect();
        Ok(DimStat {
            dimension: query.dimension,
            buckets,
        })
    }

    async fn dup_prob(
        &self,
        text: &str,
        fingerprint: Option<&str>,
        scope_ids: Option<&[ScopeId]>,
    ) -> Result<DupEstimate, KbError> {
        let guard = self.inner.read().await;
        let mut evidence = Vec::new();

        if let Some(fp) = fingerprint {
            for m in guard.mius.values().filter(|m| Self::in_scopes(m, scope_ids)) {
                if m.metadata.fingerprint.as_deref() == Some(fp) {
                    return Ok(DupEstimate {
                        p_dup: 1.0,
                        evidence: vec!["fingerprint_exact".into()],
                        nearest_miu_id: Some(m.id.clone()),
                    });
                }
            }
        }

        let tokens: Vec<&str> = text.split_whitespace().collect();
        let mut best: Option<(MiuId, f64)> = None;
        for m in guard.mius.values().filter(|m| Self::in_scopes(m, scope_ids)) {
            let other: Vec<&str> = m.raw_summary.split_whitespace().collect();
            if tokens.is_empty() || other.is_empty() {
                continue;
            }
            let set_a: std::collections::HashSet<_> = tokens.iter().copied().collect();
            let set_b: std::collections::HashSet<_> = other.iter().copied().collect();
            let inter = set_a.intersection(&set_b).count() as f64;
            let union = set_a.union(&set_b).count() as f64;
            let j = if union > 0.0 { inter / union } else { 0.0 };
            if best.as_ref().map(|(_, s)| j > *s).unwrap_or(true) {
                best = Some((m.id.clone(), j));
            }
        }

        let (nearest, p) = match best {
            Some((id, j)) => {
                evidence.push(format!("token_jaccard={j:.3}"));
                (Some(id), j)
            }
            None => (None, 0.0),
        };
        Ok(DupEstimate {
            p_dup: p,
            evidence,
            nearest_miu_id: nearest,
        })
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
