//! 插件清单与 provider 合并。

use std::path::{Path, PathBuf};

use frame_core::{AbilityExpose, Error, Result};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Default, Deserialize)]
struct PluginsProfile {
    #[serde(default)]
    claude_mod: Vec<PluginPath>,
    #[serde(default)]
    dsh: Vec<PluginPath>,
}

#[derive(Debug, Deserialize)]
struct PluginPath {
    path: String,
    #[serde(default)]
    loader: Option<String>,
    #[serde(default)]
    config: Option<Value>,
}

pub fn default_provider_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/provider.example.yaml")
}

/// 与 `agent-test` 一致：优先仓库 `config/agent_test.provider.yaml`，再本地 `provider.yaml`。
pub fn resolve_default_provider_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("LEAF_CHAT_PROVIDER") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/agent_test.provider.yaml");
    if repo.is_file() {
        return Some(repo);
    }
    let local = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/provider.yaml");
    if local.is_file() {
        return Some(local);
    }
    let example = default_provider_path();
    if example.is_file() {
        return Some(example);
    }
    None
}

pub fn default_plugins_profile_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dsh/web-profile.plugins.yaml")
}

pub fn load_yaml_value(path: &Path) -> Result<Value> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| Error::config(format!("读取配置失败 {}: {e}", path.display())))?;
    serde_yaml::from_str(&raw).map_err(|e| Error::config(format!("解析 YAML 失败: {e}")))
}

/// 将 `dsh/*.plugins.yaml` 中的路径写入对应 Ability 的 `config.plugins`。
/// 将插件清单 YAML 写入 `abilities.claude_mod` / `dsh_plugin`。
pub fn apply_plugins_profile(doc: &mut Value, profile_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(profile_path)
        .map_err(|e| Error::config(format!("读取插件清单失败: {e}")))?;
    let profile: PluginsProfile = serde_yaml::from_str(&raw)
        .map_err(|e| Error::config(format!("插件清单格式无效: {e}")))?;
    let base = profile_path.parent().unwrap_or(Path::new("."));
    let abilities = doc
        .as_object_mut()
        .and_then(|o| o.get_mut("abilities"))
        .and_then(|a| a.as_object_mut())
        .ok_or_else(|| Error::config("装配缺少 abilities"))?;

    if !profile.claude_mod.is_empty() {
        let entry = abilities.entry("claude_mod".to_string()).or_insert(json!({}));
        entry["enabled"] = json!(true);
        let plugins = profile
            .claude_mod
            .iter()
            .map(|p| plugin_entry_json(base, p))
            .collect::<Vec<_>>();
        entry["config"] = json!({ "plugins": plugins });
    }
    if !profile.dsh.is_empty() {
        let entry = abilities.entry("dsh_plugin".to_string()).or_insert(json!({}));
        entry["enabled"] = json!(true);
        let plugins = profile
            .dsh
            .iter()
            .map(|p| plugin_entry_json(base, p))
            .collect::<Vec<_>>();
        entry["config"] = json!({ "plugins": plugins });
    }
    Ok(())
}

fn plugin_entry_json(base: &Path, p: &PluginPath) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("path".into(), json!(resolve_profile_path(base, &p.path)));
    if let Some(loader) = &p.loader {
        obj.insert("loader".into(), json!(loader));
    }
    if let Some(cfg) = &p.config {
        obj.insert("config".into(), cfg.clone());
    }
    Value::Object(obj)
}

fn resolve_profile_path(base: &Path, path: &str) -> String {
    let p = Path::new(path);
    if p.is_absolute() {
        return path.to_string();
    }
    base.join(p).to_string_lossy().into_owned()
}

pub fn merge_chat_provider(mut expose: AbilityExpose, provider: Value) -> AbilityExpose {
    let chat_cfg = expose.abilities.entry("chat".into()).or_default();
    chat_cfg.enabled = Some(true);
    chat_cfg.config = Some(provider);
    expose
}

pub fn load_assembly_document(
    assembly_path: &Path,
    plugins_profile: Option<&Path>,
) -> Result<Value> {
    let mut doc = load_yaml_value(assembly_path)?;
    if let Some(profile) = plugins_profile {
        if profile.is_file() {
            apply_plugins_profile(&mut doc, profile)?;
        }
    }
    Ok(doc)
}
