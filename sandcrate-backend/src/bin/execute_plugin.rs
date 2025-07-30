use std::fs;
use std::path::Path;
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <plugin_path>", args[0]);
        std::process::exit(1);
    }
    
    let plugin_path = &args[1];
    
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
        eprintln!("No suitable entry function found in WASM module");
        std::process::exit(1);
    }
    
    Ok(())
} 