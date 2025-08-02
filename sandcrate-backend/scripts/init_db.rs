use std::env;
use dotenv::dotenv;

use sandcrate_backend::{
    DatabaseConfig, create_pool, PostgresPluginRepository, PluginService
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    println!("Initializing Sandcrate database...");
    
    let db_config = DatabaseConfig::default();
    let db_pool = create_pool(&db_config).await?;
    
    println!("Database connection established");
    
    let plugin_repo = PostgresPluginRepository::new(db_pool.clone());
    let plugin_service = PluginService::new(Box::new(plugin_repo));
    
    let plugins_dir = env::var("PLUGINS_DIR").unwrap_or_else(|_| "../assets/plugins".to_string());
    println!("Syncing plugins from: {}", plugins_dir);
    
    match plugin_service.sync_plugins_from_filesystem(&plugins_dir).await {
        Ok(plugins) => {
            println!("Synced {} plugins from filesystem", plugins.len());
            for plugin in plugins {
                println!("  - {} (v{})", plugin.name, plugin.version);
            }
        }
        Err(e) => {
            println!("Failed to sync plugins: {}", e);
        }
    }
    
    println!("\nAll plugins in database:");
    match plugin_service.list_plugins(None, None).await {
        Ok(plugins) => {
            for plugin in plugins {
                println!("  - {} (v{}) - {}", plugin.name, plugin.version, plugin.status);
            }
        }
        Err(e) => {
            println!("Failed to list plugins: {}", e);
        }
    }
    
    println!("\nDatabase initialization completed!");
    Ok(())
} 