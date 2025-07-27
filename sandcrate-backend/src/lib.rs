mod plugin;
mod api;

use axum::Server;
use std::net::SocketAddr;

#[tokio::main]
pub async fn run_backend() {
    let app = api::routes();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Backend running at http://{}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
