//! 把宿主事件与能力事件转到 Web 通道。
//!
//! 用户事件与 Host 系统事件走引擎总线。能力事件经入口应用的 `AppControl`：
//! 只收到本应用、`common` 以及已开放的他应用能力。

use std::sync::Arc;

use ability_core::AbilityHost;
use ability_support::{
    AbilityEvent, BusEvent, EventClass, EventFilter, EventListener, InstanceId, UserEvent,
    UserEventSink, host_event,
};
use frame_compose::{AppControl, AppHandle};
use serde_json::{Value, json};
use tokio::sync::broadcast;

pub fn spawn_driver(app: &AppHandle, events_tx: broadcast::Sender<String>) {
    spawn_user_sse(&app.host, events_tx.clone());
    spawn_host_pipeline(app.host.clone(), events_tx.clone());
    spawn_ability_pipeline(app.control(), app.host.clone(), events_tx);
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
        let listener = host
            .events()
            .listen(EventFilter {
                class: Some(EventClass::Host),
                ..Default::default()
            })
            .await;

        forward_host_events(listener, tx).await;
    });
}

async fn forward_host_events(mut listener: impl EventListener, tx: broadcast::Sender<String>) {
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
        // 暂无 SSE 客户端是正常状态，后续客户端仍可订阅。
        let _ = tx.send(line.to_string());
    }
}

fn spawn_ability_pipeline(control: AppControl, host: AbilityHost, tx: broadcast::Sender<String>) {
    tokio::spawn(async move {
        let Ok(listener) = control
            .listen(EventFilter {
                class: Some(EventClass::Ability),
                ..Default::default()
            })
            .await
        else {
            return;
        };

        forward_ability_events(listener, host, tx).await;
    });
}

async fn forward_ability_events(
    mut listener: impl EventListener,
    host: AbilityHost,
    tx: broadcast::Sender<String>,
) {
    let turn = InstanceId::new("turn").ok();
    let session = InstanceId::new("session").ok();
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
        // 即使没有 SSE 客户端，也继续翻译事件并监听后续消息。
        let _ = tx.send(debug_line.to_string());
        translate_ability_event(&host, &ab).await;
    }
}

#[cfg(test)]
mod tests;

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
            host.emit_user(UserEvent::new("session.items_appended", ev.payload.clone()))
                .await;
        }
        _ => {}
    }
}
