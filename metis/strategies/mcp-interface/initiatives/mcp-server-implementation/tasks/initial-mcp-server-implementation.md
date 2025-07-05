---
id: task-initial-mcp-server-implementation
level: task
status: completed
created_at: 2025-07-03T19:55:00Z
updated_at: 2025-07-04T20:30:00Z
parent: initiative-mcp-server-implementation
blocked_by:
tags:
  - "#task"
  - "#phase/completed"
  # - "#phase/doing"
  # - "#phase/completed"
exit_criteria_met: true
assignee: 
estimated_hours: 8
pr_links: []
---

# Initial MCP Server Implementation

## Parent Initiative
[[MCP Server Implementation Initiative]]

## Objective
Create the foundational MCP server implementation using `rust_mcp_sdk` that exposes core Metis functionality through MCP tools, enabling AI agents to interact with the Metis documentation system.

## Acceptance Criteria
- [x] `metis-mcp-server` crate created with proper dependencies
- [x] MCP server starts successfully and accepts connections
- [x] Basic project discovery tools implemented (`list_projects`, `initialize_project`)
- [x] Core document management tools implemented (`create_document`, `validate_document`)
- [x] Document update tools integrated (`update_document_content`, `update_exit_criterion`, `update_blocked_by`)
- [x] Phase transition tools implemented (`transition_phase`, `check_phase_transition`)
- [x] Exit criteria validation tools implemented (`validate_exit_criteria`)
- [x] Document querying tools implemented (`list_documents`, `search_documents`)
- [x] Comprehensive error handling with agent-friendly messages
- [x] Server configuration and workspace management
- [x] Basic integration tests with MCP client

## Implementation Notes

### Crate Structure
```
metis-mcp-server/
├── Cargo.toml
├── src/
│   ├── main.rs          # Server executable
│   ├── lib.rs           # Library exports
│   ├── server.rs        # MCP server setup
│   ├── config.rs        # Configuration management
│   ├── tools/           # MCP tool implementations
│   │   ├── mod.rs
│   │   ├── project.rs   # Project management tools
│   │   ├── document.rs  # Document creation/validation
│   │   ├── update.rs    # Document update tools
│   │   ├── phase.rs     # Phase transition tools
│   │   └── query.rs     # Document querying tools
│   └── error.rs         # Error handling
└── tests/
    └── integration.rs   # MCP integration tests
```

### Dependencies (Cargo.toml)
```toml
[package]
name = "metis-mcp-server"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "metis-mcp-server"
path = "src/main.rs"

[dependencies]
metis-core = { path = "../metis-core" }
rust_mcp_sdk = "0.1"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
tempfile = "3.0"
```

### Server Configuration
```rust
#[derive(Debug, Clone)]
pub struct MetisServerConfig {
    pub workspace_root: PathBuf,
    pub max_results: u32,
    pub bind_address: String,
    pub port: u16,
}

impl Default for MetisServerConfig {
    fn default() -> Self {
        Self {
            workspace_root: PathBuf::from("./"),
            max_results: 100,
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}
```

### Core MCP Tools

#### Project Management Tools
```rust
// List available projects in workspace
#[mcp_tool(name = "list_projects")]
pub struct ListProjectsTool {
    pub include_inactive: Option<bool>,
}

// Initialize new project
#[mcp_tool(name = "initialize_project")]
pub struct InitializeProjectTool {
    pub project_name: String,
    pub description: Option<String>,
}
```

#### Document Management Tools
```rust
// Create new document
#[mcp_tool(name = "create_document")]
pub struct CreateDocumentTool {
    pub project_name: String,
    pub document_type: String,  // vision, strategy, initiative, task, adr
    pub title: String,
    pub parent_title: Option<String>,
    pub risk_level: Option<String>,
    pub complexity: Option<String>,
    pub decision_maker: Option<String>,
    pub stakeholders: Vec<String>,
}

// Validate document
#[mcp_tool(name = "validate_document")]
pub struct ValidateDocumentTool {
    pub project_name: String,
    pub document_path: String,
}
```

#### Update Tools
```rust
// Update document content
#[mcp_tool(name = "update_document_content")]
pub struct UpdateDocumentContentTool {
    pub project_name: String,
    pub document_path: String,
    pub section_heading: String,
    pub new_content: String,
}

// Update exit criterion
#[mcp_tool(name = "update_exit_criterion")]
pub struct UpdateExitCriterionTool {
    pub project_name: String,
    pub document_path: String,
    pub criterion_text: String,
    pub completed: bool,
}

// Update blocked_by relationships
#[mcp_tool(name = "update_blocked_by")]
pub struct UpdateBlockedByTool {
    pub project_name: String,
    pub document_path: String,
    pub blocked_by: Vec<String>,
}
```

#### Phase Transition Tools
```rust
// Transition document phase
#[mcp_tool(name = "transition_phase")]
pub struct TransitionPhaseTool {
    pub project_name: String,
    pub document_path: String,
    pub new_phase: String,
    pub force: Option<bool>,
}

// Check if phase transition is valid
#[mcp_tool(name = "check_phase_transition")]
pub struct CheckPhaseTransitionTool {
    pub project_name: String,
    pub document_path: String,
    pub target_phase: String,
}
```

#### Query Tools
```rust
// List documents in project
#[mcp_tool(name = "list_documents")]
pub struct ListDocumentsTool {
    pub project_name: String,
    pub document_type: Option<String>,
    pub phase: Option<String>,
    pub limit: Option<u32>,
}

// Search documents
#[mcp_tool(name = "search_documents")]
pub struct SearchDocumentsTool {
    pub project_name: String,
    pub query: String,
    pub document_type: Option<String>,
    pub limit: Option<u32>,
}
```

### Project Resolution Strategy
```rust
pub struct ProjectResolver {
    workspace_root: PathBuf,
}

impl ProjectResolver {
    pub fn resolve_project(&self, project_name: &str) -> Result<PathBuf> {
        let project_path = self.workspace_root.join(project_name);
        let db_path = project_path.join(".metis.db");
        
        if db_path.exists() {
            Ok(project_path)
        } else {
            Err(anyhow::anyhow!("Project '{}' not found in workspace", project_name))
        }
    }
    
    pub fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        // Scan workspace for .metis.db files
        // Return project information
    }
}
```

### Error Handling Strategy
```rust
#[derive(Debug, thiserror::Error)]
pub enum McpServerError {
    #[error("Project not found: {project_name}")]
    ProjectNotFound { project_name: String },
    
    #[error("Document not found: {document_path}")]
    DocumentNotFound { document_path: String },
    
    #[error("Invalid parameter: {param_name} - {message}")]
    InvalidParameter { param_name: String, message: String },
    
    #[error("Core library error: {0}")]
    CoreLibrary(#[from] metis_core::MetisError),
    
    #[error("MCP protocol error: {0}")]
    McpProtocol(#[from] rust_mcp_sdk::Error),
}
```

### Server Initialization
```rust
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::init();
    
    let config = MetisServerConfig::from_args();
    
    let server = MetisServer::new(config).await?;
    server.run().await?;
    
    Ok(())
}
```

### Integration with Core Library
Each MCP tool handler will:
1. **Resolve Project**: Convert project_name to full path
2. **Validate Parameters**: Check required fields and formats
3. **Call Core Function**: Use metis-core library functions
4. **Handle Errors**: Convert core errors to agent-friendly messages
5. **Return Results**: Format responses for MCP protocol

### Testing Strategy
- **Unit Tests**: Each tool with various parameter combinations
- **Integration Tests**: Full MCP server with mock client
- **Error Handling Tests**: Invalid parameters, missing projects
- **Performance Tests**: Large document sets, concurrent operations

## Dependencies
- Requires workspace restructuring to be completed first
- Uses existing core library functions from Document Update Management initiative
- Integrates with all implemented core functionality

## Status Updates
*To be added during implementation*

## Exit Criteria
- [x] MCP server starts and accepts connections successfully
- [x] All core Metis operations accessible through MCP tools
- [x] Project discovery and resolution working correctly
- [x] Document CRUD operations functional via MCP
- [x] Update operations integrated and working
- [x] Phase transitions accessible to agents
- [x] Error handling provides clear, actionable messages
- [x] Integration tests pass with MCP client interactions
- [x] Server ready for agent integration and testing