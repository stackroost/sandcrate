use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::plugin::list_plugins;

#[derive(Serialize)]
struct PluginList {
    plugins: Vec<String>,
}

async fn get_plugins() -> Json<PluginList> {
    let plugins = list_plugins();
    Json(PluginList { plugins })
}

pub fn routes() -> Router {
    Router::new().route("/plugins", get(get_plugins))
}
