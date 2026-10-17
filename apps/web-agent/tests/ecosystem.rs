//! 生态 Ability 集成（依赖本机 node）。共享全局宿主协调器，各用例串行执行。

use std::path::PathBuf;

use serial_test::serial;

use frame_core::{AbilityExpose, InstanceId, InvokeRequest, MethodId};
use serde_json::json;
use web_agent::{bootstrap_expose, diagnose};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// `apps/plugin-compat` 中代表第三方 Mod 出站模式的 fixture。
fn compat_mod_fixture(name: &str) -> PathBuf {
    manifest_dir()
        .join("../plugin-compat/tests/fixtures")
        .join(name)
}

fn eco_dsh_tool_name(names: &[String], tool_tail: &str) -> Option<String> {
    names
        .iter()
        .find(|n| {
            n.starts_with("eco__dsh__")
                && (n.ends_with(&format!("__{tool_tail}")) || *n == &format!("eco__dsh__{tool_tail}"))
        })
        .cloned()
}

fn node_ok() -> bool {
    std::process::Command::new(
        std::env::var("LEAF_PLUGIN_NODE").unwrap_or_else(|_| "node".into()),
    )
    .arg("--version")
    .status()
    .map(|s| s.success())
    .unwrap_or(false)
}

fn claude_mod_expose(mod_path: &PathBuf) -> AbilityExpose {
    let path = mod_path.to_string_lossy();
    AbilityExpose::from_value(&json!({
        "unknown": "deny",
        "defaults": { "enabled": false },
        "abilities": {
            "dispatch": { "enabled": true },
            "read_file": { "enabled": true },
            "bash": { "enabled": true },
            "claude_mod": {
                "enabled": true,
                "config": {
                    "plugins": [{ "path": path }]
                }
            }
        }
    }))
    .expect("expose")
}

#[tokio::test]
#[serial]
async fn claude_mod_fixture_invoke_hook() {
    if !node_ok() {
        eprintln!("跳过：未找到 node");
        return;
    }
    let fixture = manifest_dir().join("tests/fixtures/minimal-mod");
    assert!(
        fixture.join("hooks/hooks.json").is_file(),
        "缺少 fixture: {}",
        fixture.display()
    );
    let boot = bootstrap_expose(claude_mod_expose(&fixture))
        .await
        .expect("bootstrap");
    let resp = boot
        .app
        .gateway
        .invoke(InvokeRequest::new(
            InstanceId::new("claude_mod").unwrap(),
            MethodId::new("invoke").unwrap(),
            json!({
                "tool": "session.start",
                "args": { "session_id": "web-test" }
            }),
        ))
        .await
        .expect("invoke");
    assert!(
        resp.data.get("error").is_none(),
        "钩子失败: {}",
        resp.data
    );
}

fn dsh_expose(plugin_path: &PathBuf) -> AbilityExpose {
    let path = plugin_path.to_string_lossy();
    AbilityExpose::from_value(&json!({
        "unknown": "deny",
        "defaults": { "enabled": false },
        "abilities": {
            "dispatch": { "enabled": true },
            "dsh_plugin": {
                "enabled": true,
                "config": {
                    "plugins": [{ "path": path }]
                }
            }
        }
    }))
    .expect("expose")
}

#[tokio::test]
#[serial]
async fn dsh_plugin_tools_in_dispatch_defs() {
    if !node_ok() {
        eprintln!("跳过：未找到 node");
        return;
    }
    let plugin = manifest_dir().join("tests/fixtures/minimal-dsh.mjs");
    assert!(plugin.is_file());
    let boot = bootstrap_expose(dsh_expose(&plugin))
        .await
        .expect("bootstrap");
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
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();
    assert!(
        eco_dsh_tool_name(&names, "leaf_echo").is_some(),
        "工具列表: {names:?}"
    );
}

#[tokio::test]
#[serial]
async fn dsh_plugin_tool_dispatch_run() {
    if !node_ok() {
        return;
    }
    let plugin = manifest_dir().join("tests/fixtures/minimal-dsh.mjs");
    let boot = bootstrap_expose(dsh_expose(&plugin))
        .await
        .expect("bootstrap");
    let defs = boot
        .app
        .gateway
        .invoke(InvokeRequest::new(
            InstanceId::new("dispatch").unwrap(),
            MethodId::new("defs").unwrap(),
            json!({}),
        ))
        .await
        .expect("defs");
    let names: Vec<String> = defs
        .data
        .get("tools")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    t.get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();
    let llm_name = eco_dsh_tool_name(&names, "leaf_echo").expect("leaf_echo 未注册");
    let resp = boot
        .app
        .gateway
        .invoke(InvokeRequest::new(
            InstanceId::new("dispatch").unwrap(),
            MethodId::new("run").unwrap(),
            json!({
                "calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {
                        "name": llm_name,
                        "arguments": "{\"msg\":\"ping\"}"
                    }
                }]
            }),
        ))
        .await
        .expect("run");
    let body = resp.data.to_string();
    assert!(body.contains("ping"), "run 结果: {body}");
}

#[tokio::test]
#[serial]
async fn claude_mod_tool_check_blocks_dispatch_run() {
    if !node_ok() {
        return;
    }
    let fixture = manifest_dir().join("tests/fixtures/tool-check-mod");
    let boot = bootstrap_expose(claude_mod_expose(&fixture))
        .await
        .expect("bootstrap");
    let err = boot
        .app
        .gateway
        .invoke(InvokeRequest::new(
            InstanceId::new("dispatch").unwrap(),
            MethodId::new("run").unwrap(),
            json!({
                "calls": [{
                    "id": "c1",
                    "type": "function",
                    "function": {
                        "name": "read_file_run",
                        "arguments": "{\"path\":\"README.md\"}"
                    }
                }]
            }),
        ))
        .await
        .expect_err("应被 tool.check 拦截");
    assert!(
        err.to_string().contains("fixture_block"),
        "错误: {}",
        err
    );
}

#[tokio::test]
#[serial]
async fn compat_blast_radius_session_start() {
    if !node_ok() {
        return;
    }
    let fixture = compat_mod_fixture("blast-radius");
    assert!(fixture.join("hooks/hooks.json").is_file());
    let boot = bootstrap_expose(claude_mod_expose(&fixture))
        .await
        .expect("bootstrap");
    let resp = boot
        .app
        .gateway
        .invoke(InvokeRequest::new(
            InstanceId::new("claude_mod").unwrap(),
            MethodId::new("invoke").unwrap(),
            json!({
                "tool": "session.start",
                "args": { "session_id": "blast-radius-test" }
            }),
        ))
        .await
        .expect("invoke session.start（含 $.session.cwd 出站）");
    assert!(
        resp.data.get("error").is_none(),
        "blast-radius: {}",
        resp.data
    );
}

#[tokio::test]
#[serial]
async fn compat_replay_theater_turn_complete() {
    if !node_ok() {
        return;
    }
    let fixture = compat_mod_fixture("replay-theater");
    let boot = bootstrap_expose(claude_mod_expose(&fixture))
        .await
        .expect("bootstrap");
    let resp = boot
        .app
        .gateway
        .invoke(InvokeRequest::new(
            InstanceId::new("claude_mod").unwrap(),
            MethodId::new("invoke").unwrap(),
            json!({
                "tool": "turn.complete",
                "args": { "session_id": "rt", "turn_id": "1" }
            }),
        ))
        .await
        .expect("invoke turn.complete（含 $.agents.get 出站）");
    assert!(
        resp.data.get("error").is_none(),
        "replay-theater: {}",
        resp.data
    );
}

#[tokio::test]
#[serial]
async fn compat_token_weather_session_hooks() {
    if !node_ok() {
        return;
    }
    let fixture = compat_mod_fixture("token-weather");
    let boot = bootstrap_expose(claude_mod_expose(&fixture))
        .await
        .expect("bootstrap");
    for tool in ["session.start", "turn.complete"] {
        let resp = boot
            .app
            .gateway
            .invoke(InvokeRequest::new(
                InstanceId::new("claude_mod").unwrap(),
                MethodId::new("invoke").unwrap(),
                json!({
                    "tool": tool,
                    "args": { "session_id": "tw" }
                }),
            ))
            .await
            .expect(tool);
        assert!(
            resp.data.get("error").is_none(),
            "token-weather {tool}: {}",
            resp.data
        );
    }
}

#[tokio::test]
#[serial]
async fn dsh_plugin_doctor_minimal() {
    if !node_ok() {
        return;
    }
    let plugin = manifest_dir().join("tests/fixtures/minimal-dsh.mjs");
    let boot = bootstrap_expose(dsh_expose(&plugin))
        .await
        .expect("bootstrap");
    let report = diagnose(&boot).await.expect("doctor");
    let dsh = report.get("dsh_plugin").expect("dsh_plugin doctor");
    assert_eq!(dsh.get("ok").and_then(|v| v.as_bool()), Some(true), "{dsh}");
}

#[tokio::test]
#[serial]
async fn claude_mod_and_dsh_plugin_coexist() {
    if !node_ok() {
        return;
    }
    let mod_path = manifest_dir().join("tests/fixtures/minimal-mod");
    let dsh_path = manifest_dir().join("tests/fixtures/minimal-dsh.mjs");
    let expose = AbilityExpose::from_value(&json!({
        "unknown": "deny",
        "defaults": { "enabled": false },
        "abilities": {
            "dispatch": { "enabled": true },
            "claude_mod": {
                "enabled": true,
                "config": { "plugins": [{ "path": mod_path.to_string_lossy() }] }
            },
            "dsh_plugin": {
                "enabled": true,
                "config": { "plugins": [{ "path": dsh_path.to_string_lossy() }] }
            }
        }
    }))
    .expect("expose");
    let boot = bootstrap_expose(expose).await.expect("bootstrap");
    let report = diagnose(&boot).await.expect("doctor");
    assert_eq!(
        report.get("claude_mod").and_then(|v| v.get("ok")).and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        report.get("dsh_plugin").and_then(|v| v.get("ok")).and_then(|v| v.as_bool()),
        Some(true)
    );
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
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();
    assert!(
        names.iter().any(|n| n.starts_with("eco__dsh__")),
        "应有 DSH 生态工具: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n.starts_with("eco__claude_mod__")),
        "minimal-mod 无 register 工具，不应伪造 eco__claude_mod__ 条目"
    );
}

#[tokio::test]
#[serial]
async fn both_ecosystems_hook_and_tool_call() {
    if !node_ok() {
        return;
    }
    let mark = manifest_dir().join("tests/fixtures/mark-mod");
    let thin = manifest_dir().join("tests/fixtures/minimal-dsh.mjs");
    let cordis = manifest_dir().join("tests/fixtures/cordis-echo.mjs");
    let expose = AbilityExpose::from_value(&json!({
        "unknown": "deny",
        "defaults": { "enabled": false },
        "abilities": {
            "session": { "enabled": true },
            "dispatch": { "enabled": true },
            "claude_mod": {
                "enabled": true,
                "config": { "plugins": [{ "path": mark.to_string_lossy() }] }
            },
            "dsh_plugin": {
                "enabled": true,
                "config": {
                    "plugins": [
                        { "path": thin.to_string_lossy() },
                        { "path": cordis.to_string_lossy(), "loader": "cordis" }
                    ]
                }
            }
        }
    }))
    .expect("expose");
    let boot = bootstrap_expose(expose).await.expect("bootstrap");
    let created = boot
        .app
        .gateway
        .invoke(InvokeRequest::new(
            InstanceId::new("session").unwrap(),
            MethodId::new("create").unwrap(),
            json!({}),
        ))
        .await
        .expect("session.create 应转发 session.start");
    let id = created
        .data
        .get("id")
        .and_then(|v| v.as_str())
        .expect("session id")
        .to_string();
    let mut item_count = 0u64;
    for _ in 0..20 {
        let got = boot
            .app
            .gateway
            .invoke(InvokeRequest::new(
                InstanceId::new("session").unwrap(),
                MethodId::new("get").unwrap(),
                json!({ "id": id }),
            ))
            .await
            .expect("session.get");
        item_count = got
            .data
            .get("item_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if item_count >= 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(
        item_count >= 1,
        "CC session.start 应在两类宿主重启后仍追加消息, item_count={item_count}"
    );

    let defs = boot
        .app
        .gateway
        .invoke(InvokeRequest::new(
            InstanceId::new("dispatch").unwrap(),
            MethodId::new("defs").unwrap(),
            json!({}),
        ))
        .await
        .expect("defs");
    let names: Vec<String> = defs
        .data
        .get("tools")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    t.get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();
    for tail in ["leaf_echo", "cordis_echo"] {
        let llm_name = eco_dsh_tool_name(&names, tail).unwrap_or_else(|| panic!("{tail} 未注册: {names:?}"));
        let resp = boot
            .app
            .gateway
            .invoke(InvokeRequest::new(
                InstanceId::new("dispatch").unwrap(),
                MethodId::new("run").unwrap(),
                json!({
                    "calls": [{
                        "id": format!("call-{tail}"),
                        "type": "function",
                        "function": {
                            "name": llm_name,
                            "arguments": "{\"msg\":\"ping\"}"
                        }
                    }]
                }),
            ))
            .await
            .unwrap_or_else(|e| panic!("{tail} 调用失败: {e}"));
        let body = resp.data.to_string();
        assert!(body.contains("ping"), "{tail} 结果: {body}");
    }
}

#[tokio::test]
#[serial]
async fn doctor_when_claude_mod_enabled() {
    if !node_ok() {
        return;
    }
    let fixture = manifest_dir().join("tests/fixtures/minimal-mod");
    let boot = bootstrap_expose(claude_mod_expose(&fixture))
        .await
        .expect("bootstrap");
    let report = diagnose(&boot).await.expect("diagnose");
    assert!(report.get("claude_mod").is_some());
}
