use std::fs;
use std::path::Path;

pub fn list_plugins() -> Vec<String> {
    let plugins_dir = Path::new("assets/plugins");

    if !plugins_dir.exists() {
        return vec![];
    }

    fs::read_dir(plugins_dir)
        .unwrap_or_else(|_| fs::ReadDir::from(vec![]))
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension().and_then(|e| e.to_str()) == Some("wasm") {
                    path.file_name()?.to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .collect()
}
