use axum::{
    routing::{get, post},
    Json, Router, extract::{State, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::sync::Arc;
use std::fs;
use std::path::Path as FsPath;

use crate::auth::AuthConfig;
use crate::plugin;

#[derive(Serialize)]
struct Plugin {
    id: String,
    name: String,
    filename: String,
    size: u64,
    created_at: String,
    status: String,
}

#[derive(Serialize)]
struct PluginList {
    plugins: Vec<Plugin>,
}

#[derive(Serialize)]
struct PluginExecutionRequest {
    parameters: Option<serde_json::Value>,
    timeout: Option<u64>,
}

#[derive(Serialize)]
struct PluginExecutionResponse {
    success: bool,
    result: String,
    execution_time_ms: u64,
    error: Option<String>,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
    code: String,
}

async fn get_plugins(
    State(_config): State<Arc<AuthConfig>>,
) -> Json<ApiResponse<PluginList>> {
    let plugins_dir = FsPath::new("assets/plugins");
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
                                status: "ready".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    Json(ApiResponse {
        success: true,
        data: Some(PluginList { plugins }),
        error: None,
    })
}

async fn get_plugin(
    State(_config): State<Arc<AuthConfig>>,
    Path(plugin_id): Path<String>,
) -> Response {
    let plugins_dir = FsPath::new("assets/plugins");
    let plugin_path = plugins_dir.join(format!("{}.wasm", plugin_id));
    
    if !plugin_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<Plugin> {
                success: false,
                data: None,
                error: Some(format!("Plugin '{}' not found", plugin_id)),
            })
        ).into_response();
    }
    
    if let Ok(metadata) = fs::metadata(&plugin_path) {
        let filename = plugin_path.file_name()
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
        
        let plugin = Plugin {
            id,
            name,
            filename,
            size: metadata.len(),
            created_at,
            status: "ready".to_string(),
        };
        
        (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some(plugin),
                error: None,
            })
        ).into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<Plugin> {
                success: false,
                data: None,
                error: Some("Failed to read plugin metadata".to_string()),
            })
        ).into_response()
    }
}

async fn execute_plugin(
    State(_config): State<Arc<AuthConfig>>,
    Path(plugin_id): Path<String>,
    Json(request): Json<serde_json::Value>,
) -> Response {
    let start_time = std::time::Instant::now();
    
    // Find the plugin file
    let plugins_dir = FsPath::new("assets/plugins");
    let plugin_path = plugins_dir.join(format!("{}.wasm", plugin_id));
    
    if !plugin_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<PluginExecutionResponse> {
                success: false,
                data: None,
                error: Some(format!("Plugin '{}' not found", plugin_id)),
            })
        ).into_response();
    }
    
    // Extract parameters from the request
    let parameters = request.get("parameters").cloned();
    let timeout = request.get("timeout").and_then(|v| v.as_u64());
    
    // Execute the plugin with parameters and timeout
    let plugin_path_str = plugin_path.to_str().unwrap_or("");
    let execution_result = plugin::run_plugin_with_params(
        plugin_path_str,
        parameters,
        timeout
    );
    
    let execution_time = start_time.elapsed();
    let execution_time_ms = execution_time.as_millis() as u64;
    
    match execution_result {
        Ok(result) => {
            let response = PluginExecutionResponse {
                success: true,
                result,
                execution_time_ms,
                error: None,
            };
            
            (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    data: Some(response),
                    error: None,
                })
            ).into_response()
        }
        Err(e) => {
            let response = PluginExecutionResponse {
                success: false,
                result: String::new(),
                execution_time_ms,
                error: Some(e.to_string()),
            };
            
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: true,
                    data: Some(response),
                    error: None,
                })
            ).into_response()
        }
    }
}

pub fn routes() -> Router<Arc<AuthConfig>> {
    Router::new()
        .route("/plugins", get(get_plugins))
        .route("/plugins/:id", get(get_plugin))
        .route("/plugins/:id/execute", post(execute_plugin))
}
