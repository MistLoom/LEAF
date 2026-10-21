//! HTTP 路由。

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    extract::{Path, State},
    response::{
        Html, IntoResponse, Json,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
};
use futures_util::StreamExt;
use futures_util::future::ready;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio_stream::wrappers::BroadcastStream;

use crate::diagnose_runtime;
use crate::gateway::{invoke, run_turn};
use crate::persist;
use crate::web::WebState;

pub fn static_router() -> Router<Arc<WebState>> {
    Router::new()
        .route("/", get(index))
        .route("/style.css", get(style))
        .route("/app.js", get(script))
        .route("/vendor/marked.min.js", get(vendor_marked))
}

pub fn api_router() -> Router<Arc<WebState>> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/ecosystem", get(ecosystem))
        .route("/api/ecosystem/doctor", get(ecosystem_doctor))
        .route("/api/ecosystem/inventory", get(ecosystem_inventory))
        .route("/api/ecosystem/invoke", post(ecosystem_invoke))
        .route("/api/ecosystem/dispatch-run", post(ecosystem_dispatch_run))
        .route("/api/dispatch/tools", get(dispatch_tools))
        .route("/api/sessions", get(list_sessions).post(create_session))
        .route("/api/sessions/{id}/messages", get(session_messages))
        .route("/api/sessions/{id}/chat", post(session_chat))
        .route("/api/cancel", post(cancel_turn))
        .route("/api/events", get(event_stream))
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../../web/index.html"))
}

async fn style() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../../web/style.css"),
    )
}

async fn script() -> impl IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../../web/app.js"),
    )
}

async fn vendor_marked() -> impl IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../../web/vendor/marked.min.js"),
    )
}

#[derive(Serialize)]
struct HealthResp {
    ok: bool,
    restored_sessions: usize,
    ecosystem: crate::runtime::EcosystemFlags,
}

async fn health(State(st): State<Arc<WebState>>) -> Json<HealthResp> {
    Json(HealthResp {
        ok: true,
        restored_sessions: st.restored_count,
        ecosystem: st.rt.ecosystem_enabled(),
    })
}

async fn ecosystem(State(st): State<Arc<WebState>>) -> Json<Value> {
    Json(json!({
        "ecosystem": st.rt.ecosystem_enabled(),
    }))
}

async fn ecosystem_doctor(State(st): State<Arc<WebState>>) -> Result<Json<Value>, ApiError> {
    let out = diagnose_runtime(st.gateway(), &st.rt.expose)
        .await
        .map_err(|e| ApiError(ability_support::Error::failed(e.to_string())))?;
    Ok(Json(out))
}

async fn ecosystem_inventory(State(st): State<Arc<WebState>>) -> Result<Json<Value>, ApiError> {
    let eco = crate::eco_visibility::fetch_eco_dispatch_tools(st.gateway())
        .await
        .map_err(ApiError)?;
    Ok(Json(crate::eco_visibility::build_inventory(
        &st.rt.expose,
        &eco,
    )))
}

#[derive(Deserialize)]
struct EcoInvokeBody {
    ability: String,
    tool: String,
    #[serde(default)]
    args: Value,
}

#[derive(Deserialize)]
struct DispatchRunBody {
    calls: Value,
}

async fn ecosystem_dispatch_run(
    State(st): State<Arc<WebState>>,
    Json(body): Json<DispatchRunBody>,
) -> Result<Json<Value>, ApiError> {
    let calls = body.calls.as_array().cloned().unwrap_or_default();
    let out = invoke(
        st.gateway(),
        "dispatch",
        "run",
        json!({ "calls": calls, "continue_on_error": true }),
    )
    .await
    .map_err(ApiError)?;
    Ok(Json(json!({
        "messages": out.get("messages"),
        "raw": out,
    })))
}

async fn ecosystem_invoke(
    State(st): State<Arc<WebState>>,
    Json(body): Json<EcoInvokeBody>,
) -> Result<Json<Value>, ApiError> {
    let out = crate::ecosystem::invoke_hook(
        st.gateway(),
        &st.rt.expose,
        &body.ability,
        &body.tool,
        body.args,
    )
    .await
    .map_err(ApiError)?;
    Ok(Json(out))
}

async fn dispatch_tools(State(st): State<Arc<WebState>>) -> Result<Json<Value>, ApiError> {
    let defs = invoke(st.gateway(), "dispatch", "defs", json!({}))
        .await
        .map_err(ApiError)?;
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
    Ok(Json(json!({ "eco_tools": eco, "defs": defs })))
}

async fn list_sessions(State(st): State<Arc<WebState>>) -> Result<Json<Value>, ApiError> {
    let items = invoke(st.gateway(), "session", "list", json!({ "limit": 100 }))
        .await
        .map_err(ApiError)?;
    let list = if items.is_array() { items } else { json!([]) };
    Ok(Json(json!({ "items": list })))
}

async fn create_session(State(st): State<Arc<WebState>>) -> Result<Json<Value>, ApiError> {
    let out = invoke(st.gateway(), "session", "create", json!({}))
        .await
        .map_err(ApiError)?;
    if let Some(id) = out.get("id").and_then(Value::as_str) {
        let _ = persist::save_session(st.gateway(), id).await;
    }
    Ok(Json(out))
}

async fn session_messages(
    State(st): State<Arc<WebState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let msgs = invoke(
        st.gateway(),
        "session",
        "protocol_messages",
        json!({ "id": id }),
    )
    .await
    .map_err(ApiError)?;
    Ok(Json(msgs))
}

#[derive(Deserialize)]
struct ChatBody {
    text: String,
}

async fn session_chat(
    State(st): State<Arc<WebState>>,
    Path(id): Path<String>,
    Json(body): Json<ChatBody>,
) -> Result<Json<Value>, ApiError> {
    let out = run_turn(st.gateway(), &body.text, Some(&id))
        .await
        .map_err(ApiError)?;
    let _ = persist::save_session(st.gateway(), &id).await;
    let msgs = invoke(
        st.gateway(),
        "session",
        "protocol_messages",
        json!({ "id": id }),
    )
    .await
    .map_err(ApiError)?;
    let eco_trace = crate::eco_visibility::trace_latest_turn_eco(&msgs);
    let mut resp = out.as_object().cloned().unwrap_or_default();
    resp.insert("ecosystem".into(), eco_trace);
    Ok(Json(Value::Object(resp)))
}

async fn cancel_turn(State(st): State<Arc<WebState>>) -> Result<Json<Value>, ApiError> {
    let out = invoke(st.gateway(), "turn", "cancel", json!({}))
        .await
        .map_err(ApiError)?;
    Ok(Json(out))
}

async fn event_stream(
    State(st): State<Arc<WebState>>,
) -> Sse<impl StreamExt<Item = Result<Event, Infallible>>> {
    let rx = st.events_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(line) => ready(Some(Ok(Event::default().data(line)))),
        Err(_) => ready(None),
    });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

struct ApiError(ability_support::Error);

impl From<ability_support::Error> for ApiError {
    fn from(e: ability_support::Error) -> Self {
        Self(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({ "error": self.0.to_string() })),
        )
            .into_response()
    }
}
