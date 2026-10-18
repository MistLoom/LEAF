//! 插件清单合并进装配（无 Node）。

use std::io::Write;
use std::path::PathBuf;

use tempfile::TempDir;
use web_agent::{apply_plugins_profile, default_assembly_path, load_assembly_document};

#[test]
fn vendor_npm_expands_dsh_dependencies_with_dsh_manifest() {
    let tmp = TempDir::new().expect("tmpdir");
    let vendor = tmp.path().join("vendor-npm");
    let modules = vendor.join("node_modules");
    std::fs::create_dir_all(modules.join("demo-dsh-plugin")).expect("mkdir");
    std::fs::write(
        vendor.join("package.json"),
        r#"{"dependencies":{"demo-dsh-plugin":"1.0.0","plain-lib":"1.0.0"}}"#,
    )
    .expect("vendor pkg");
    std::fs::write(
        modules.join("demo-dsh-plugin").join("package.json"),
        r#"{"name":"demo-dsh-plugin","dsh":{"bundle":{}}}"#,
    )
    .expect("plugin pkg");
    std::fs::create_dir_all(modules.join("plain-lib")).expect("plain");
    std::fs::write(
        modules.join("plain-lib").join("package.json"),
        r#"{"name":"plain-lib"}"#,
    )
    .expect("lib");

    let profile = tmp.path().join("profile.yaml");
    let mut f = std::fs::File::create(&profile).expect("profile");
    writeln!(
        f,
        "dsh:\n  - vendor_npm: ./vendor-npm\n    loader: cordis\n"
    )
    .expect("write");

    let mut doc = serde_json::json!({
        "abilities": {
            "dsh_plugin": { "enabled": false }
        }
    });
    apply_plugins_profile(&mut doc, &profile).expect("merge");
    let plugins = doc["abilities"]["dsh_plugin"]["config"]["plugins"]
        .as_array()
        .expect("plugins");
    assert_eq!(plugins.len(), 1);
    assert!(plugins[0]["path"]
        .as_str()
        .unwrap()
        .ends_with("demo-dsh-plugin"));
    assert_eq!(plugins[0]["loader"], "cordis");
}

#[test]
fn vendor_plugins_dir_finds_claude_mod_hooks() {
    let tmp = TempDir::new().expect("tmpdir");
    let root = tmp.path().join("plugins");
    let good = root.join("good-mod");
    std::fs::create_dir_all(good.join("hooks")).expect("hooks");
    std::fs::write(good.join("hooks/hooks.json"), "{}").expect("hooks.json");
    std::fs::create_dir_all(root.join("not-a-mod")).expect("empty");

    let profile = tmp.path().join("profile.yaml");
    std::fs::write(
        &profile,
        "claude_mod:\n  - vendor_plugins_dir: ./plugins\n",
    )
    .expect("profile");

    let mut doc = serde_json::json!({ "abilities": {} });
    apply_plugins_profile(&mut doc, &profile).expect("merge");
    let plugins = doc["abilities"]["claude_mod"]["config"]["plugins"]
        .as_array()
        .expect("plugins");
    assert_eq!(plugins.len(), 1);
    assert!(plugins[0]["path"].as_str().unwrap().ends_with("good-mod"));
}

#[test]
fn vendor_plugins_dir_skips_hooks_module_tier() {
    let tmp = TempDir::new().expect("tmpdir");
    let root = tmp.path().join("plugins");
    let good = root.join("good-mod");
    std::fs::create_dir_all(good.join("hooks")).expect("hooks");
    std::fs::write(good.join("hooks/hooks.json"), "{}").expect("hooks.json");
    let module = root.join("module-mod");
    std::fs::create_dir_all(module.join("hooks")).expect("hooks");
    std::fs::write(
        module.join("hooks/hooks.json"),
        r#"{"modules":["./register.ts"]}"#,
    )
    .expect("hooks.json");

    let profile = tmp.path().join("profile.yaml");
    std::fs::write(
        &profile,
        "claude_mod:\n  - vendor_plugins_dir: ./plugins\n",
    )
    .expect("profile");

    let mut doc = serde_json::json!({ "abilities": {} });
    apply_plugins_profile(&mut doc, &profile).expect("merge");
    let plugins = doc["abilities"]["claude_mod"]["config"]["plugins"]
        .as_array()
        .expect("plugins");
    assert_eq!(plugins.len(), 1);
    assert!(plugins[0]["path"].as_str().unwrap().ends_with("good-mod"));
}

#[test]
fn explicit_path_to_hooks_module_fails() {
    let tmp = TempDir::new().expect("tmpdir");
    let mod_dir = tmp.path().join("module-mod");
    std::fs::create_dir_all(mod_dir.join("hooks")).expect("hooks");
    std::fs::write(
        mod_dir.join("hooks/hooks.json"),
        r#"{"modules":["./register.ts"]}"#,
    )
    .expect("hooks.json");

    let profile = tmp.path().join("profile.yaml");
    std::fs::write(
        &profile,
        "claude_mod:\n  - path: ./module-mod\n",
    )
    .expect("profile");

    let mut doc = serde_json::json!({ "abilities": {} });
    let err = apply_plugins_profile(&mut doc, &profile).expect_err("reject");
    assert!(err.to_string().contains("claude_mod_unsupported:hooks_modules"));
}

#[test]
fn plugins_profile_enables_eco_abilities() {
    let assembly = default_assembly_path();
    let profile = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dsh/web-profile.plugins.yaml");
    let mut doc = load_assembly_document(&assembly, None).expect("assembly");
    apply_plugins_profile(&mut doc, &profile).expect("merge");
    let abilities = doc.get("abilities").and_then(|v| v.as_object()).expect("abilities");
    assert_eq!(
        abilities.get("claude_mod").and_then(|v| v.get("enabled")),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        abilities.get("dsh_plugin").and_then(|v| v.get("enabled")),
        Some(&serde_json::json!(true))
    );
}
