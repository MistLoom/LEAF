//! Hook 订阅配置；不实现 HTTP 投递细节

use std::sync::Arc;

use crate::store::KbHub;
use ability_support::Error;
use crate::outbound::Subscription;
use serde::Deserialize;
use serde_json::{json, Value};

pub(crate) struct Subscribe {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct SubArgs {
    url: String,
    topics: Vec<String>,
    #[serde(default)]
    secret: Option<String>,
    #[serde(default)]
    filter: Option<Value>,
    /// 通常kb_id，由调用方约
    #[serde(default)]
    scope_key: Option<String>,
}
impl Subscribe {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: SubArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let sub = self
            .hub
            .outbound()
            .subscribe(Subscription {
                id: String::new(),
                url: a.url,
                topics: a.topics,
                secret: a.secret,
                filter: a.filter,
                scope_key: a.scope_key,
            })
            .await
            .map_err(|e| Error::failed(e.to_string()))?;
        Ok(json!({ "subscription": sub }))
    }
}

pub(crate) struct Unsubscribe {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct UnsubArgs {
    subscription_id: String,
}
impl Unsubscribe {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: UnsubArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        self.hub
            .outbound()
            .unsubscribe(&a.subscription_id)
            .await
            .map_err(|e| Error::failed(e.to_string()))?;
        Ok(json!({ "ok": true }))
    }
}

pub(crate) struct List {
    pub(crate) hub: Arc<KbHub>,
}
#[derive(Deserialize)]
pub(crate) struct ListArgs {
    #[serde(default)]
    scope_key: Option<String>,
}
impl List {
pub async fn invoke(&self, args: Value) -> Result<Value, Error> {
        let a: ListArgs =
            serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
        let list = self
            .hub
            .outbound()
            .list(a.scope_key.as_deref())
            .await;
        Ok(json!({ "subscriptions": list }))
    }
}

