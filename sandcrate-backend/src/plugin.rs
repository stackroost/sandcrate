use std::fs;
use std::path::Path;
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;
use serde_json::Value;
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::io::{Read, Write};



pub fn list_plugins() -> Vec<String> {
    let plugins_dir = Path::new("assets/plugins");

    if !plugins_dir.exists() {
        return vec![];
    }

    match fs::read_dir(plugins_dir) {
        Ok(entries) => entries.filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension().and_then(|e| e.to_str()) == Some("wasm") {
                    path.file_name()?.to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        }).collect(),
        Err(_) => vec![],
    }
}

pub fn run_plugin(plugin_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    run_plugin_with_params(plugin_path, None, None)
}

pub fn run_plugin_with_params(
    plugin_path: &str, 
    parameters: Option<Value>,
    _timeout: Option<u64>
) -> Result<String, Box<dyn std::error::Error>> {
    let engine = Engine::default();
    
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    
    let mut store = Store::new(&engine, wasi);
    
    let wasm_bytes = fs::read(plugin_path)?;
    let module = Module::new(&engine, &wasm_bytes)?;
    
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
    
    let instance = linker.instantiate(&mut store, &module)?;
    
    let function_names = ["_start", "start", "main", "run"];
    
    let mut executed = false;
    let mut result = String::new();
    
    for func_name in &function_names {
        if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, func_name) {
            func.call(&mut store, ())?;
            executed = true;
            result = format!("Plugin executed successfully using function '{}'", func_name);
            break;
        }
    }
    
    if !executed {
        return Err("No suitable entry function found in WASM module".into());
    }
    
    Ok(result)
}

pub async fn run_plugin_with_realtime_output(
    plugin_path: &str,
    _parameters: Option<Value>,
    _timeout: Option<u64>,
    ws_tx: broadcast::Sender<crate::websocket::PluginExecutionSession>,
    session_id: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let session_id = session_id.to_string();
    let plugin_id = Path::new(plugin_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let _ = ws_tx.send(crate::websocket::PluginExecutionSession {
        id: session_id.clone(),
        plugin_id: plugin_id.clone(),
        status: "starting".to_string(),
        output: "Plugin execution started".to_string(),
    });
    
    let output = tokio::process::Command::new("cargo")
        .args(&["run", "--bin", "execute_plugin", "--quiet"])
        .arg(plugin_path)
        .output()
        .await?;
    
    let _ = ws_tx.send(crate::websocket::PluginExecutionSession {
        id: session_id.clone(),
        plugin_id: plugin_id.clone(),
        status: "running".to_string(),
        output: "Executing plugin...".to_string(),
    });
    
    if !output.stdout.is_empty() {
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        for line in stdout_str.lines() {
            if !line.trim().is_empty() {
                let _ = ws_tx.send(crate::websocket::PluginExecutionSession {
                    id: session_id.clone(),
                    plugin_id: plugin_id.clone(),
                    status: "running".to_string(),
                    output: line.trim().to_string(),
                });
            }
        }
    }
    
    if !output.stderr.is_empty() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        for line in stderr_str.lines() {
            if !line.trim().is_empty() {
                let _ = ws_tx.send(crate::websocket::PluginExecutionSession {
                    id: session_id.clone(),
                    plugin_id: plugin_id.clone(),
                    status: "running".to_string(),
                    output: format!("ERROR: {}", line.trim()),
                });
            }
        }
    }
    
    let result = if output.status.success() {
        "Plugin executed successfully".to_string()
    } else {
        format!("Plugin execution failed with exit code: {}", output.status)
    };
    
    let _ = ws_tx.send(crate::websocket::PluginExecutionSession {
        id: session_id.clone(),
        plugin_id: plugin_id.clone(),
        status: "completed".to_string(),
        output: format!("Plugin completed: {}", result),
    });
    
    Ok(result)
}

pub fn get_plugin_info(plugin_path: &str) -> Result<PluginInfo, Box<dyn std::error::Error>> {
    let path = Path::new(plugin_path);
    
    if !path.exists() {
        return Err("Plugin file does not exist".into());
    }
    
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len();
    
    let wasm_bytes = fs::read(path)?;
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes)?;
    
    let exports: Vec<String> = module
        .exports()
        .map(|export| export.name().to_string())
        .collect();
    
    let has_start = exports.contains(&"_start".to_string()) || 
                   exports.contains(&"start".to_string()) ||
                   exports.contains(&"main".to_string()) ||
                   exports.contains(&"run".to_string());
    
    Ok(PluginInfo {
        path: plugin_path.to_string(),
        size: file_size,
        exports,
        has_start,
    })
}

#[derive(Debug)]
pub struct PluginInfo {
    pub path: String,
    pub size: u64,
    pub exports: Vec<String>,
    pub has_start: bool,
}
