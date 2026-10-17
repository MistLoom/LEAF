//! Gateway 调用辅助。

use std::sync::Arc;

use ability_support::{InstanceId, MethodId, Result};
use frame_core::{Gateway, InvokeRequest};
use serde_json::{json, Value};

pub async fn invoke(
    gateway: &Arc<dyn Gateway>,
    instance: &str,
    method: &str,
    args: Value,
) -> Result<Value> {
    let resp = gateway
        .invoke(InvokeRequest::new(
            InstanceId::new(instance).map_err(|e| ability_support::Error::invalid_args(e.to_string()))?,
            MethodId::new(method).map_err(|e| ability_support::Error::invalid_args(e.to_string()))?,
            args,
        ))
        .await?;
    Ok(resp.data)
}

pub async fn run_turn(
    gateway: &Arc<dyn Gateway>,
    text: &str,
    session_id: Option<&str>,
) -> Result<Value> {
    let mut args = json!({
        "text": text,
        "create_session": session_id.is_none(),
    });
    if let Some(sid) = session_id {
        args["session_id"] = json!(sid);
    }
    invoke(gateway, "turn", "run", args).await
}
