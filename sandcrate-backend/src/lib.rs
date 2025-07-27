mod api;
mod auth;
mod plugin;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use axum::Router;

// Re-export plugin runner
pub use plugin::run_plugin;

#[tokio::main]
pub async fn run_backend() {
    let auth_config = Arc::new(auth::AuthConfig::new());
    
    let app = Router::new()
        .nest("/api", api::routes())
        .nest("/auth", auth::auth_routes())
        .with_state(auth_config)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Backend running at http://{}", addr);
    println!("API endpoints:");
    println!("  GET  http://{}/api/plugins", addr);
    println!("  POST http://{}/auth/login", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
