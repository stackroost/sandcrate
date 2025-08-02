use std::fs;
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