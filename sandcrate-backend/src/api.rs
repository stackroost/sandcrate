use axum::{routing::get, Json, Router, extract::State};
use serde::Serialize;
use std::sync::Arc;
use std::fs;
use std::path::Path;

use crate::auth::AuthConfig;

#[derive(Serialize)]
struct Plugin {
    id: String,
    name: String,
    filename: String,
    size: u64,
    created_at: String,
}

#[derive(Serialize)]
struct PluginList {
    plugins: Vec<Plugin>,
}

async fn get_plugins(
    State(_config): State<Arc<AuthConfig>>,
) -> Json<PluginList> {
    let plugins_dir = Path::new("../assets/plugins");
    let mut plugins = Vec::new();
    
    if let Ok(entries) = fs::read_dir(plugins_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "wasm" {
                        if let Ok(metadata) = fs::metadata(&path) {
                            let filename = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            
                            let name = filename.replace(".wasm", "");
                            let id = name.clone();
                            
                            let created_at = metadata.created()
                                .ok()
                                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|duration| {
                                    chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                                        .unwrap_or_default()
                                        .format("%Y-%m-%d %H:%M:%S")
                                        .to_string()
                                })
                                .unwrap_or_else(|| "Unknown".to_string());
                            
                            plugins.push(Plugin {
                                id,
                                name,
                                filename,
                                size: metadata.len(),
                                created_at,
                            });
                        }
                    }
                }
            }
        }
    }
    
    Json(PluginList { plugins })
}

pub fn routes() -> Router<Arc<AuthConfig>> {
    Router::new().route("/plugins", get(get_plugins))
}
