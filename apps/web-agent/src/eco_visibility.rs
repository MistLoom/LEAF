//! 第三方生态插件清单与单轮使用归因（仅 claude_mod / dsh_plugin）。

use std::path::Path;

use frame_core::AbilityExpose;
use serde_json::{json, Value};

const ECO_ABILITIES: [&str; 2] = ["claude_mod", "dsh_plugin"];

pub fn build_inventory(expose: &AbilityExpose, eco_dispatch_tools: &[String]) -> Value {
    let mut abilities = json!({});
    for id in ECO_ABILITIES {
        if !expose.is_enabled(id) {
            continue;
        }
        let cap = expose.resolve(id);
        let plugins = cap
            .config
            .get("plugins")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let enriched: Vec<Value> = plugins
            .iter()
            .map(|p| enrich_plugin_entry(id, p, eco_dispatch_tools))
            .collect();
        abilities[id] = json!({
            "enabled": true,
            "plugins": enriched,
        });
    }
    json!({
        "abilities": abilities,
        "dispatch_eco_tools": eco_dispatch_tools,
        "legend": {
            "claude_mod": "钩子（session.start / turn.* / tool.check 等），多数官方 Mod 需手动 session.start 或事件转发",
            "dsh_plugin": "dispatch 工具 eco__dsh__*，仅当模型在本轮调用时生效",
            "native": "read_file、bash 等 Leaf 内置 Ability，非第三方插件",
        },
    })
}

fn enrich_plugin_entry(ability_id: &str, entry: &Value, eco_tools: &[String]) -> Value {
    let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or("");
    let label = plugin_label(path);
    let loader = entry.get("loader").and_then(|v| v.as_str());
    let bound = if ability_id == "dsh_plugin" {
        bind_dsh_tools(path, loader, eco_tools)
    } else {
        json!([])
    };
    json!({
        "path": path,
        "label": label,
        "loader": loader,
        "dispatch_tools": bound,
    })
}

fn plugin_label(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

fn bind_dsh_tools(path: &str, loader: Option<&str>, eco_tools: &[String]) -> Value {
    let path_l = path.to_lowercase();
    let mut out = Vec::new();
    for full in eco_tools {
        let Some(short) = full.strip_prefix("eco__dsh__") else {
            continue;
        };
        let tool_tail = dsh_tool_tail(short);
        let hit = path_l.contains(&tool_tail.to_lowercase())
            || (path_l.contains("dsh-npm-profile") && tool_tail.contains("modlens"))
            || path_l.ends_with("minimal-dsh.mjs") && tool_tail == "leaf_echo";
        if hit {
            out.push(json!({
                "dispatch_name": full,
                "tool_name": tool_tail,
                "loader": loader,
            }));
        }
    }
    json!(out)
}

/// 从会话协议消息中提取「最近一轮」内的生态工具调用与结果摘要。
pub fn trace_latest_turn_eco(messages: &Value) -> Value {
    let arr = messages
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut segment: Vec<&Value> = Vec::new();
    for msg in arr.iter().rev() {
        if msg.get("role").and_then(|v| v.as_str()) == Some("user") {
            break;
        }
        segment.push(msg);
    }
    segment.reverse();

    let mut tool_calls = Vec::new();
    let mut tool_results = Vec::new();
    for msg in segment {
        if let Some(calls) = msg.get("tool_calls").and_then(|v| v.as_array()) {
            for c in calls {
                let name = c
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                    .or_else(|| c.get("name").and_then(|n| n.as_str()));
                if let Some(name) = name {
                    if let Some(att) = attribute_dispatch_tool(name) {
                        tool_calls.push(att);
                    }
                }
            }
        }
        if msg.get("role").and_then(|v| v.as_str()) == Some("tool") {
            let id = msg.get("tool_call_id").and_then(|v| v.as_str());
            let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let preview: String = content.chars().take(200).collect();
            tool_results.push(json!({
                "tool_call_id": id,
                "preview": preview,
                "chars": content.len(),
            }));
        }
    }

    let used = !tool_calls.is_empty();
    json!({
        "used_third_party": used,
        "summary": if used {
            format!("本轮模型调用了 {} 个生态工具", tool_calls.len())
        } else {
            "本轮未调用 eco__* 生态工具（回复来自内置 chat/turn 与原生 Ability）".into()
        },
        "tool_calls": tool_calls,
        "tool_results": tool_results,
        "mod_hooks_note": "Claude Mod 钩子（如 session.start、turn.complete）不在消息流中留痕；请用侧栏 session.start 或查看事件流",
    })
}

fn dsh_tool_tail(short: &str) -> String {
    let parts: Vec<&str> = short.splitn(2, "__").collect();
    if parts.len() == 2 {
        parts[1].to_string()
    } else {
        short.to_string()
    }
}

fn claude_mod_tool_tail(short: &str) -> String {
    dsh_tool_tail(short)
}

fn attribute_dispatch_tool(llm_name: &str) -> Option<Value> {
    if let Some(short) = llm_name.strip_prefix("eco__dsh__") {
        return Some(json!({
            "ability": "dsh_plugin",
            "dispatch_name": llm_name,
            "tool_name": dsh_tool_tail(short),
            "kind": "dsh_tool",
        }));
    }
    if let Some(short) = llm_name.strip_prefix("eco__claude_mod__") {
        return Some(json!({
            "ability": "claude_mod",
            "dispatch_name": llm_name,
            "tool_name": claude_mod_tool_tail(short),
            "kind": "mod_tool",
        }));
    }
    None
}

pub async fn fetch_eco_dispatch_tools(
    gateway: &std::sync::Arc<dyn frame_core::Gateway>,
) -> ability_support::Result<Vec<String>> {
    let defs = crate::gateway::invoke(gateway, "dispatch", "defs", json!({})).await?;
    let eco: Vec<String> = defs
        .get("tools")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    t.get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        .filter(|n| n.starts_with("eco__"))
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(eco)
}
