use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{Notifier, OutboundError, OutboundEvent};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub url: String,
    pub topics: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Value>,
    /// 可选作用域键（由调用方解释，如 kb_id），outbound 不做业务校验。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeliveryRecord {
    pub subscription_id: String,
    pub url: String,
    pub event: OutboundEvent,
}

/// 订阅登记 + 内存投递记录（可扩展为真实 HTTP）。
pub struct OutboundHub {
    subs: RwLock<HashMap<String, Subscription>>,
    deliveries: RwLock<Vec<DeliveryRecord>>,
}

impl OutboundHub {
    pub fn new() -> Self {
        Self {
            subs: RwLock::new(HashMap::new()),
            deliveries: RwLock::new(Vec::new()),
        }
    }

    pub async fn subscribe(&self, mut sub: Subscription) -> Result<Subscription, OutboundError> {
        if sub.url.is_empty() {
            return Err(OutboundError::Invalid("url 为空".into()));
        }
        if sub.topics.is_empty() {
            return Err(OutboundError::Invalid("topics 为空".into()));
        }
        if sub.id.is_empty() {
            sub.id = Uuid::new_v4().to_string();
        }
        let id = sub.id.clone();
        self.subs.write().await.insert(id, sub.clone());
        Ok(sub)
    }

    pub async fn unsubscribe(&self, id: &str) -> Result<(), OutboundError> {
        if self.subs.write().await.remove(id).is_none() {
            return Err(OutboundError::NotFound(id.into()));
        }
        Ok(())
    }

    pub async fn list(&self, scope_key: Option<&str>) -> Vec<Subscription> {
        let guard = self.subs.read().await;
        guard
            .values()
            .filter(|s| match scope_key {
                None => true,
                Some(k) => s.scope_key.as_deref() == Some(k),
            })
            .cloned()
            .collect()
    }

    pub async fn deliveries(&self) -> Vec<DeliveryRecord> {
        self.deliveries.read().await.clone()
    }

    pub async fn clear_deliveries(&self) {
        self.deliveries.write().await.clear();
    }
}

impl Default for OutboundHub {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Notifier for OutboundHub {
    async fn emit(&self, event: OutboundEvent) -> Result<(), OutboundError> {
        let subs: Vec<_> = {
            let guard = self.subs.read().await;
            guard
                .values()
                .filter(|s| s.topics.iter().any(|t| t == "*" || t == &event.topic))
                .cloned()
                .collect()
        };
        let mut records = self.deliveries.write().await;
        for sub in subs {
            // 首期记录投递意图；真实 HTTP 可在此扩展
            records.push(DeliveryRecord {
                subscription_id: sub.id,
                url: sub.url,
                event: event.clone(),
            });
        }
        Ok(())
    }
}

/// 便于工具侧共享。
pub type SharedOutbound = Arc<OutboundHub>;
