//! 读 expose / provider 配置并合并 chat 凭据。

use std::path::Path;

use frame_core::{AbilityExpose, Error, Result};
use serde_json::{json, Value};

pub fn load_expose(path: impl AsRef<Path>) -> Result<AbilityExpose> {
    let raw = std::fs::read_to_string(path.as_ref())
        .map_err(|e| Error::config(format!("读取 expose 失败: {e}")))?;
    AbilityExpose::from_value(&serde_yaml::from_str::<Value>(&raw).map_err(|e| {
        Error::config(format!("解析 expose 失败: {e}"))
    })?)
}

pub fn load_provider(path: impl AsRef<Path>) -> Result<Value> {
    let raw = std::fs::read_to_string(path.as_ref())
        .map_err(|e| Error::config(format!("读取 provider 失败: {e}")))?;
    serde_yaml::from_str(&raw).map_err(|e| Error::config(format!("解析 provider 失败: {e}")))
}

pub fn merge_chat_config(mut expose: AbilityExpose, provider: Value) -> AbilityExpose {
    let chat_cfg = expose.abilities.entry("chat".into()).or_default();
    chat_cfg.enabled = Some(true);
    chat_cfg.config = Some(provider);
    expose
}

pub fn default_paths() -> (std::path::PathBuf, std::path::PathBuf) {
    let root = std::env::current_dir().unwrap_or_else(|_| ".".into());
    (
        root.join("config/agent_test.expose.yaml"),
        root.join("config/agent_test.provider.yaml"),
    )
}

pub fn web_expose_path() -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| ".".into())
        .join("config/agent_test.web.expose.yaml")
}

pub fn spawn_expose_path() -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| ".".into())
        .join("config/agent_test.spawn.expose.yaml")
}

pub fn load_spawn_expose() -> Result<AbilityExpose> {
    load_expose(spawn_expose_path())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_injects_provider() {
        let expose = AbilityExpose::open_all();
        let provider = json!({
            "base_url": "https://example.com",
            "api_key": "k",
            "model": "m"
        });
        let merged = merge_chat_config(expose, provider.clone());
        let chat = merged.abilities.get("chat").unwrap();
        assert_eq!(chat.config.as_ref(), Some(&provider));
    }
}
