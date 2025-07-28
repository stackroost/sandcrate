use std::env;
use std::path::Path;
use sandcrate_backend::plugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        println!("Usage: {} <plugin_name>", args[0]);
        println!("Example: {} sandcrate-plugin", args[0]);
        return Ok(());
    }
    
    let plugin_name = &args[1];
    let plugin_path = format!("assets/plugins/{}.wasm", plugin_name);
    
    if !Path::new(&plugin_path).exists() {
        println!("❌ Plugin '{}' not found at {}", plugin_name, plugin_path);
        println!("Available plugins:");
        
        let plugins = plugin::list_plugins();
        if plugins.is_empty() {
            println!("  No plugins found in assets/plugins directory");
        } else {
            for plugin in plugins {
                println!("  - {}", plugin);
            }
        }
        return Ok(());
    }
    
    println!("🚀 Executing plugin: {}", plugin_name);
    println!("📁 Path: {}", plugin_path);
    println!("---");
    
    match plugin::run_plugin(&plugin_path) {
        Ok(result) => {
            println!("✅ Plugin executed successfully!");
            println!("📋 Result: {}", result);
        }
        Err(e) => {
            println!("❌ Plugin execution failed: {}", e);
        }
    }
    
    Ok(())
} 