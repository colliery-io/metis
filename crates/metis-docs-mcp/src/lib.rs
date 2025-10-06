#![allow(clippy::redundant_closure)]
#![allow(clippy::io_other_error)]

pub mod config;
pub mod error;
pub mod error_utils;
pub mod server;
pub mod tools;

pub use config::MetisServerConfig;
pub use error::{McpServerError, Result};
pub use server::MetisServerHandler;

use anyhow::Result as AnyhowResult;
use metis_core::{
    application::services::workspace::WorkspaceDetectionService,
    dal::database::Database,
    domain::configuration::FlightLevelConfig,
};
use rust_mcp_sdk::{
    mcp_server::server_runtime,
    schema::{
        Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
        LATEST_PROTOCOL_VERSION,
    },
    McpServer, StdioTransport, TransportOptions,
};
use tracing::info;

fn find_metis_log_path() -> Option<String> {
    let detection_service = WorkspaceDetectionService::new();
    
    // Use core service to find workspace
    if let Ok(Some(metis_dir)) = detection_service.find_workspace() {
        return Some(
            metis_dir
                .join("metis-mcp-server.log")
                .to_string_lossy()
                .to_string(),
        );
    }

    None
}

fn get_current_configuration() -> Option<FlightLevelConfig> {
    let detection_service = WorkspaceDetectionService::new();
    
    // Try to find workspace and load configuration
    if let Ok(Some(metis_dir)) = detection_service.find_workspace() {
        let db_path = metis_dir.join("metis.db");
        if let Ok(db) = Database::new(db_path.to_string_lossy().as_ref()) {
            if let Ok(mut config_repo) = db.configuration_repository() {
                if let Ok(config) = config_repo.get_flight_level_config() {
                    return Some(config);
                }
            }
        }
    }
    
    None
}

fn generate_dynamic_instructions() -> String {
    let config = get_current_configuration();
    let static_instructions = include_str!("../instructions.md");
    
    if let Some(config) = config {
        let config_section = format!(
            r#"
## Current Project Configuration

**Active Preset**: {}
**Enabled Document Types**: {}
**Hierarchy**: {}

### Available Operations
{}

### Configuration Notes
- To change configuration: Use `metis config set --preset <preset>` where preset is one of: full, streamlined, direct
- To enable/disable specific levels: Use `metis config set --strategies <true/false> --initiatives <true/false>`

"#,
            config.preset_name(),
            config.enabled_document_types().iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", "),
            config.hierarchy_display(),
            generate_operation_notes(&config)
        );
        
        format!("{}{}", config_section, static_instructions)
    } else {
        format!(
            r#"
## Configuration Status
**Status**: No active Metis workspace detected. Initialize a project first using `initialize_project`.

{}
"#,
            static_instructions
        )
    }
}

fn generate_operation_notes(config: &FlightLevelConfig) -> String {
    let mut notes = Vec::new();
    
    if !config.strategies_enabled {
        notes.push("- Strategy creation is disabled in this configuration");
    }
    
    if !config.initiatives_enabled {
        notes.push("- Initiative creation is disabled in this configuration");
    }
    
    if config.strategies_enabled && config.initiatives_enabled {
        notes.push("- All document types are available for creation");
    } else if !config.strategies_enabled && config.initiatives_enabled {
        notes.push("- Direct creation: Vision → Initiative → Task workflow");
    } else if config.strategies_enabled && !config.initiatives_enabled {
        notes.push("- Streamlined workflow: Vision → Strategy → Task");
    } else {
        notes.push("- Minimal workflow: Vision → Task (direct mode)");
    }
    
    notes.join("\n")
}

/// Run the MCP server
pub async fn run() -> AnyhowResult<()> {
    // Initialize logging only if we find an initialized metis project
    // Try to initialize, ignore if already initialized
    let _ = if let Some(log_path) = find_metis_log_path() {
        // Initialize tracing with file output in metis project
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        tracing_subscriber::fmt()
            .with_writer(log_file)
            .with_ansi(false)
            .try_init()
    } else {
        // No metis project found - use minimal console logging only
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .with_max_level(tracing::Level::WARN)
            .try_init()
    };

    // Load configuration (minimal for now)
    let config = MetisServerConfig::from_env()?;

    info!("Starting Metis MCP Server");

    // Create server details
    let server_details = InitializeResult {
        server_info: Implementation {
            name: "Metis Documentation Management System".to_string(),
            version: "0.4.2".to_string(),
            title: Some("Metis MCP Server".to_string()),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some(generate_dynamic_instructions()),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    // Create transport
    let transport = StdioTransport::new(TransportOptions::default())
        .map_err(|e| anyhow::anyhow!("Failed to create transport: {}", e))?;

    // Create handler
    let handler = MetisServerHandler::new(config);

    // Create and start server
    let server = server_runtime::create_server(server_details, transport, handler);

    info!("MCP Server starting on stdio transport");
    server
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("MCP server failed to start: {}", e))?;

    Ok(())
}
