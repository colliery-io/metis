---
id: initiative-mcp-server-implementation
level: initiative
status: completed
created_at: 2025-07-03T15:45:00Z
updated_at: 2025-07-04T21:15:00Z
parent: strategy-mcp-interface
blocked_by: 
tags:
  - "#initiative"
  # - "#phase/discovery"
  # - "#phase/design"
  # - "#phase/ready"
  # - "#phase/decompose"
  # - "#phase/active"
  - "#phase/completed"
exit_criteria_met: true
technical_lead: 
estimated_complexity: l
related_adrs: 
---

# MCP Server Implementation Initiative

## Context

The MCP Interface strategy requires a functioning MCP server that exposes the Metis core library functions to AI agents. A partial implementation exists in `src-bu/` using `rust_mcp_sdk` with skeleton tools for ticket operations, but it needs to be updated to work with the current core library architecture and expose the full range of document management capabilities.

Agents need access to document creation, validation, phase transitions, exit criteria checking, and the surgical update operations being developed in the Document Update Management initiative. The MCP server should provide a clean, well-documented interface that agents can use effectively.

## Goals & Non-Goals

**Goals:**
- Implement complete MCP server using rust_mcp_sdk
- Expose all core library functions through MCP tools
- Update existing skeleton implementation to current architecture
- Provide clear tool descriptions and parameter validation
- Support document creation, validation, transitions, and updates
- Handle errors gracefully with meaningful agent feedback
- Support vault path configuration and project setup

**Non-Goals:**
- Complex workflow orchestration or automation
- Real-time collaboration or multi-agent coordination
- Custom tool creation or dynamic tool registration
- Direct database administration through MCP
- File system operations beyond document management
- Advanced configuration management

## Detailed Design

### MCP Tools Architecture

Based on existing skeleton in `src-bu/mcp/tools.rs`, implement comprehensive tools:

```rust
// Project Discovery
#[mcp_tool(name = "list_projects")]
pub struct ListProjectsTool {
    pub include_inactive: Option<bool>,
}

#[mcp_tool(name = "initialize_project")]
pub struct InitializeProjectTool {
    pub project_name: String,
    pub description: Option<String>,
}

// Document Creation
#[mcp_tool(name = "create_document")]
pub struct CreateDocumentTool {
    pub project_name: String,
    pub document_type: String,  // vision, strategy, initiative, task, adr
    pub title: String,
    pub parent_title: Option<String>,
    pub risk_level: Option<String>,      // For strategies
    pub complexity: Option<String>,      // For initiatives  
    pub decision_maker: Option<String>,  // For ADRs
    pub stakeholders: Vec<String>,
}

// Document Validation
#[mcp_tool(name = "validate_document")]
pub struct ValidateDocumentTool {
    pub project_name: String,
    pub document_path: String,
}

// Phase Transitions
#[mcp_tool(name = "transition_phase")]
pub struct TransitionPhaseTool {
    pub project_name: String,
    pub document_path: String,
    pub new_phase: String,
    pub force: Option<bool>,
}

#[mcp_tool(name = "check_phase_transition")]
pub struct CheckPhaseTransitionTool {
    pub project_name: String,
    pub document_path: String,
    pub target_phase: String,
}

// Exit Criteria
#[mcp_tool(name = "validate_exit_criteria")]
pub struct ValidateExitCriteriaTool {
    pub project_name: String,
    pub document_path: String,
}

// Document Updates (requires Update Management initiative)
#[mcp_tool(name = "update_document_content")]
pub struct UpdateDocumentContentTool {
    pub project_name: String,
    pub document_path: String,
    pub section_heading: String,
    pub new_content: String,
}

#[mcp_tool(name = "update_exit_criterion")]
pub struct UpdateExitCriterionTool {
    pub project_name: String,
    pub document_path: String,
    pub criterion_text: String,
    pub completed: bool,
}

#[mcp_tool(name = "update_blocked_by")]
pub struct UpdateBlockedByTool {
    pub project_name: String,
    pub document_path: String,
    pub blocked_by: Vec<String>,
}

// Document Querying
#[mcp_tool(name = "list_documents")]
pub struct ListDocumentsTool {
    pub project_name: String,
    pub document_type: Option<String>,
    pub phase: Option<String>,
    pub limit: Option<u32>,
}

#[mcp_tool(name = "search_documents")]
pub struct SearchDocumentsTool {
    pub project_name: String,
    pub query: String,
    pub document_type: Option<String>,
    pub limit: Option<u32>,
}
```

### Server Configuration

```rust
pub struct MetisServerConfig {
    pub workspace_root: PathBuf,  // Contains multiple projects
    pub max_results: u32,
}

impl Default for MetisServerConfig {
    fn default() -> Self {
        Self {
            workspace_root: PathBuf::from("./"),
            max_results: 100,
        }
    }
}
```

### Error Handling Strategy

- **Project Not Found**: Clear error when project_name doesn't exist in workspace
- **Document Not Found**: Clear file path errors with suggestions
- **Validation Failures**: Detailed frontmatter and structure errors
- **Permission Errors**: Clear file system access messages
- **Invalid Parameters**: Parameter validation with expected values
- **Core Library Errors**: Translate Rust errors to agent-friendly messages

### Project Discovery Workflow

```rust
pub struct ProjectInfo {
    pub name: String,           // Directory name
    pub path: PathBuf,          // Full path to project
    pub last_modified: DateTime<Utc>,
    pub document_count: Option<u32>,    // Optional stats
    pub has_active_documents: bool,     // For filtering
}

// Agent workflow:
// 1. Agent calls list_projects() → gets ["project-a", "project-b", "project-c"]
// 2. Agent selects target project (UI or logic)
// 3. Agent calls create_document(project_name: "project-a", ...)
// 4. Server resolves "project-a" → /workspace/project-a/.metis.db
```

### Integration Approach

1. **Update existing src-bu structure** to work with current core library
2. **Implement project discovery** by scanning workspace_root for .metis.db files
3. **Add project resolution** to convert project names to full paths
4. **Implement tool handlers** that call core library functions with resolved paths
5. **Add comprehensive parameter validation** before core calls
6. **Convert core library results** to appropriate MCP responses

## Alternatives Considered

1. **Build from Scratch**
   - Pros: Clean implementation, perfect fit
   - Cons: Reinventing MCP setup, more development time
   - Decision: Rejected - existing skeleton provides good foundation

2. **Different MCP Framework**
   - Pros: Potentially simpler implementation
   - Cons: rust_mcp_sdk is already integrated and working
   - Decision: Rejected - stick with proven choice

3. **REST API Instead of MCP**
   - Pros: More universal, easier testing
   - Cons: Not designed for agent integration, requires separate discovery
   - Decision: Rejected - MCP is specifically for agent interactions

## Implementation Plan

1. **Phase 1: Core Integration** (Week 1)
   - Update existing MCP structure to use current core library
   - Implement basic document creation and validation tools
   - Add proper error handling and parameter validation

2. **Phase 2: Full Tool Set** (Week 2)
   - Implement phase transition tools
   - Add exit criteria validation tools
   - Create document querying and listing tools

3. **Phase 3: Update Tools** (Week 3)
   - Integrate with Document Update Management functions
   - Add content, criteria, and relationship update tools
   - Comprehensive tool testing

4. **Phase 4: Polish & Testing** (Week 4)
   - Server configuration and initialization
   - Integration testing with agent workflows
   - Documentation and usage examples

## Testing Strategy

- **Unit Tests**: Each MCP tool with various parameter combinations
- **Integration Tests**: Full MCP server with actual agent interactions
- **Error Handling Tests**: Invalid parameters, missing files, permission issues
- **Documentation Tests**: Tool descriptions and parameter validation

## Exit Criteria

- [ ] MCP server successfully starts and accepts connections
- [ ] All core library functions exposed through appropriate MCP tools
- [ ] Document creation, validation, and transitions work through MCP
- [ ] Exit criteria checking available to agents
- [ ] Update operations integrated (when Update Management complete)
- [ ] Document querying and searching functional
- [ ] Clear error messages for all failure scenarios
- [ ] Server configuration and vault setup working
- [ ] Comprehensive test coverage for all tools
- [ ] Documentation for agent integration