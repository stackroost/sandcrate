use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::auth::AuthConfig;
use crate::plugin;

#[derive(Debug, Clone)]
pub struct PluginExecutionSession {
    pub id: String,
    pub plugin_id: String,
    pub status: String,
    pub output: String,
}

pub struct WebSocketManager {
    tx: broadcast::Sender<PluginExecutionSession>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn get_sender(&self) -> broadcast::Sender<PluginExecutionSession> {
        self.tx.clone()
    }
}

pub async fn plugin_execution_websocket(
    ws: WebSocketUpgrade,
    State((state, ws_manager)): State<(Arc<AuthConfig>, Arc<WebSocketManager>)>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_plugin_execution_socket(socket, state, ws_manager))
}

async fn handle_plugin_execution_socket(
    mut socket: WebSocket,
    _state: Arc<AuthConfig>,
    ws_manager: Arc<WebSocketManager>,
) {
    let mut rx = ws_manager.get_sender().subscribe();
    
    let connect_msg = json!({
        "type": "connected",
        "message": "WebSocket connected successfully"
    });
    
    if let Err(_) = socket.send(Message::Text(connect_msg.to_string())).await {
        return;
    }

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(command) = data.get("command").and_then(|c| c.as_str()) {
                                match command {
                                    "execute_plugin" => {
                                        if let (Some(plugin_id), parameters, timeout) = (
                                            data.get("plugin_id").and_then(|p| p.as_str()),
                                            data.get("parameters").cloned(),
                                            data.get("timeout").and_then(|t| t.as_u64()),
                                        ) {
                                            let session_id = Uuid::new_v4().to_string();
                                            
                                            let initial_status = json!({
                                                "type": "status",
                                                "session_id": session_id,
                                                "plugin_id": plugin_id,
                                                "status": "starting",
                                                "message": "Plugin execution started"
                                            });
                                            
                                            if let Err(_) = socket.send(Message::Text(initial_status.to_string())).await {
                                                break;
                                            }

                                            let ws_tx = ws_manager.get_sender();
                                            let plugin_id = plugin_id.to_string();
                                            
                                            tokio::spawn(async move {
                                                let result = plugin::run_plugin_with_realtime_output(
                                                    &format!("assets/plugins/{}.wasm", plugin_id),
                                                    parameters,
                                                    timeout,
                                                    ws_tx.clone(),
                                                    &session_id,
                                                ).await;
                                                
                                                let final_message = match result {
                                                    Ok(output) => json!({
                                                        "type": "result",
                                                        "session_id": session_id,
                                                        "plugin_id": plugin_id,
                                                        "status": "completed",
                                                        "output": output,
                                                        "success": true
                                                    }),
                                                    Err(e) => json!({
                                                        "type": "result",
                                                        "session_id": session_id,
                                                        "plugin_id": plugin_id,
                                                        "status": "error",
                                                        "error": e.to_string(),
                                                        "success": false
                                                    }),
                                                };
                                                
                                                let _ = ws_tx.send(PluginExecutionSession {
                                                    id: session_id,
                                                    plugin_id,
                                                    status: "completed".to_string(),
                                                    output: final_message.to_string(),
                                                });
                                            });
                                        }
                                    }
                                    "subscribe" => {
                                        if let Some(session_id) = data.get("session_id").and_then(|s| s.as_str()) {
                                            let subscribe_msg = json!({
                                                "type": "subscribed",
                                                "session_id": session_id,
                                                "message": "Subscribed to session updates"
                                            });
                                            
                                            if let Err(_) = socket.send(Message::Text(subscribe_msg.to_string())).await {
                                                break;
                                            }
                                        }
                                    }
                                    _ => {
                                        let error_msg = json!({
                                            "type": "error",
                                            "message": format!("Unknown command: {}", command)
                                        });
                                        
                                        if let Err(_) = socket.send(Message::Text(error_msg.to_string())).await {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Err(_)) => break,
                    None => break,
                    _ => continue,
                }
            }
            
            session_result = rx.recv() => {
                match session_result {
                    Ok(session) => {
                        let message = json!({
                            "type": "update",
                            "session_id": session.id,
                            "plugin_id": session.plugin_id,
                            "status": session.status,
                            "output": session.output
                        });
                        
                        if let Err(_) = socket.send(Message::Text(message.to_string())).await {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }
} 