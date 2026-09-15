//! Showcase: 经 Fun 消费 AbilityExpose（外部接入，不编写开放表）。

use std::path::PathBuf;
use std::sync::Arc;

use frame_core::{AccessKind, AbilityExpose, ArtifactKind, LocalsScope};
use frame_compose::compose::{access_allowed, build_fun_port_with, ComposeOptions};
use frame_compose::{ConfigCatalog, ToolHost};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========== leafEX Showcase ==========\n");

    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../config/tool_expose.example.yaml");
    let catalog = Arc::new(ConfigCatalog::new());
    let (id, value) = catalog.load_id(Some("framework_expose"), &config_path)?;
    let expose = AbilityExpose::from_value(&value)?;

    let opts = ComposeOptions {
        config: Some(catalog),
        config_id: Some(id),
        host: ToolHost::new(expose.clone()),
        access_gate: Some(AccessKind::Fun),
        ..Default::default()
    };

    scenario_per_cap_summary(&expose);
    let fun = build_fun_port_with(opts.clone()).await;
    let listed = fun.catalog();
    let ids: Vec<&str> = listed.iter().map(|e| e.instance.as_str()).collect();

    scenario_tool_list(&fun).await?;
    scenario_fun_echo_shout(&fun).await?;
    scenario_counter_session(&fun).await?;
    scenario_fs_roundtrip(&fun).await?;
    scenario_terminal(&fun).await?;
    scenario_todo(&fun).await?;
    scenario_per_cap_access(&expose, &ids);
    scenario_tool_bin_hint(&expose, &ids);
    scenario_filtered_denied(&fun).await?;

    println!("\n========== Showcase done ==========");
    Ok(())
}

fn banner(title: &str) {
    println!("----- {title} -----");
}

fn scenario_per_cap_summary(expose: &AbilityExpose) {
    banner("0 AbilityExpose per-instance");
    println!("unknown  = {:?}", expose.unknown);
    for (id, cap) in &expose.abilities {
        println!(
            "  {id}: enabled={:?} access={:?} artifacts={:?}",
            cap.enabled, cap.access, cap.artifacts
        );
    }
    println!();
}

async fn scenario_tool_list(
    fun: &frame_port_fun::FunPort,
) -> Result<(), Box<dyn std::error::Error>> {
    banner("1 Fun catalog");
    for e in fun.catalog() {
        println!("  - {}.{}", e.instance, e.method);
    }
    println!();
    Ok(())
}

async fn scenario_fun_echo_shout(
    fun: &frame_port_fun::FunPort,
) -> Result<(), Box<dyn std::error::Error>> {
    banner("2 Fun invoke");
    let echo = fun
        .invoke_str("algo.echo", json!({"text": "hello leafEX"}), None)
        .await?;
    println!("algo.echo => {}", serde_json::to_string(&echo.data)?);
    let shout = fun
        .invoke_str("app.shout", json!({"text": "hello leafEX"}), None)
        .await?;
    println!("app.shout => {}", serde_json::to_string(&shout.data)?);
    println!();
    Ok(())
}

async fn scenario_counter_session(
    fun: &frame_port_fun::FunPort,
) -> Result<(), Box<dyn std::error::Error>> {
    banner("3 stateful counter (fun only)");
    let sid = LocalsScope::new();
    let r1 = fun
        .invoke_str("demo.counter", json!({"delta": 2}), Some(sid.clone()))
        .await?;
    let r2 = fun
        .invoke_str("demo.counter", json!({"delta": 3}), Some(sid))
        .await?;
    println!("+2 => {}", serde_json::to_string(&r1.data)?);
    println!("+3 => {} (expect count=5)", serde_json::to_string(&r2.data)?);
    println!();
    Ok(())
}

async fn scenario_fs_roundtrip(
    fun: &frame_port_fun::FunPort,
) -> Result<(), Box<dyn std::error::Error>> {
    banner("4 write_file -> read_file");
    let dir = std::env::temp_dir().join("runtime-showcase");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("note.txt");
    let path_str = path.to_string_lossy().to_string();
    let w = fun
        .invoke_str(
            "write_file",
            json!({"path": path_str, "contents": "showcase-fs-ok"}),
            None,
        )
        .await?;
    let r = fun
        .invoke_str("read_file", json!({"path": path_str}), None)
        .await?;
    println!("write => {}", serde_json::to_string(&w.data)?);
    println!("read  => {}", serde_json::to_string(&r.data)?);
    println!();
    Ok(())
}

async fn scenario_terminal(
    fun: &frame_port_fun::FunPort,
) -> Result<(), Box<dyn std::error::Error>> {
    banner("5 bash");
    let r = fun
        .invoke_str(
            "bash",
            json!({"command": "echo showcase-shell-ok", "block_until_ms": 10000}),
            None,
        )
        .await?;
    println!("{}", serde_json::to_string_pretty(&r.data)?);
    println!();
    Ok(())
}

async fn scenario_todo(fun: &frame_port_fun::FunPort) -> Result<(), Box<dyn std::error::Error>> {
    banner("6 todo_write");
    let sid = LocalsScope::new();
    let r = fun
        .invoke_str(
            "todo_write",
            json!({
                "merge": false,
                "todos": [{"id":"1","content":"demo","status":"completed"}]
            }),
            Some(sid),
        )
        .await?;
    println!("{}", serde_json::to_string(&r.data)?);
    println!();
    Ok(())
}

fn scenario_per_cap_access(expose: &AbilityExpose, ids: &[&str]) {
    banner("7 per-instance access");
    for id in ["algo.echo", "demo.counter", "read_file"] {
        let r = expose.resolve(id);
        println!(
            "  {id}: enabled={} access={:?}",
            r.enabled,
            r.access.iter().map(|a| a.as_str()).collect::<Vec<_>>()
        );
    }
    println!(
        "  gate any(cli)={} any(http)={} any(mcp)={} any(fun)={}",
        access_allowed(
            &ComposeOptions {
                host: ToolHost::new(expose.clone()),
                ..Default::default()
            },
            AccessKind::Cli,
            ids
        ),
        access_allowed(
            &ComposeOptions {
                host: ToolHost::new(expose.clone()),
                ..Default::default()
            },
            AccessKind::Http,
            ids
        ),
        access_allowed(
            &ComposeOptions {
                host: ToolHost::new(expose.clone()),
                ..Default::default()
            },
            AccessKind::Mcp,
            ids
        ),
        access_allowed(
            &ComposeOptions {
                host: ToolHost::new(expose.clone()),
                ..Default::default()
            },
            AccessKind::Fun,
            ids
        ),
    );
    println!();
}

fn scenario_tool_bin_hint(expose: &AbilityExpose, ids: &[&str]) {
    banner("8 tool_bin artifacts");
    for id in expose.tool_bin_ids(ids.iter().copied()) {
        let r = expose.resolve(&id);
        let has_bin = r.artifacts.contains(&ArtifactKind::ToolBin);
        println!("  {id}: tool_bin={has_bin}");
    }
    println!(
        "  build: cargo run -p apps-runtime --bin tool-bin -- build"
    );
    println!();
}

async fn scenario_filtered_denied(
    fun: &frame_port_fun::FunPort,
) -> Result<(), Box<dyn std::error::Error>> {
    banner("9 denied unknown ability");
    match fun.invoke_str("config.env", json!({}), None).await {
        Ok(v) => println!("UNEXPECTED => {}", serde_json::to_string(&v.data)?),
        Err(e) => println!("expected fail => {e}"),
    }
    println!();
    Ok(())
}
