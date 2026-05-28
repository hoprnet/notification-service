use axum::{
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

mod config;
mod formatter;
mod message_store;
mod models;
mod output;
mod processing;
mod routes;

pub use config::Config;
pub use message_store::MessageStore;

/// Shared application state injected into every Axum handler via [`axum::extract::State`].
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub messages: MessageStore,
}

#[tokio::main]
async fn main() {
    // Initialise structured logging. Level is controlled by the RUST_LOG env var
    // (default: info). Example: RUST_LOG=notification_service=debug,info
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Arc::new(Config::from_env());

    tracing::info!(
        port             = config.port,
        zulip_host       = %config.zulip_host,
        zulip_email      = %config.zulip_email,
        zulip_configured = config.zulip_configured(),
        "Starting notification-service"
    );

    let messages = message_store::new();

    message_store::spawn_eviction_task(
        messages.clone(),
        Duration::from_secs(config.message_ttl_days * 24 * 3600),
        Duration::from_secs(3600), // sweep every hour
    );

    let state = AppState {
        config: config.clone(),
        messages,
    };

    let app = Router::new()
        .route("/", get(health_check))
        .route("/healthz", get(health_check))
        .route("/alerts", post(routes::receive_alert))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server error");
}

async fn health_check() -> &'static str {
    "OK"
}
