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

#[derive(Debug, Clone, Deserialize)]
struct PluginPath {
    path: Option<String>,
    /// npm 钉扎目录（含 `package.json` + `node_modules`），展开为带 `dsh` 清单的直接依赖包。
    #[serde(default)]
    vendor_npm: Option<String>,
    /// Claude 官方仓库 `plugins/` 一类目录，展开为含 `hooks/hooks.json` 的子目录。
    #[serde(default)]
    vendor_plugins_dir: Option<String>,
    #[serde(default)]
    loader: Option<String>,
    #[serde(default)]
    config: Option<Value>,
}

#[derive(Debug, Clone)]
struct ResolvedPlugin {
    path: String,
    loader: Option<String>,
    config: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct VendorPackageJson {
    #[serde(default)]
    dependencies: std::collections::BTreeMap<String, String>,
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
        let plugins = resolve_plugin_entries(base, &profile.claude_mod)?
            .iter()
            .map(plugin_entry_json)
            .collect::<Vec<_>>();
        entry["config"] = json!({ "plugins": plugins });
    }
    if !profile.dsh.is_empty() {
        let entry = abilities.entry("dsh_plugin".to_string()).or_insert(json!({}));
        entry["enabled"] = json!(true);
        let plugins = resolve_plugin_entries(base, &profile.dsh)?
            .iter()
            .map(plugin_entry_json)
            .collect::<Vec<_>>();
        entry["config"] = json!({ "plugins": plugins });
    }
    Ok(())
}

fn resolve_plugin_entries(base: &Path, entries: &[PluginPath]) -> Result<Vec<ResolvedPlugin>> {
    let mut out = Vec::new();
    for entry in entries {
        out.extend(expand_plugin_entry(base, entry)?);
    }
    Ok(out)
}

fn expand_plugin_entry(base: &Path, entry: &PluginPath) -> Result<Vec<ResolvedPlugin>> {
    let mode_count = [
        entry.path.is_some(),
        entry.vendor_npm.is_some(),
        entry.vendor_plugins_dir.is_some(),
    ]
        .into_iter()
        .filter(|set| *set)
        .count();
    if mode_count != 1 {
        return Err(Error::config(
            "插件项需且仅需指定 path、vendor_npm 或 vendor_plugins_dir 之一",
        ));
    }
    if let Some(path) = &entry.path {
        return Ok(vec![ResolvedPlugin {
            path: resolve_profile_path(base, path),
            loader: entry.loader.clone(),
            config: entry.config.clone(),
        }]);
    }
    if let Some(vendor) = &entry.vendor_npm {
        return expand_vendor_npm(base, vendor, entry);
    }
    let dir = entry.vendor_plugins_dir.as_ref().expect("vendor_plugins_dir");
    expand_claude_vendor_plugins_dir(base, dir, entry)
}

fn expand_vendor_npm(base: &Path, vendor_rel: &str, template: &PluginPath) -> Result<Vec<ResolvedPlugin>> {
    let vendor_dir = PathBuf::from(resolve_profile_path(base, vendor_rel));
    let pkg_path = vendor_dir.join("package.json");
    let raw = std::fs::read_to_string(&pkg_path).map_err(|e| {
        Error::config(format!("读取 vendor npm 清单失败 {}: {e}", pkg_path.display()))
    })?;
    let vendor: VendorPackageJson = serde_json::from_str(&raw).map_err(|e| {
        Error::config(format!("解析 vendor package.json 失败: {e}"))
    })?;
    let modules = vendor_dir.join("node_modules");
    let mut plugins = Vec::new();
    for dep_name in vendor.dependencies.keys() {
        let dep_dir = npm_package_dir(&modules, dep_name);
        let dep_pkg = dep_dir.join("package.json");
        if !dep_pkg.is_file() {
            return Err(Error::config(format!(
                "vendor 依赖未安装: {dep_name}（在 {} 执行 npm ci）",
                vendor_dir.display()
            )));
        }
        if !package_json_has_dsh_plugin(&dep_pkg)? {
            continue;
        }
        plugins.push(ResolvedPlugin {
            path: dep_dir.to_string_lossy().into_owned(),
            loader: template.loader.clone(),
            config: template.config.clone(),
        });
    }
    if plugins.is_empty() {
        return Err(Error::config(format!(
            "vendor_npm 未解析到任何 DSH 插件（{} dependencies 中无 dsh 清单）",
            vendor_dir.display()
        )));
    }
    Ok(plugins)
}

fn expand_claude_vendor_plugins_dir(
    base: &Path,
    dir_rel: &str,
    template: &PluginPath,
) -> Result<Vec<ResolvedPlugin>> {
    let root = PathBuf::from(resolve_profile_path(base, dir_rel));
    if !root.is_dir() {
        return Err(Error::config(format!("vendor_plugins_dir 不是目录: {}", root.display())));
    }
    let mut plugins = Vec::new();
    let children = std::fs::read_dir(&root).map_err(|e| {
        Error::config(format!("读取 vendor_plugins_dir 失败 {}: {e}", root.display()))
    })?;
    for child in children {
        let child = child.map_err(|e| Error::config(format!("读取子目录失败: {e}")))?;
        let path = child.path();
        if !path.is_dir() {
            continue;
        }
        if path.join("hooks").join("hooks.json").is_file() {
            plugins.push(ResolvedPlugin {
                path: path.to_string_lossy().into_owned(),
                loader: template.loader.clone(),
                config: template.config.clone(),
            });
        }
    }
    plugins.sort_by(|a, b| a.path.cmp(&b.path));
    if plugins.is_empty() {
        return Err(Error::config(format!(
            "vendor_plugins_dir 未找到含 hooks/hooks.json 的插件: {}",
            root.display()
        )));
    }
    Ok(plugins)
}

fn npm_package_dir(node_modules: &Path, name: &str) -> PathBuf {
    if let Some((scope, pkg)) = name.split_once('/') {
        node_modules.join(scope).join(pkg)
    } else {
        node_modules.join(name)
    }
}

fn package_json_has_dsh_plugin(pkg_json: &Path) -> Result<bool> {
    let raw = std::fs::read_to_string(pkg_json)
        .map_err(|e| Error::config(format!("读取 {}: {e}", pkg_json.display())))?;
    let doc: Value = serde_json::from_str(&raw)
        .map_err(|e| Error::config(format!("解析 {}: {e}", pkg_json.display())))?;
    Ok(doc.get("dsh").is_some())
}

fn plugin_entry_json(p: &ResolvedPlugin) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("path".into(), json!(p.path));
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
