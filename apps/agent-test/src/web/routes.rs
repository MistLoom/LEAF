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
use futures_util::future::ready;
use futures_util::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio_stream::wrappers::BroadcastStream;

use crate::gateway::{self, invoke};
use crate::persist;
use crate::web::WebState;

pub fn static_router() -> Router<Arc<WebState>> {
    Router::new()
        .route("/", get(index))
        .route("/style.css", get(style))
        .route("/app.js", get(script))
}

pub fn api_router() -> Router<Arc<WebState>> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/sessions", get(list_sessions).post(create_session))
        .route("/api/sessions/{id}/messages", get(session_messages))
        .route("/api/sessions/{id}/chat", post(session_chat))
        .route("/api/sessions/{id}/compact", post(session_compact))
        .route("/api/sessions/{id}/spawn", post(session_spawn))
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

#[derive(Serialize)]
struct HealthResp {
    ok: bool,
    restored_sessions: usize,
}

async fn health(State(st): State<Arc<WebState>>) -> Json<HealthResp> {
    Json(HealthResp {
        ok: true,
        restored_sessions: st.restored_count,
    })
}

async fn list_sessions(State(st): State<Arc<WebState>>) -> Result<Json<Value>, ApiError> {
    let items = invoke(st.gateway(), "session", "list", json!({ "limit": 100 }))
        .await
        .map_err(ApiError)?;
    Ok(Json(items))
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
    let out = gateway::run_turn(st.gateway(), &body.text, Some(&id))
        .await
        .map_err(ApiError)?;
    let _ = persist::save_session(st.gateway(), &id).await;
    Ok(Json(out))
}

#[derive(Deserialize)]
struct CompactBody {
    until_seq: u64,
    marker_text: Option<String>,
}

async fn session_compact(
    State(st): State<Arc<WebState>>,
    Path(id): Path<String>,
    Json(body): Json<CompactBody>,
) -> Result<Json<Value>, ApiError> {
    let out = invoke(
        st.gateway(),
        "session",
        "compact",
        json!({
            "id": id,
            "until_seq": body.until_seq,
            "marker_text": body.marker_text,
        }),
    )
    .await
    .map_err(ApiError)?;
    let _ = persist::save_session(st.gateway(), &id).await;
    Ok(Json(out))
}

#[derive(Deserialize)]
struct SpawnBody {
    prompt: String,
}

async fn session_spawn(
    State(st): State<Arc<WebState>>,
    Path(id): Path<String>,
    Json(body): Json<SpawnBody>,
) -> Result<Json<Value>, ApiError> {
    let out = st
        .rt
        .spawn_turn_restricted(&st.spawn_profile, &id, &body.prompt)
        .await
        .map_err(ApiError)?;
    let _ = persist::save_session(st.gateway(), &id).await;
    Ok(Json(out))
}

#[derive(Deserialize)]
struct CancelBody {
    turn_id: String,
}

async fn cancel_turn(
    State(st): State<Arc<WebState>>,
    Json(body): Json<CancelBody>,
) -> Result<Json<Value>, ApiError> {
    let out = invoke(
        st.gateway(),
        "turn",
        "cancel",
        json!({ "turn_id": body.turn_id }),
    )
    .await
    .map_err(ApiError)?;
    Ok(Json(out))
}

async fn event_stream(
    State(st): State<Arc<WebState>>,
) -> Sse<impl stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = st.events_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| {
        ready(match msg {
            Ok(data) => Some(Ok(Event::default().data(data))),
            Err(_) => None,
        })
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
