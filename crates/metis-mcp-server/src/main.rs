use anyhow::Result;
use metis_mcp_server::{MetisServerConfig, MetisServerHandler};
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
    let current_dir = std::env::current_dir().ok()?;
    let mut current = current_dir;

    // Traverse upward looking for initialized metis project
    loop {
        let metis_dir = current.join("metis");
        let metis_db = metis_dir.join(".metis.db");

        // Only create logs if there's an initialized metis project
        if metis_dir.is_dir() && metis_db.exists() {
            return Some(
                metis_dir
                    .join("metis-mcp-server.log")
                    .to_string_lossy()
                    .to_string(),
            );
        }

        // Move to parent directory
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            // Reached filesystem root
            break;
        }
    }

    None
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging only if we find an initialized metis project
    if let Some(log_path) = find_metis_log_path() {
        // Initialize tracing with file output in metis project
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        tracing_subscriber::fmt()
            .with_writer(log_file)
            .with_ansi(false)
            .init();
    } else {
        // No metis project found - use minimal console logging only
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .with_max_level(tracing::Level::WARN)
            .init();
    }

    // Load configuration (minimal for now)
    let config = MetisServerConfig::from_env()?;

    info!("Starting Metis MCP Server");

    // Create server details
    let server_details = InitializeResult {
        server_info: Implementation {
            name: "Metis Documentation Management System".to_string(),
            version: "0.1.0".to_string(),
            title: Some("Metis MCP Server".to_string()),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some(
            r#"# Metis Flight Levels Documentation Management

## Overview
Metis implements a hierarchical document management system based on Flight Levels methodology. You manage projects by creating and transitioning documents through defined phases using direct file paths.

## Document Hierarchy (Flight Levels)
Create documents in this order, with each level building on the previous:

1. **Vision** (Level 3) - Overall purpose and direction
   - Always start here - defines why the project exists
   - Phases: draft → review → published

2. **Strategy** (Level 2) - How to achieve the vision  
   - Must reference Vision as parent
   - Phases: shaping → design → ready → active → completed

3. **Initiative** (Level 1) - Concrete projects implementing strategies
   - Must reference Strategy as parent
   - Phases: discovery → design → ready → decompose → active → completed

4. **Task** (Level 0) - Individual work items
   - Must reference Initiative as parent
   - Phases: todo → doing → completed

5. **ADR** (Architectural Decision Record) - Technical decisions
   - Can exist at any level, no parent required
   - Phases: draft → discussion → decided → superseded

## Direct Path Usage
All tools use `project_path` pointing to the directory containing `.metis.db`:
- Initialize: `{"project_path": "/path/to/project", "project_name": "my-project"}`
- Create doc: `{"project_path": "/path/to/project", "document_type": "vision", "title": "Project Vision"}`
- Update: `{"project_path": "/path/to/project", "document_path": "vision.md", ...}`

## Essential Workflow
1. **Start**: `initialize_project` creates `.metis.db` and initial structure
2. **Build hierarchy**: Create Vision → Strategies → Initiatives → Tasks
3. **Progress**: Use `validate_exit_criteria` before `transition_phase`
4. **Update**: Use `update_*` tools for incremental changes
5. **Query**: Use `list_documents` and `search_documents` to explore

## Phase Transition Rules
- Always validate exit criteria before transitioning: `validate_exit_criteria`
- Use `transition_phase` only when ready to progress
- Phase progression is generally linear (no skipping)
- Force transitions with `force: true` only when necessary

## Best Practices
- Create documents in hierarchical order (Vision before Strategy, etc.)
- Define clear exit criteria for each phase
- Use `blocked_by` to track dependencies
- Validate documents after creation/updates
- Keep documents focused on their level's scope
- Update parent documents when children complete

## Common Patterns
- Start with Vision, then 2-3 key Strategies
- Each Strategy typically has 3-5 Initiatives  
- Each Initiative usually has 5-15 Tasks
- Create ADRs for significant technical decisions
- Review hierarchy regularly as work progresses"#.to_string(),
        ),
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
