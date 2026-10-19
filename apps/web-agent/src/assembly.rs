//! 从 assembly YAML 引导 Gateway。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ability_core::AbilityHost;
use ability_plugin_host::set_user_fanout;
use ability_support::UserEvent;
use frame_compose::{AppBuilder, AppHandle};
use frame_core::{AbilityExpose, Error, Result};
use serde_json::Value;

use crate::profile::{
    load_assembly_document, load_yaml_value, merge_chat_provider, resolve_default_provider_path,
};

pub struct WebBoot {
    pub app: AppHandle,
}

pub fn default_assembly_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assembly.yaml")
}

pub fn expose_from_document(doc: &Value) -> Result<AbilityExpose> {
    if doc.get("abilities").is_some()
        || doc.get("expose").is_some()
        || doc.get("tool_expose").is_some()
    {
        return AbilityExpose::from_value(doc);
    }
    Err(Error::config("装配缺少 abilities / expose 段"))
}

pub async fn bootstrap(assembly: Option<&Path>) -> Result<WebBoot> {
    bootstrap_with_options(assembly, None, None).await
}

/// `plugins_profile`：合并生态插件路径；`provider`：注入 `chat` 凭据。
pub async fn bootstrap_with_options(
    assembly: Option<&Path>,
    plugins_profile: Option<&Path>,
    provider: Option<&Path>,
) -> Result<WebBoot> {
    let path = assembly
        .map(PathBuf::from)
        .unwrap_or_else(default_assembly_path);
    let doc = load_assembly_document(&path, plugins_profile)?;
    let mut expose = expose_from_document(&doc)?;
    if let Some(p) = provider {
        if p.is_file() {
            let provider_val = load_yaml_value(p)?;
            expose = merge_chat_provider(expose, provider_val);
        }
    } else if let Some(def) = resolve_default_provider_path() {
        let provider_val = load_yaml_value(&def)?;
        expose = merge_chat_provider(expose, provider_val);
    }
    ability_plugin_host::coordinator().reset().await;
    let app = AppBuilder::new()
        .app("web-agent")
        .expose(expose)
        .bootstrap()
        .await;
    install_eco_user_fanout(&app.host);
    Ok(WebBoot { app })
}

pub async fn bootstrap_expose(expose: AbilityExpose) -> Result<WebBoot> {
    ability_plugin_host::coordinator().reset().await;
    let app = AppBuilder::new()
        .app("web-agent")
        .expose(expose)
        .bootstrap()
        .await;
    install_eco_user_fanout(&app.host);
    Ok(WebBoot { app })
}

fn install_eco_user_fanout(host: &AbilityHost) {
    let host = host.clone();
    set_user_fanout(Arc::new(move |name, payload| {
        let host = host.clone();
        let name = name.to_string();
        tokio::spawn(async move {
            host.emit_user(UserEvent::new(&name, payload)).await;
        });
    }));
}
