use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value;

use crate::database::{
    PluginRepository, PostgresPluginRepository, CreatePluginRequest, UpdatePluginRequest,
    CreateExecutionRequest, Plugin, PluginExecution, PluginStatus, ExecutionStatus
};

pub struct PluginService {
    repo: Arc<dyn PluginRepository + Send + Sync>,
}

impl PluginService {
    pub fn new(repo: Arc<dyn PluginRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub async fn list_plugins(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Plugin>, Box<dyn std::error::Error + Send + Sync>> {
        self.repo.list_plugins(limit, offset).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    pub async fn get_plugin_by_id(&self, id: Uuid) -> Result<Option<Plugin>, Box<dyn std::error::Error + Send + Sync>> {
        self.repo.get_plugin_by_id(id).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    pub async fn get_plugin_by_filename(&self, filename: &str) -> Result<Option<Plugin>, Box<dyn std::error::Error + Send + Sync>> {
        self.repo.get_plugin_by_filename(filename).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    pub async fn create_plugin(&self, request: CreatePluginRequest) -> Result<Plugin, Box<dyn std::error::Error + Send + Sync>> {
        self.repo.create_plugin(request).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    pub async fn update_plugin(&self, id: Uuid, request: UpdatePluginRequest) -> Result<Plugin, Box<dyn std::error::Error + Send + Sync>> {
        self.repo.update_plugin(id, request).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    pub async fn delete_plugin(&self, id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        self.repo.delete_plugin(id).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    pub async fn record_execution_start(&self, plugin_id: Uuid, user_id: Option<Uuid>, session_id: Option<String>, parameters: Option<Value>) -> Result<PluginExecution, Box<dyn std::error::Error + Send + Sync>> {
        let request = CreateExecutionRequest {
            plugin_id,
            user_id,
            session_id,
            parameters,
        };
        
        self.repo.record_execution(request).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    pub async fn get_execution_history(&self, plugin_id: Uuid, limit: Option<i64>) -> Result<Vec<PluginExecution>, Box<dyn std::error::Error + Send + Sync>> {
        self.repo.get_execution_history(plugin_id, limit).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    pub async fn sync_plugins_from_filesystem(&self, plugins_dir: &str) -> Result<Vec<Plugin>, Box<dyn std::error::Error + Send + Sync>> {
        use std::fs;
        use std::path::Path;
        
        let mut synced_plugins = Vec::new();
        let plugins_path = Path::new(plugins_dir);
        
        if !plugins_path.exists() {
            return Ok(synced_plugins);
        }
        
        for entry in fs::read_dir(plugins_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(extension) = path.extension() {
                if extension == "wasm" {
                    let filename = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    if let Ok(Some(_)) = self.get_plugin_by_filename(&filename).await {
                        continue;
                    }
                    
                    let metadata = fs::metadata(&path)?;
                    let name = filename.replace(".wasm", "");
                    
                    let request = CreatePluginRequest {
                        name: name.clone(),
                        filename,
                        file_path: path.to_string_lossy().to_string(),
                        file_size: metadata.len() as i64,
                        description: Some(format!("Auto-imported plugin: {}", name)),
                        version: "1.0.0".to_string(),
                        author: None,
                        tags: vec!["auto-imported".to_string()],
                    };
                    
                    match self.create_plugin(request).await {
                        Ok(plugin) => synced_plugins.push(plugin),
                        Err(e) => eprintln!("Failed to create plugin {}: {}", name, e),
                    }
                }
            }
        }
        
        Ok(synced_plugins)
    }
} 