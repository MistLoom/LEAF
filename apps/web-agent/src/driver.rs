//! 驱动层：能力事件 → 用户通道 + SSE。

use std::sync::Arc;

use ability_core::AbilityHost;
use ability_support::{
    host_event, AbilityEvent, BusEvent, EventClass, EventFilter, EventListener, InstanceId,
    UserEvent, UserEventSink,
};
use serde_json::{json, Value};
use tokio::sync::broadcast;

pub fn spawn_driver(host: &AbilityHost, events_tx: broadcast::Sender<String>) {
    spawn_user_sse(host, events_tx.clone());
    spawn_host_pipeline(host.clone(), events_tx.clone());
    spawn_ability_pipeline(host.clone(), events_tx);
}

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
            let line = json!({
                "channel": "host",
                "name": name,
                "detail": ev.payload().clone(),
            });
            if tx.send(line.to_string()).is_err() {
                break;
            }
        }
    });
}

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
                let name = call.get("name").and_then(Value::as_str);
                host.emit_user(UserEvent::new(
                    "tool.started",
                    json!({
                        "call_id": call.get("id"),
                        "name": name,
                        "eco": name.map(|n| n.starts_with("eco__")).unwrap_or(false),
                    }),
                ))
                .await;
                if name.is_some_and(|n| n.starts_with("eco__")) {
                    host.emit_user(UserEvent::new(
                        "eco.tool_started",
                        json!({
                            "call_id": call.get("id"),
                            "name": name,
                            "arguments": call.get("arguments"),
                        }),
                    ))
                    .await;
                }
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
            if role == "user" {
                host.emit_user(UserEvent::new(
                    "prompt.submit",
                    json!({
                        "session_id": ev.payload.get("id"),
                        "role": role,
                    }),
                ))
                .await;
            }
            host.emit_user(
                UserEvent::new("session.items_appended", ev.payload.clone()),
            )
            .await;
        }
        _ => {}
    }
}
