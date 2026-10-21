//! Web UI + HTTP API。

mod routes;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use crate::config::load_spawn_expose;
use crate::driver;
use crate::persist;
use crate::runtime::Runtime;
use axum::Router;
use frame_compose::AppHandle;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::info;

pub use state::WebState;

pub async fn serve(
    app: AppHandle,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let restored = persist::restore_all(&app.gateway).await;
    let (events_tx, _) = broadcast::channel(256);
    driver::spawn_driver(&app.host, events_tx.clone());

    let mut spawn_profile = load_spawn_expose()?;
    if let Some(chat) = app.expose.abilities.get("chat") {
        let entry = spawn_profile.abilities.entry("chat".into()).or_default();
        entry.enabled = Some(true);
        entry.config = chat.config.clone();
    }
    if let Some(turn) = app.expose.abilities.get("turn") {
        let entry = spawn_profile.abilities.entry("turn".into()).or_default();
        entry.enabled = Some(true);
        entry.config = turn.config.clone();
    }
    let state = Arc::new(WebState {
        rt: Runtime::from(app),
        spawn_profile,
        events_tx,
        restored_count: restored,
    });

    let router = Router::new()
        .merge(routes::api_router())
        .merge(routes::static_router())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    info!(%addr, "Agent Test WebUI 已启动，浏览器打开 http://{host}:{port}/");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
