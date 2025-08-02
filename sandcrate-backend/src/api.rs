use axum::{
    routing::{get, post, delete},
    Json, Router, extract::{State, Path, Multipart},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::{
    extract::TypedHeader,
    headers::{authorization::Bearer, Authorization},
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::fs;
use std::path::Path as FsPath;

use crate::auth::{AuthConfig, validate_token};
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

#[derive(Deserialize)]
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



async fn get_plugins(
    State(config): State<Arc<AuthConfig>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ApiResponse<PluginList>>, (StatusCode, Json<ApiResponse<PluginList>>)> {
    let _user = validate_token(State(config.clone()), TypedHeader(Authorization(bearer))).await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid or missing authentication token".to_string()),
                })
            )
        })?;
    let plugins_dir = FsPath::new("../assets/plugins");
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
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(PluginList { plugins }),
        error: None,
    }))
}

async fn get_plugin(
    State(config): State<Arc<AuthConfig>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Path(plugin_id): Path<String>,
) -> Result<Response, (StatusCode, Json<ApiResponse<Plugin>>)> {
    let _user = validate_token(State(config.clone()), TypedHeader(Authorization(bearer))).await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid or missing authentication token".to_string()),
                })
            )
        })?;
    let plugins_dir = FsPath::new("../assets/plugins");
    let plugin_path = plugins_dir.join(format!("{}.wasm", plugin_id));
    
    if !plugin_path.exists() {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<Plugin> {
                success: false,
                data: None,
                error: Some(format!("Plugin '{}' not found", plugin_id)),
            })
        ).into_response());
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
        
        Ok((
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some(plugin),
                error: None,
            })
        ).into_response())
    } else {
        Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<Plugin> {
                success: false,
                data: None,
                error: Some("Failed to read plugin metadata".to_string()),
            })
        ).into_response())
    }
}

async fn execute_plugin(
    State(config): State<Arc<AuthConfig>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Path(plugin_id): Path<String>,
    Json(request): Json<PluginExecutionRequest>,
) -> Result<Response, (StatusCode, Json<ApiResponse<PluginExecutionResponse>>)> {
    let _user = validate_token(State(config.clone()), TypedHeader(Authorization(bearer))).await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid or missing authentication token".to_string()),
                })
            )
        })?;
    let start_time = std::time::Instant::now();
    
    let plugins_dir = FsPath::new("../assets/plugins");
    let plugin_path = plugins_dir.join(format!("{}.wasm", plugin_id));
    
    if !plugin_path.exists() {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<PluginExecutionResponse> {
                success: false,
                data: None,
                error: Some(format!("Plugin '{}' not found", plugin_id)),
            })
        ).into_response());
    }
    
    let parameters = request.parameters;
    let timeout = request.timeout;
    
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
                
                Ok((
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        data: Some(response),
                        error: None,
                    })
                ).into_response())
            }
            Err(e) => {
                let response = PluginExecutionResponse {
                    success: false,
                    result: String::new(),
                    execution_time_ms,
                    error: Some(e.to_string()),
                };
                
                Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        success: true,
                        data: Some(response),
                        error: None,
                    })
                ).into_response())
            }
        }
}

async fn upload_plugin(
    State(config): State<Arc<AuthConfig>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<String>>)> {
    let _user = validate_token(State(config.clone()), TypedHeader(Authorization(bearer))).await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid or missing authentication token".to_string()),
                })
            )
        })?;

    while let Some(field) = multipart.next_field().await.map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Failed to read multipart data".to_string()),
            })
        )
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name == "plugin" {
            let data = field.bytes().await.map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        data: None,
                        error: Some("Failed to read plugin file".to_string()),
                    })
                )
            })?;
            
            let filename = format!("plugin_{}.wasm", uuid::Uuid::new_v4());
            let plugin_path = FsPath::new("../assets/plugins").join(&filename);
            
            if let Err(_) = fs::write(&plugin_path, data) {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        success: false,
                        data: None,
                        error: Some("Failed to save plugin file".to_string()),
                    })
                ));
            }
            
            return Ok(Json(ApiResponse {
                success: true,
                data: Some(filename),
                error: None,
            }));
        }
    }
    
    Err((
        StatusCode::BAD_REQUEST,
        Json(ApiResponse {
            success: false,
            data: None,
            error: Some("No plugin file found in request".to_string()),
        })
    ))
}

async fn delete_plugin(
    State(config): State<Arc<AuthConfig>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Path(plugin_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<String>>)> {
    let _user = validate_token(State(config.clone()), TypedHeader(Authorization(bearer))).await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid or missing authentication token".to_string()),
                })
            )
        })?;

    let plugins_dir = FsPath::new("../assets/plugins");
    let plugin_path = plugins_dir.join(format!("{}.wasm", plugin_id));
    
    if !plugin_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Plugin '{}' not found", plugin_id)),
            })
        ));
    }
    
    if let Err(_) = fs::remove_file(&plugin_path) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Failed to delete plugin file".to_string()),
            })
        ));
    }
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(format!("Plugin '{}' deleted successfully", plugin_id)),
        error: None,
    }))
}

pub fn routes() -> Router<Arc<AuthConfig>> {
    Router::new()
        .route("/plugins", get(get_plugins))
        .route("/plugins/upload", post(upload_plugin))
        .route("/plugins/:id", get(get_plugin))
        .route("/plugins/:id", delete(delete_plugin))
        .route("/plugins/:id/execute", post(execute_plugin))
}
