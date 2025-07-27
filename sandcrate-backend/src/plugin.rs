use std::fs;
use std::path::Path;
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;

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
    for func_name in &function_names {
        if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, func_name) {
            func.call(&mut store, ())?;
            executed = true;
            break;
        }
    }
    
    if !executed {
        return Err("No suitable entry function found in WASM module".into());
    }
    
    Ok("Plugin executed successfully".to_string())
}
