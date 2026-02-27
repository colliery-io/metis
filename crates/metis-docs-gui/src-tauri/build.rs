use std::fs;
use std::path::Path;

fn main() {
    // Sync version from Cargo.toml to tauri.conf.json
    sync_version_to_tauri_config();

    tauri_build::build()
}

fn sync_version_to_tauri_config() {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let tauri_config_path = "tauri.conf.json";

    if !Path::new(tauri_config_path).exists() {
        println!("cargo:warning=tauri.conf.json not found, skipping version sync");
        return;
    }

    let config_content = match fs::read_to_string(tauri_config_path) {
        Ok(content) => content,
        Err(e) => {
            println!("cargo:warning=Failed to read tauri.conf.json: {}", e);
            return;
        }
    };

    let mut config: serde_json::Value = match serde_json::from_str(&config_content) {
        Ok(config) => config,
        Err(e) => {
            println!("cargo:warning=Failed to parse tauri.conf.json: {}", e);
            return;
        }
    };

    // Update version in config
    if let Some(version) = config.get_mut("version") {
        let current_version = version.as_str().unwrap_or("");

        // Only write if version has changed
        if current_version != cargo_version {
            *version = serde_json::Value::String(cargo_version.to_string());

            // Write back to file
            let updated_content = match serde_json::to_string_pretty(&config) {
                Ok(content) => content,
                Err(e) => {
                    println!("cargo:warning=Failed to serialize tauri.conf.json: {}", e);
                    return;
                }
            };

            if let Err(e) = fs::write(tauri_config_path, updated_content) {
                println!("cargo:warning=Failed to write tauri.conf.json: {}", e);
            } else {
                println!(
                    "cargo:warning=Synced version {} to tauri.conf.json",
                    cargo_version
                );
            }
        }
    }
}
