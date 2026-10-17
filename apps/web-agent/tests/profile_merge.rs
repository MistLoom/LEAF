//! 插件清单合并进装配（无 Node）。

use std::path::PathBuf;

use web_agent::{apply_plugins_profile, default_assembly_path, load_assembly_document};

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
