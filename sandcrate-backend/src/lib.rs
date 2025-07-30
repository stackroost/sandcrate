mod api;
mod auth;
pub mod plugin;
mod websocket;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use axum::{Router, routing::get};

pub use plugin::run_plugin;
pub use websocket::{WebSocketManager, PluginExecutionSession};

#[tokio::main]
pub async fn run_backend() {
    let auth_config = Arc::new(auth::AuthConfig::new());
    let ws_manager = Arc::new(websocket::WebSocketManager::new());
    
    let api_router = api::routes().with_state(auth_config.clone());
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
    println!("Backend running at http://{}", addr);
    println!("API endpoints:");
    println!("  GET  http://{}/api/plugins", addr);
    println!("  POST http://{}/auth/login", addr);
    println!("WebSocket endpoints:");
    println!("  WS   ws://{}/ws/plugins", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
