//! Web 编码代理参考应用。
//!
//! 文件、shell、Agent 回路与生态插件来自 `common` 与 `agent-test` 的开放能力。

mod assembly;
mod driver;
mod eco_visibility;
pub mod ecosystem;
pub mod gateway;
mod persist;
mod profile;
mod runtime;

pub mod web;

pub use assembly::{
    WebBoot, bootstrap, bootstrap_expose, bootstrap_with_options, default_assembly_path,
    expose_from_document,
};
pub use profile::{
    apply_plugins_profile, default_plugins_profile_path, default_provider_path,
    load_assembly_document, resolve_default_provider_path,
};
pub use runtime::{EcosystemFlags, Runtime};

use frame_core::{InstanceId, InvokeRequest, MethodId, Result};
use serde_json::{Value, json};
use std::sync::Arc;

/// 对已启用生态 Ability 调用 `doctor`。
pub async fn diagnose(boot: &WebBoot) -> Result<Value> {
    diagnose_runtime(&boot.app.gateway, &boot.app.expose).await
}

pub async fn diagnose_runtime(
    gateway: &Arc<dyn frame_core::Gateway>,
    expose: &frame_core::AbilityExpose,
) -> Result<Value> {
    let mut out = json!({});
    for id in ["claude_mod", "dsh_plugin"] {
        if !expose.is_enabled(id) {
            continue;
        }
        let args = expose.resolve(id).config.clone();
        let args = if args.is_null() { json!({}) } else { args };
        let resp = gateway
            .invoke(InvokeRequest::new(
                InstanceId::new(id).map_err(|e| frame_core::Error::invalid_args(e.to_string()))?,
                MethodId::new("doctor")
                    .map_err(|e| frame_core::Error::invalid_args(e.to_string()))?,
                args,
            ))
            .await?;
        out[id] = resp.data;
    }
    Ok(out)
}
