//! 运行时：调用链 InvokeMask 编排（不 mutate 全局 Expose）。

use std::sync::Arc;

use ability_core::AbilityHost;
use ability_support::{InstanceId, MethodId, Result};
use frame_core::{AbilityExpose, Gateway, InvokeMask, InvokeRequest, invoke_mask_from_expose};
use serde_json::{Value, json};

/// Web / spawn 编排所需的最小运行时句柄。
#[derive(Clone)]
pub struct Runtime {
    pub host: AbilityHost,
    pub gateway: Arc<dyn Gateway>,
}

impl Runtime {
    pub fn spawn_mask_from(&self, profile: &AbilityExpose) -> InvokeMask {
        let ids: Vec<String> = self
            .host
            .catalog()
            .into_iter()
            .map(|e| e.instance.to_string())
            .collect();
        let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        invoke_mask_from_expose(profile, &id_refs)
    }

    pub async fn spawn_turn_restricted(
        &self,
        profile: &AbilityExpose,
        session_id: &str,
        prompt: &str,
    ) -> Result<Value> {
        let mask = self.spawn_mask_from(profile);
        invoke_with_mask(
            &self.gateway,
            "turn",
            "spawn",
            json!({
                "session_id": session_id,
                "prompt": prompt,
                "summarize": true,
            }),
            mask,
        )
        .await
    }
}

impl From<frame_compose::AppHandle> for Runtime {
    fn from(app: frame_compose::AppHandle) -> Self {
        Self {
            host: app.host,
            gateway: app.gateway,
        }
    }
}

pub async fn invoke_with_mask(
    gateway: &Arc<dyn Gateway>,
    instance: &str,
    method: &str,
    args: Value,
    mask: InvokeMask,
) -> Result<Value> {
    let resp = gateway
        .invoke(
            InvokeRequest::new(
                InstanceId::new(instance)
                    .map_err(|e| ability_support::Error::invalid_args(e.to_string()))?,
                MethodId::new(method)
                    .map_err(|e| ability_support::Error::invalid_args(e.to_string()))?,
                args,
            )
            .with_invoke_mask(mask),
        )
        .await?;
    Ok(resp.data)
}
