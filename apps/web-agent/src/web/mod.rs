//! Web UI + HTTP API。

mod routes;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use frame_compose::AppHandle;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::driver;
use crate::persist;
use crate::runtime::Runtime;

pub use state::WebState;

pub async fn serve(
    app: AppHandle,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let restored = persist::restore_all(&app.gateway).await;
    let (events_tx, _) = broadcast::channel(256);
    driver::spawn_driver(&app, events_tx.clone());

    let state = Arc::new(WebState {
        rt: Runtime::from(app),
        events_tx,
        restored_count: restored,
    });

    let router = Router::new()
        .merge(routes::api_router())
        .merge(routes::static_router())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    info!(%addr, "web-agent 已启动，打开 http://{host}:{port}/");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
