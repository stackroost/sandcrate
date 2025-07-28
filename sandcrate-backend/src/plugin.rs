use std::fs;
use std::path::Path;
use std::time::Duration;
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;
use serde_json::Value;

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
    // Create a WASM engine
    let engine = Engine::default();
    
    // Create a store with WASI context
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    
    let mut store = Store::new(&engine, wasi);
    
    // Read the WASM module
    let wasm_bytes = fs::read(plugin_path)?;
    let module = Module::new(&engine, &wasm_bytes)?;
    
    // Create a linker and add WASI
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
    
    // Instantiate the module
    let instance = linker.instantiate(&mut store, &module)?;
    
    // Try to find and call a suitable function
    // Common function names for WASM modules: _start, start, main, run
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

pub fn get_plugin_info(plugin_path: &str) -> Result<PluginInfo, Box<dyn std::error::Error>> {
    let path = Path::new(plugin_path);
    
    if !path.exists() {
        return Err("Plugin file does not exist".into());
    }
    
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len();
    
    // Try to read WASM module info
    let wasm_bytes = fs::read(path)?;
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes)?;
    
    // Get exported functions from module
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
