//! 市面插件集成（需 vendor + node；默认 `#[ignore]`，显式跑见手册）。

use std::path::PathBuf;

use frame_core::{AbilityExpose, InstanceId, InvokeRequest, MethodId};
use serde_json::json;
use web_agent::bootstrap_expose;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn vendor_dir() -> PathBuf {
    manifest_dir().join("vendor")
}

fn node_ok() -> bool {
    std::process::Command::new(std::env::var("LEAF_PLUGIN_NODE").unwrap_or_else(|_| "node".into()))
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn market_expose() -> Result<Option<AbilityExpose>, frame_core::Error> {
    let profile = manifest_dir().join("market/market.plugins.yaml");
    if !profile.is_file() {
        return Ok(None);
    }
    let assembly = manifest_dir().join("assembly.yaml");
    let doc = web_agent::load_assembly_document(&assembly, Some(&profile))?;
    Ok(Some(web_agent::expose_from_document(&doc)?))
}

fn bash_ok() -> bool {
    ["bash", "sh"].iter().any(|cmd| {
        std::process::Command::new(cmd)
            .arg("-c")
            .arg("exit 0")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
}

#[tokio::test]
#[ignore = "市面插件：cargo test -p web-agent --test market_plugins -- --ignored"]
async fn official_learning_output_style_session_start() {
    if !node_ok() || !bash_ok() {
        eprintln!("跳过：需要 node 与 bash/sh");
        return;
    }
    let plugin = vendor_dir().join("claude-plugins-official/plugins/learning-output-style");
    if !plugin.join("hooks/hooks.json").is_file() {
        eprintln!("跳过：先运行 apps/web-agent/scripts/fetch-ecosystem-vendor.ps1");
        return;
    }
    let expose = AbilityExpose::from_value(&json!({
        "unknown": "deny",
        "defaults": { "enabled": false },
        "abilities": {
            "dispatch": { "enabled": true },
            "claude_mod": {
                "enabled": true,
                "config": { "plugins": [{ "path": plugin.to_string_lossy() }] }
            }
        }
    }))
    .expect("expose");

    let boot = bootstrap_expose(expose).await.expect("bootstrap");
    let resp = boot
        .app
        .gateway
        .invoke(InvokeRequest::new(
            InstanceId::new("claude_mod").unwrap(),
            MethodId::new("invoke").unwrap(),
            json!({
                "tool": "session.start",
                "args": { "session_id": "market-test" }
            }),
        ))
        .await
        .expect("invoke");
    let body = resp.data.to_string();
    assert!(
        body.contains("learning")
            || body.contains("additionalContext")
            || body.contains("hookSpecificOutput"),
        "官方 learning-output-style 应返回上下文: {body}"
    );
}

#[tokio::test]
#[ignore = "市面插件：cargo test -p web-agent --test market_plugins -- --ignored"]
async fn market_profile_doctor_ok() {
    if !node_ok() {
        return;
    }
    let expose = match market_expose() {
        Ok(Some(e)) => e,
        Ok(None) => {
            eprintln!("跳过：无 market.plugins.yaml");
            return;
        }
        Err(e) => panic!("{e}"),
    };
    let boot = bootstrap_expose(expose).await.expect("bootstrap");
    let report = web_agent::diagnose(&web_agent::WebBoot { app: boot.app })
        .await
        .expect("doctor");
    if let Some(m) = report.get("claude_mod") {
        assert_eq!(
            m.get("ok").and_then(|v| v.as_bool()),
            Some(true),
            "{report}"
        );
    }
    if let Some(d) = report.get("dsh_plugin") {
        assert_eq!(
            d.get("ok").and_then(|v| v.as_bool()),
            Some(true),
            "{report}"
        );
    }
}

#[tokio::test]
#[ignore = "市面插件：cargo test -p web-agent --test market_plugins -- --ignored"]
async fn market_profile_dsh_thin_registers_eco_tools() {
    if !node_ok() {
        return;
    }
    let expose = match market_expose() {
        Ok(Some(e)) => e,
        Ok(None) => {
            eprintln!("跳过：无 market.plugins.yaml");
            return;
        }
        Err(e) => panic!("{e}"),
    };
    let boot = bootstrap_expose(expose).await.expect("bootstrap");
    let resp = boot
        .app
        .gateway
        .invoke(InvokeRequest::new(
            InstanceId::new("dispatch").unwrap(),
            MethodId::new("defs").unwrap(),
            json!({}),
        ))
        .await
        .expect("defs");
    let names: Vec<String> = resp
        .data
        .get("tools")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    t.get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        .filter(|n| n.starts_with("eco__dsh__"))
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();
    assert!(
        names
            .iter()
            .any(|n| n.starts_with("eco__dsh__") && n.ends_with("__leaf_echo")),
        "market 清单 thin DSH 应注册 leaf_echo，且无 MCP 桥命名: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n.contains("mcp__")),
        "不应依赖 MCP 桥工具: {names:?}"
    );
    let modlens = vendor_dir().join("dsh-npm-profile/node_modules/@liustack/modlens/dsh/index.js");
    if modlens.is_file() {
        assert!(
            names.iter().any(|n| n.ends_with("__modlens_read_image")),
            "cordis modlens 应注册 modlens_read_image: {names:?}"
        );
    }
}
