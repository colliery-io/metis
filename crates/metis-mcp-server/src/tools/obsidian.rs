use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[mcp_tool(
    name = "open_vault_in_obsidian",
    description = "Open a metis project vault in Obsidian by configuring it as a known vault and launching Obsidian",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OpenVaultInObsidianTool {
    /// Path to the project directory to open as an Obsidian vault
    pub project_path: String,
}

impl OpenVaultInObsidianTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let project_path = PathBuf::from(&self.project_path);

        // Ensure the project path exists
        if !project_path.exists() {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                serde_json::to_string_pretty(&serde_json::json!({
                    "error": format!("Project path does not exist: {}", self.project_path)
                }))
                .map_err(CallToolError::new)?,
            )]));
        }

        // Detect operating system and set appropriate configuration
        let os_info = get_os_info();

        match open_obsidian_vault(&project_path, &os_info) {
            Ok(message) => {
                let response = serde_json::json!({
                    "message": message,
                    "project_path": self.project_path,
                    "os": os_info.name
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to open vault in Obsidian: {}", e)
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}

#[derive(Debug)]
struct OsInfo {
    name: String,
    config_path: PathBuf,
    close_command: Vec<String>,
    open_command: Vec<String>,
}

fn get_os_info() -> OsInfo {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        OsInfo {
            name: "macOS".to_string(),
            config_path: PathBuf::from(home)
                .join("Library/Application Support/obsidian/obsidian.json"),
            close_command: vec![
                "osascript".to_string(),
                "-e".to_string(),
                "quit app \"Obsidian\"".to_string(),
            ],
            open_command: vec!["open".to_string(), "-a".to_string(), "Obsidian".to_string()],
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try multiple possible config locations for different installation methods
        let config_path = get_linux_config_path();
        OsInfo {
            name: "Linux".to_string(),
            config_path,
            close_command: vec!["killall".to_string(), "obsidian".to_string()],
            open_command: vec!["obsidian".to_string()],
        }
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        OsInfo {
            name: "Windows".to_string(),
            config_path: PathBuf::from(appdata).join("Obsidian/obsidian.json"), // Note: capital O
            close_command: vec![
                "taskkill".to_string(),
                "/F".to_string(),
                "/IM".to_string(),
                "Obsidian.exe".to_string(),
            ],
            open_command: get_windows_obsidian_command(),
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        OsInfo {
            name: "Unknown".to_string(),
            config_path: PathBuf::from("obsidian.json"),
            close_command: vec![],
            open_command: vec!["obsidian".to_string()],
        }
    }
}

#[cfg(target_os = "linux")]
fn get_linux_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();

    // Check multiple possible locations based on installation method
    let possible_paths = vec![
        // Standard location
        PathBuf::from(&home).join(".config/obsidian/obsidian.json"),
        // Flatpak location
        PathBuf::from(&home).join(".var/app/md.obsidian.Obsidian/config/obsidian/obsidian.json"),
        // Snap location (pattern - actual path varies by revision)
        // We'll check for this dynamically below
    ];

    // Check standard and Flatpak locations
    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }

    // Check for Snap installation (dynamic revision)
    let snap_base = PathBuf::from(&home).join("snap/obsidian");
    if snap_base.exists() {
        if let Ok(entries) = fs::read_dir(&snap_base) {
            for entry in entries.flatten() {
                let config_path = entry.path().join(".config/obsidian/obsidian.json");
                if config_path.exists() {
                    return config_path;
                }
            }
        }
    }

    // Default to standard location if none found
    PathBuf::from(&home).join(".config/obsidian/obsidian.json")
}

#[cfg(target_os = "windows")]
fn get_windows_obsidian_command() -> Vec<String> {
    // Try to find Obsidian executable in common locations
    let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let possible_paths = vec![
        PathBuf::from(&local_appdata).join("Obsidian/Obsidian.exe"),
        // Could add other possible installation paths here
    ];

    for path in possible_paths {
        if path.exists() {
            return vec![
                "cmd".to_string(),
                "/C".to_string(),
                "start".to_string(),
                "".to_string(),
                path.to_string_lossy().to_string(),
            ];
        }
    }

    // Fallback to hoping it's in PATH or using start with obsidian
    vec![
        "cmd".to_string(),
        "/C".to_string(),
        "start".to_string(),
        "obsidian".to_string(),
    ]
}

fn format_path_for_os(path: &Path, _os_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path_str = path.to_string_lossy().to_string();

    // On Windows, ensure we use backslashes for the JSON config
    // (the JSON library will properly escape them when serializing)
    #[cfg(target_os = "windows")]
    {
        Ok(path_str.replace("/", "\\"))
    }

    // On Unix-like systems, use forward slashes
    #[cfg(not(target_os = "windows"))]
    {
        Ok(path_str)
    }
}

fn open_obsidian_vault(
    project_path: &Path,
    os_info: &OsInfo,
) -> Result<String, Box<dyn std::error::Error>> {
    // Step 1: Close Obsidian if running
    if !os_info.close_command.is_empty() {
        let _ = Command::new(&os_info.close_command[0])
            .args(&os_info.close_command[1..])
            .output(); // Ignore errors - app might not be running

        // Small delay to ensure app closes
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // Step 2: Get absolute path and format appropriately for OS
    let vault_path = project_path.canonicalize()?;
    let vault_path_str = format_path_for_os(&vault_path, &os_info.name)?;

    // Step 3: Initialize .obsidian directory if needed
    let obsidian_dir = vault_path.join(".obsidian");
    if !obsidian_dir.exists() {
        fs::create_dir_all(&obsidian_dir)?;
    }

    // Step 4: Ensure config directory exists
    if let Some(config_dir) = os_info.config_path.parent() {
        fs::create_dir_all(config_dir)?;
    }

    // Step 5: Update Obsidian configuration
    update_obsidian_config(&os_info.config_path, &vault_path_str)?;

    // Step 6: Open Obsidian with the vault
    if !os_info.open_command.is_empty() {
        let mut cmd = Command::new(&os_info.open_command[0]);
        cmd.args(&os_info.open_command[1..]);
        cmd.arg(&vault_path_str);

        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to open Obsidian: {}", stderr).into());
        }
    }

    Ok(format!(
        "Successfully opened vault '{}' in Obsidian",
        vault_path_str
    ))
}

fn update_obsidian_config(
    config_path: &PathBuf,
    vault_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read existing config or create new one
    let mut config: serde_json::Value = if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        if content.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
        }
    } else {
        serde_json::json!({})
    };

    // Ensure vaults object exists
    if !config.is_object() {
        config = serde_json::json!({});
    }

    let config_obj = config.as_object_mut().unwrap();
    if !config_obj.contains_key("vaults") {
        config_obj.insert("vaults".to_string(), serde_json::json!({}));
    }

    // Generate vault ID (16-character hex string based on path)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    vault_path.hash(&mut hasher);
    let hash = hasher.finish();
    let vault_id = format!("{:016x}", hash)[..16].to_string();

    // Add vault entry (check if it already exists)
    let vaults = config_obj
        .get_mut("vaults")
        .unwrap()
        .as_object_mut()
        .unwrap();

    // Only add if vault doesn't already exist
    if !vaults.contains_key(&vault_id) {
        vaults.insert(
            vault_id,
            serde_json::json!({
                "path": vault_path,
                "ts": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                "open": true
            }),
        );
    } else {
        // Update existing entry to mark as open and update timestamp
        if let Some(vault_entry) = vaults.get_mut(&vault_id) {
            if let Some(vault_obj) = vault_entry.as_object_mut() {
                vault_obj.insert("open".to_string(), serde_json::json!(true));
                vault_obj.insert(
                    "ts".to_string(),
                    serde_json::json!(std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64),
                );
            }
        }
    }

    // Write config back
    let config_str = serde_json::to_string_pretty(&config)?;
    let mut file = fs::File::create(config_path)?;
    file.write_all(config_str.as_bytes())?;

    Ok(())
}
