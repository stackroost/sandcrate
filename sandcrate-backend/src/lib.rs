mod api;
mod auth;
pub mod plugin;
mod websocket;
mod database;
mod services;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use axum::{Router, routing::get};

pub use plugin::run_plugin;
pub use websocket::{WebSocketManager, PluginExecutionSession};
pub use database::{DatabaseConfig, create_pool, PostgresPluginRepository, PluginRepository};
pub use services::PluginService;

#[tokio::main]
pub async fn run_backend() {
    dotenv::dotenv().ok();
    
    let db_config = DatabaseConfig::default();
    let db_pool = create_pool(&db_config).await.expect("Failed to create database pool");
    let plugin_repo = Arc::new(PostgresPluginRepository::new(db_pool.clone()));
    let plugin_service = Arc::new(PluginService::new(plugin_repo.clone()));
    
    let auth_config = Arc::new(auth::AuthConfig::new());
    let ws_manager = Arc::new(websocket::WebSocketManager::new());
    
    let api_router = api::routes().with_state((auth_config.clone(), plugin_service.clone()));
    let auth_router = auth::auth_routes().with_state(auth_config.clone());
    let ws_router = Router::new()
        .route("/plugins", get(websocket::plugin_execution_websocket))
        .with_state((auth_config, ws_manager));
    
    let app = Router::new()
        .nest("/api", api_router)
        .nest("/auth", auth_router)
        .nest("/ws", ws_router)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
