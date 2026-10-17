//! 默认装配可 bootstrap。

use frame_core::{InstanceId, InvokeRequest, MethodId};
use serde_json::json;
use web_agent::bootstrap;

#[tokio::test]
async fn boots_default_assembly() {
    let boot = bootstrap(None).await.expect("bootstrap");
    let resp = boot
        .app
        .gateway
        .invoke(InvokeRequest::new(
            InstanceId::new("dispatch").unwrap(),
            MethodId::new("defs").unwrap(),
            json!({}),
        ))
        .await
        .expect("dispatch.defs");
    assert!(resp.data.get("tools").and_then(|v| v.as_array()).is_some());
}
