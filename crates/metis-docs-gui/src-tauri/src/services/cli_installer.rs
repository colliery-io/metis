//! CLI Installer Service
//!
//! Handles automatic installation of the Metis CLI binary to the user's PATH.
//! On first launch (or when an update is needed), this service:
//! 1. Copies the bundled CLI binary to a persistent location in app data
//! 2. Creates a symlink in /usr/local/bin (macOS/Linux) or updates PATH (Windows)
//! 3. Tracks installed version for upgrade detection

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Emitter, Manager};

/// Status of CLI installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliInstallStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub binary_path: Option<String>,
    pub symlink_path: Option<String>,
    pub needs_update: bool,
}

/// Result of CLI installation attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliInstallResult {
    pub success: bool,
    pub message: String,
    pub needs_elevation: bool,
}

/// Version tracking info stored in app data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CliVersionInfo {
    version: String,
    installed_at: String,
    binary_path: String,
}

/// Get the app data directory for CLI storage
fn get_cli_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("io.colliery.metis"))
}

/// Get the CLI binary destination path within app data
fn get_cli_binary_path() -> Option<PathBuf> {
    get_cli_data_dir().map(|d| {
        #[cfg(windows)]
        {
            d.join("bin").join("metis.exe")
        }
        #[cfg(not(windows))]
        {
            d.join("bin").join("metis")
        }
    })
}

/// Get the symlink location for PATH integration
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn get_symlink_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".local").join("bin").join("metis"))
}

#[cfg(target_os = "windows")]
fn get_symlink_path() -> Option<PathBuf> {
    None // Windows uses PATH modification instead of symlinks
}

/// Get the version info file path
fn get_version_info_path() -> Option<PathBuf> {
    get_cli_data_dir().map(|d| d.join("cli-version.json"))
}

/// Read current installed CLI version info
fn read_version_info() -> Option<CliVersionInfo> {
    let path = get_version_info_path()?;
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Write CLI version info after installation
fn write_version_info(version: &str, binary_path: &PathBuf) -> Result<(), String> {
    let info = CliVersionInfo {
        version: version.to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        binary_path: binary_path.to_string_lossy().to_string(),
    };

    let path = get_version_info_path().ok_or("Failed to determine version info path")?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(&info)
        .map_err(|e| format!("Failed to serialize version info: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("Failed to write version info: {}", e))
}

/// Check CLI installation status
#[tauri::command]
pub fn get_cli_install_status() -> Result<CliInstallStatus, String> {
    let app_version = env!("CARGO_PKG_VERSION");
    let binary_path = get_cli_binary_path();
    let symlink_path = get_symlink_path();

    let version_info = read_version_info();

    let installed = binary_path
        .as_ref()
        .map(|p| p.exists())
        .unwrap_or(false);

    let needs_update = if installed {
        version_info
            .as_ref()
            .map(|v| v.version != app_version)
            .unwrap_or(true)
    } else {
        false
    };

    Ok(CliInstallStatus {
        installed,
        version: version_info.map(|v| v.version),
        binary_path: binary_path.map(|p| p.to_string_lossy().to_string()),
        symlink_path: symlink_path.map(|p| p.to_string_lossy().to_string()),
        needs_update,
    })
}

/// Get the path to the bundled sidecar binary
fn get_sidecar_path(app: &AppHandle) -> Result<PathBuf, String> {
    // Tauri stores sidecar binaries adjacent to the main executable
    let exe_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    #[cfg(target_os = "macos")]
    {
        // On macOS, the binary is in Contents/MacOS/
        let macos_dir = exe_dir
            .parent()
            .and_then(|p| Some(p.join("MacOS")))
            .ok_or("Failed to find MacOS directory")?;

        let sidecar = macos_dir.join("metis");
        if sidecar.exists() {
            return Ok(sidecar);
        }

        // Fallback: check in Resources
        let resources_sidecar = exe_dir.join("metis");
        if resources_sidecar.exists() {
            return Ok(resources_sidecar);
        }

        Err(format!(
            "Sidecar not found at {:?} or {:?}",
            sidecar, resources_sidecar
        ))
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On Linux/Windows, check adjacent to resources or in external-bin
        let sidecar = exe_dir.join("metis");
        if sidecar.exists() {
            return Ok(sidecar);
        }

        #[cfg(windows)]
        {
            let sidecar_exe = exe_dir.join("metis.exe");
            if sidecar_exe.exists() {
                return Ok(sidecar_exe);
            }
        }

        Err(format!("Sidecar not found at {:?}", sidecar))
    }
}

/// Internal installation function - copies binary and attempts symlink
pub async fn install_cli_internal(app: &AppHandle) -> Result<CliInstallResult, String> {
    let app_version = env!("CARGO_PKG_VERSION");

    // Get the bundled sidecar path
    let sidecar_path = get_sidecar_path(app)?;

    let target_path =
        get_cli_binary_path().ok_or("Failed to determine CLI installation path")?;

    // Create target directory
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create CLI directory: {}", e))?;
    }

    // Copy the binary
    fs::copy(&sidecar_path, &target_path)
        .map_err(|e| format!("Failed to copy CLI binary: {}", e))?;

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_path)
            .map_err(|e| format!("Failed to read permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms)
            .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
    }

    // Fix macOS security attributes and re-sign
    #[cfg(target_os = "macos")]
    {
        let path_str = target_path.to_string_lossy();
        // Remove quarantine and provenance attributes
        Command::new("xattr")
            .args(["-rd", "com.apple.quarantine", &path_str])
            .output()
            .ok();
        Command::new("xattr")
            .args(["-d", "com.apple.provenance", &path_str])
            .output()
            .ok();
        // Re-sign with adhoc signature to fix any signature issues
        Command::new("codesign")
            .args(["--force", "--sign", "-", &path_str])
            .output()
            .ok();
    }

    // Try to create symlink for PATH integration
    #[cfg(not(windows))]
    {
        if let Some(symlink_path) = get_symlink_path() {
            // Ensure parent directory exists
            if let Some(parent) = symlink_path.parent() {
                fs::create_dir_all(parent).ok();
            }

            // Remove existing symlink if present
            if symlink_path.exists() || symlink_path.is_symlink() {
                fs::remove_file(&symlink_path).ok();
            }

            // Try to create symlink
            #[cfg(unix)]
            match std::os::unix::fs::symlink(&target_path, &symlink_path) {
                Ok(_) => {
                    // Success - record version and return
                    write_version_info(app_version, &target_path)?;
                    return Ok(CliInstallResult {
                        success: true,
                        message: format!(
                            "CLI v{} installed to {}. Ensure ~/.local/bin is in your PATH.",
                            app_version,
                            symlink_path.display()
                        ),
                        needs_elevation: false,
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    // Need elevated permissions
                    write_version_info(app_version, &target_path)?;
                    return Ok(CliInstallResult {
                        success: false,
                        message: format!(
                            "CLI copied to {}. Need permissions to create symlink.",
                            target_path.display()
                        ),
                        needs_elevation: true,
                    });
                }
                Err(e) => {
                    log::warn!("Failed to create symlink: {}", e);
                    write_version_info(app_version, &target_path)?;
                    return Ok(CliInstallResult {
                        success: true,
                        message: format!(
                            "CLI installed to {}. Add ~/.local/bin to PATH or run with full path.",
                            target_path.display()
                        ),
                        needs_elevation: false,
                    });
                }
            }
        }
    }

    // Windows: Add to PATH
    #[cfg(windows)]
    {
        if let Some(bin_dir) = target_path.parent() {
            match add_to_windows_path(bin_dir) {
                Ok(_) => {
                    write_version_info(app_version, &target_path)?;
                    return Ok(CliInstallResult {
                        success: true,
                        message: format!(
                            "CLI v{} installed. Restart your terminal to use 'metis' command.",
                            app_version
                        ),
                        needs_elevation: false,
                    });
                }
                Err(e) => {
                    log::warn!("Failed to update PATH: {}", e);
                    write_version_info(app_version, &target_path)?;
                    return Ok(CliInstallResult {
                        success: true,
                        message: format!(
                            "CLI installed at {:?}. Add {:?} to your PATH manually.",
                            target_path, bin_dir
                        ),
                        needs_elevation: false,
                    });
                }
            }
        }
    }

    // Fallback: just record version
    write_version_info(app_version, &target_path)?;
    Ok(CliInstallResult {
        success: true,
        message: format!(
            "CLI v{} installed to {}. Add to PATH to use 'metis' command.",
            app_version,
            target_path.display()
        ),
        needs_elevation: false,
    })
}

/// Install CLI with user-level permissions
#[tauri::command]
pub async fn install_cli(app: AppHandle) -> Result<CliInstallResult, String> {
    install_cli_internal(&app).await
}

/// Install CLI with elevated permissions (creates symlink in /usr/local/bin)
#[tauri::command]
pub async fn install_cli_elevated(app: AppHandle) -> Result<CliInstallResult, String> {
    let app_version = env!("CARGO_PKG_VERSION");

    // First ensure the binary is copied
    let target_path =
        get_cli_binary_path().ok_or("Failed to determine CLI installation path")?;

    if !target_path.exists() {
        // Need to copy first
        install_cli_internal(&app).await?;
    }

    #[cfg(target_os = "macos")]
    let result = {
        // On macOS, symlink goes to ~/.local/bin which doesn't need elevation
        // Just use the regular install process
        // Note: Users should ensure ~/.local/bin is in their PATH
        install_cli_internal(&app).await
    };

    #[cfg(target_os = "linux")]
    let result = {
        // On Linux, symlink goes to ~/.local/bin which doesn't need elevation
        // Just use the regular install process
        // Note: Users should ensure ~/.local/bin is in their PATH
        install_cli_internal(&app).await
    };

    #[cfg(windows)]
    let result = {
        // Windows doesn't need elevation for user PATH
        Ok(CliInstallResult {
            success: true,
            message: "CLI installed (elevation not needed on Windows)".to_string(),
            needs_elevation: false,
        })
    };

    // Emit event for successful installation
    if let Ok(ref install_result) = result {
        if install_result.success {
            app.emit("cli-installed", &install_result.message).ok();
        }
    }

    result
}

/// Uninstall CLI - remove binary and symlink
#[tauri::command]
pub fn uninstall_cli() -> Result<(), String> {
    // Remove binary
    if let Some(binary_path) = get_cli_binary_path() {
        if binary_path.exists() {
            fs::remove_file(&binary_path)
                .map_err(|e| format!("Failed to remove CLI binary: {}", e))?;
        }
    }

    // Remove symlink (best effort)
    #[cfg(not(windows))]
    if let Some(symlink_path) = get_symlink_path() {
        if symlink_path.is_symlink() || symlink_path.exists() {
            fs::remove_file(&symlink_path).ok();
        }
    }

    // Remove version info
    if let Some(version_path) = get_version_info_path() {
        fs::remove_file(&version_path).ok();
    }

    Ok(())
}

/// Add a directory to Windows user PATH via registry
#[cfg(windows)]
fn add_to_windows_path(bin_dir: &std::path::Path) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| format!("Failed to open registry: {}", e))?;

    let current_path: String = env_key.get_value("Path").unwrap_or_default();
    let bin_dir_str = bin_dir.to_string_lossy();

    if !current_path.contains(&*bin_dir_str) {
        let new_path = if current_path.is_empty() {
            bin_dir_str.to_string()
        } else {
            format!("{};{}", current_path, bin_dir_str)
        };

        env_key
            .set_value("Path", &new_path)
            .map_err(|e| format!("Failed to update PATH: {}", e))?;
    }

    Ok(())
}

/// Run auto-installation on app startup
pub async fn auto_install_cli(app: AppHandle) {
    match get_cli_install_status() {
        Ok(status) => {
            if !status.installed || status.needs_update {
                let action = if status.needs_update {
                    "Updating"
                } else {
                    "Installing"
                };
                log::info!("{} CLI...", action);

                match install_cli_internal(&app).await {
                    Ok(result) => {
                        if result.success {
                            log::info!("CLI installation successful: {}", result.message);
                            app.emit("cli-installed", result.message).ok();
                        } else if result.needs_elevation {
                            log::info!(
                                "CLI copied but needs elevation for PATH: {}",
                                result.message
                            );
                            // Attempt elevated install
                            match install_cli_elevated(app.clone()).await {
                                Ok(elevated_result) => {
                                    if elevated_result.success {
                                        log::info!(
                                            "CLI elevated install successful: {}",
                                            elevated_result.message
                                        );
                                    } else {
                                        log::warn!(
                                            "CLI elevated install incomplete: {}",
                                            elevated_result.message
                                        );
                                    }
                                }
                                Err(e) => {
                                    log::warn!("CLI elevated install failed: {}", e);
                                    // Still notify user - CLI is installed, just not in PATH
                                    app.emit(
                                        "cli-installed",
                                        "CLI installed but not in PATH. See settings to complete setup."
                                            .to_string(),
                                    )
                                    .ok();
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("CLI auto-installation failed: {}", e);
                    }
                }
            } else {
                log::info!(
                    "CLI already installed and up to date. Version: {:?}",
                    status.version
                );
            }
        }
        Err(e) => {
            log::warn!("Failed to check CLI status: {}", e);
        }
    }
}
