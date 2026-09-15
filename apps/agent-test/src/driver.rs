//! 驱动层：订阅 Host / 能力事件，翻译为用户通道（spec §10），经 SSE 送达 WebUI。

use std::sync::Arc;

use ability_core::AbilityHost;
use ability_support::{
    host_event, AbilityEvent, BusEvent, EventClass, EventFilter, EventListener, InstanceId,
    UserEvent, UserEventSink,
};
use serde_json::{json, Value};
use tokio::sync::broadcast;

/// 启动用户通道转发 + Host / 能力事件翻译 + 调试通道 SSE。
pub fn spawn_driver(host: &AbilityHost, events_tx: broadcast::Sender<String>) {
    spawn_user_sse(host, events_tx.clone());
    spawn_host_pipeline(host.clone(), events_tx.clone());
    spawn_ability_pipeline(host.clone(), events_tx);
}

/// 用户通道 → SSE（`channel: user`）。
fn spawn_user_sse(host: &AbilityHost, tx: broadcast::Sender<String>) {
    struct Sink(broadcast::Sender<String>);
    #[async_trait::async_trait]
    impl UserEventSink for Sink {
        async fn on_user_event(&self, event: &UserEvent) {
            let line = json!({
                "channel": "user",
                "name": event.name,
                "detail": event.payload,
            });
            let _ = self.0.send(line.to_string());
        }
    }
    let host = host.clone();
    tokio::spawn(async move {
        let sink: Arc<dyn UserEventSink> = Arc::new(Sink(tx));
        host.events().subscribe_user(sink).await;
        std::future::pending::<()>().await;
    });
}

/// Host 系统事件 → 调试 SSE（`channel: host`）。
fn spawn_host_pipeline(host: AbilityHost, tx: broadcast::Sender<String>) {
    tokio::spawn(async move {
        let mut listener = host
            .events()
            .listen(EventFilter {
                class: Some(EventClass::Host),
                ..Default::default()
            })
            .await;

        while let Some(ev) = listener.next().await {
            let name = ev.name();
            let interesting = name == host_event::LOAD
                || name == host_event::UNLOAD
                || name.starts_with(host_event::LIFECYCLE_PREFIX)
                || name == host_event::INVOKE_BEFORE
                || name == host_event::INVOKE_AFTER
                || name == host_event::INVOKE_ERROR;
            if !interesting {
                continue;
            }

            let discovery = match ev {
                BusEvent::Host(ref h) => h.discovery.as_ref().map(|d| {
                    json!({
                        "instance": d.instance.to_string(),
                        "ability_id": d.ability_id.to_string(),
                        "phase": d.phase.as_str(),
                    })
                }),
                _ => None,
            };
            let line = json!({
                "channel": "host",
                "name": name,
                "discovery": discovery,
                "detail": ev.payload().clone(),
            });
            if tx.send(line.to_string()).is_err() {
                break;
            }
        }
    });
}

/// 能力事件 → 用户通道翻译 + 原始能力事件调试 SSE（`channel: ability`）。
fn spawn_ability_pipeline(host: AbilityHost, tx: broadcast::Sender<String>) {
    tokio::spawn(async move {
        let turn = InstanceId::new("turn").ok();
        let session = InstanceId::new("session").ok();
        let mut listener = host
            .events()
            .listen(EventFilter {
                class: Some(EventClass::Ability),
                ..Default::default()
            })
            .await;

        while let Some(ev) = listener.next().await {
            let BusEvent::Ability(ab) = ev else {
                continue;
            };
            let inst = ab.instance.to_string();
            let interesting = match (&turn, &session, inst.as_str()) {
                (Some(t), Some(s), id) => id == t.as_str() || id == s.as_str(),
                _ => true,
            };
            if !interesting {
                continue;
            }

            let debug_line = json!({
                "channel": "ability",
                "instance": inst,
                "name": ab.name,
                "detail": ab.payload,
            });
            if tx.send(debug_line.to_string()).is_err() {
                break;
            }

            translate_ability_event(&host, &ab).await;
        }
    });
}

async fn translate_ability_event(host: &AbilityHost, ev: &AbilityEvent) {
    match (ev.instance.as_str(), ev.name.as_str()) {
        ("turn", "started") => {
            host.emit_user(UserEvent::new(
                "turn.started",
                json!({
                    "session_id": ev.payload.get("session_id"),
                    "turn_id": ev.payload.get("turn_id"),
                }),
            ))
            .await;
        }
        ("turn", "tool_batch") => {
            let Some(calls) = ev.payload.get("calls").and_then(Value::as_array) else {
                return;
            };
            for call in calls {
                host.emit_user(UserEvent::new(
                    "tool.started",
                    json!({
                        "call_id": call.get("id"),
                        "name": call.get("name"),
                    }),
                ))
                .await;
            }
        }
        ("turn", "completed") => {
            host.emit_user(UserEvent::new(
                "turn.completed",
                json!({
                    "session_id": ev.payload.get("session_id"),
                    "reason": ev.payload.get("reason"),
                    "text": ev.payload.get("text"),
                }),
            ))
            .await;
        }
        ("session", "items_appended") => {
            let role = ev.payload.get("role").and_then(Value::as_str).unwrap_or("");
            if role == "tool" {
                host.emit_user(UserEvent::new(
                    "tool.result",
                    json!({
                        "session_id": ev.payload.get("id"),
                        "role": role,
                    }),
                ))
                .await;
            }
            host.emit_user(UserEvent::new(
                "session.items_appended",
                ev.payload.clone(),
            ))
            .await;
        }
        _ => {}
    }
}
