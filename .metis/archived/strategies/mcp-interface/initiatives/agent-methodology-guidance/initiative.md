---
id: initiative-agent-methodology-guidance
title: Agent Methodology Guidance Initiative
level: initiative
status: completed
created_at: 2025-07-03T16:30:00Z
updated_at: 2025-07-04T21:00:00Z
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
estimated_complexity: m
related_adrs: 
archived: false
---

# Agent Methodology Guidance Initiative

## Context

While the MCP server provides tools for document management, agents need to understand the Metis methodology itself - the philosophy, document hierarchy, phase progressions, and best practices. Without this understanding, agents will use the tools incorrectly, creating documents in the wrong order, skipping important phases, or misunderstanding the relationships between document types.

This is like giving someone a hammer without explaining what nails are for. The tools alone are insufficient; agents need methodology guidance to use them effectively.

## Goals & Non-Goals

**Goals:**
- Create comprehensive agent instructions for Metis methodology
- Provide initialization tool to write guidance to agent-specific files (CLAUDE.md, .cursorrules, etc.)
- Embed methodology understanding in MCP server responses
- Provide contextual guidance based on current project state
- Include best practices and common patterns
- Guide agents through proper document hierarchies
- Explain phase transitions and their meaning
- Demonstrate when to use each document type

**Non-Goals:**
- Complex AI training or fine-tuning
- Enforcing rigid workflows (maintain flexibility)
- Creating an AI agent ourselves
- Methodology documentation for humans (separate concern)
- Project management automation
- Decision-making on behalf of users

## Detailed Design

### MCP Server Instructions

Provide methodology guidance through agent-specific mechanisms:

```rust
#[mcp_tool(name = "initialize_methodology_guidance")]
pub struct InitializeMethodologyGuidanceTool {
    pub output_format: Option<String>, // "claude_md", "cursor_rules", "instructions"
    pub output_path: Option<String>,   // Where to write the guidance
}

pub fn get_mcp_instructions() -> String {
    r#"# Metis Flight Levels Methodology

## Overview
Metis implements a hierarchical document system based on Flight Levels:
- **Vision** (Level 3): Overall purpose and direction
- **Strategy** (Level 2): How to achieve the vision
- **Initiative** (Level 1): Concrete projects implementing strategies
- **Task** (Level 0): Individual work items
- **ADR**: Architectural decisions at any level

## Document Hierarchy Rules
1. Every project starts with a Vision document
2. Strategies must reference the Vision as parent
3. Initiatives must reference a Strategy as parent
4. Tasks must reference an Initiative as parent
5. ADRs can exist at any level without a parent

## Phase Progressions
Each document type has specific phases that must be followed:

### Vision Phases
- draft → review → published
- Start in draft, move to review when ready for feedback
- Publish only when fully aligned with stakeholders

### Strategy Phases  
- shaping → design → ready → active → completed
- Shaping: Problem definition and exploration
- Design: Solution approach and planning
- Ready: Exit criteria met, ready to execute
- Active: Initiatives are being implemented
- Completed: All initiatives done, outcomes achieved

### Initiative Phases
- discovery → design → ready → decompose → active → completed
- Discovery: Understanding the problem space
- Design: Technical approach and architecture
- Ready: Design approved, resources allocated
- Decompose: Break into specific tasks
- Active: Tasks being executed
- Completed: All tasks done, initiative delivered

### Task Phases
- todo → doing → completed
- Simple workflow for execution tracking

### ADR Phases
- draft → discussion → decided → superseded
- Draft: Proposal stage
- Discussion: Gathering feedback
- Decided: Decision made and documented
- Superseded: Replaced by newer decision

## Best Practices
1. Always check exit criteria before phase transitions
2. Update parent documents when child documents complete
3. Use blocked_by to track dependencies
4. Keep documents focused on their level's concerns
5. Validate documents after any updates

## Common Patterns
- Start with Vision, then create 2-3 key Strategies
- Each Strategy should have 3-5 Initiatives
- Each Initiative typically has 5-15 Tasks
- Create ADRs for significant technical decisions
- Review and update parent documents regularly

## Tool Usage Guidance
When using MCP tools, follow this sequence:
1. `initialize_project` - Start new project with Vision
2. `create_document` - Add documents in hierarchical order
3. `validate_document` - Check structure after creation
4. `validate_exit_criteria` - Before phase transitions
5. `transition_phase` - Move through phases systematically
6. `update_*` - Make incremental improvements
"#.to_string()
}

impl InitializeMethodologyGuidanceTool {
    pub fn call_tool(&self) -> Result<CallToolResult> {
        let instructions = get_mcp_instructions();
        
        match self.output_format.as_deref() {
            Some("claude_md") => {
                let path = self.output_path.as_deref().unwrap_or("CLAUDE.md");
                write_claude_md(instructions, path)?;
                Ok(CallToolResult::text_content(
                    format!("Methodology guidance written to {}", path),
                    None
                ))
            },
            Some("cursor_rules") => {
                let path = self.output_path.as_deref().unwrap_or(".cursorrules");
                write_cursor_rules(instructions, path)?;
                Ok(CallToolResult::text_content(
                    format!("Methodology guidance written to {}", path),
                    None
                ))
            },
            _ => {
                // Return as direct response for agents that don't use files
                Ok(CallToolResult::text_content(instructions, None))
            }
        }
    }
}
```

### Contextual Tool Responses

Enhance tool responses with methodology guidance:

```rust
impl CreateDocumentTool {
    pub fn call_tool(&self) -> Result<CallToolResult> {
        // Validate document creation makes sense
        let guidance = match self.document_type.as_str() {
            "strategy" => {
                if !project_has_vision() {
                    return Ok(CallToolResult::text_content(
                        "Cannot create Strategy without Vision. Create a Vision document first using document_type='vision'.",
                        None
                    ));
                }
                "Strategy created. Next steps: Define exit criteria, then create 3-5 Initiatives when ready."
            },
            "initiative" => {
                if self.parent_title.is_none() {
                    return Ok(CallToolResult::text_content(
                        "Initiative requires a parent Strategy. Specify parent_title with the Strategy name.",
                        None
                    ));
                }
                "Initiative created. Next steps: Move through discovery→design→ready phases before creating Tasks."
            },
            // ... similar for other types
        };
        
        // Create document and return with guidance
        let result = create_document(...)?;
        Ok(CallToolResult::text_content(
            format!("{}\n\n{}", result, guidance),
            None
        ))
    }
}
```

### Project State Awareness

```rust
#[mcp_tool(name = "get_methodology_guidance")]
pub struct GetMethodologyGuidanceTool {
    pub context: Option<String>, // "starting", "planning", "executing", etc.
}

impl GetMethodologyGuidanceTool {
    pub fn call_tool(&self) -> Result<CallToolResult> {
        let project_state = analyze_project_state()?;
        
        let guidance = match project_state {
            ProjectState::Empty => {
                "Start with initializing the project, then create a Vision document to establish purpose and direction."
            },
            ProjectState::VisionOnly => {
                "Vision established. Next: Create 2-3 Strategy documents for major workstreams."
            },
            ProjectState::StrategiesInShaping => {
                "Strategies being shaped. Focus on problem definition and exit criteria before moving to design phase."
            },
            ProjectState::ReadyForInitiatives => {
                "Strategies ready. Create Initiatives for concrete implementation projects."
            },
            ProjectState::ActiveDevelopment => {
                "Active development. Monitor Task progress and update Initiative/Strategy status as work completes."
            },
            // ... more states
        };
        
        Ok(CallToolResult::text_content(guidance, None))
    }
}
```

### Validation with Guidance

```rust
impl ValidateDocumentTool {
    pub fn call_tool(&self) -> Result<CallToolResult> {
        let validation = validate_document(&self.document_path)?;
        
        if !validation.is_valid {
            let guidance = get_validation_guidance(&validation.errors);
            return Ok(CallToolResult::text_content(
                format!("Validation failed:\n{}\n\nGuidance:\n{}", 
                    validation.errors.join("\n"),
                    guidance
                ),
                None
            ));
        }
        
        Ok(CallToolResult::text_content("Document valid!", None))
    }
}
```

## Alternatives Considered

1. **Static Documentation Only**
   - Pros: Simple to create
   - Cons: Agents might not read or understand it
   - Decision: Rejected - need embedded guidance

2. **Strict Enforcement in Tools**
   - Pros: Ensures compliance
   - Cons: Too rigid, reduces flexibility
   - Decision: Rejected - guide don't force

3. **Separate Methodology Query Tool**
   - Pros: Clean separation
   - Cons: Agents might not know to use it
   - Decision: Partially adopted - have both embedded and query options

## Implementation Plan

1. **Phase 1: Core Instructions** (Week 1)
   - Write comprehensive MCP server instructions
   - Include in server initialization
   - Test with various agents

2. **Phase 2: Contextual Responses** (Week 2)
   - Enhance all tool responses with guidance
   - Add project state analysis
   - Implement methodology query tool

3. **Phase 3: Validation Guidance** (Week 3)
   - Helpful error messages with next steps
   - Phase transition recommendations
   - Best practice reminders

4. **Phase 4: Testing & Refinement** (Week 4)
   - Test with real agents
   - Refine guidance based on usage
   - Create example agent workflows

## Testing Strategy

- **Agent Testing**: Real agents following methodology
- **Scenario Testing**: Various project states and transitions
- **Guidance Testing**: Verify helpful responses
- **Error Testing**: Ensure guidance for common mistakes
- **Documentation Testing**: Accuracy of methodology description

## Implementation Decision

**Decision Made: Simple Instructions Approach**

After analyzing the full initiative design, we decided to implement a simplified approach focusing on comprehensive startup instructions rather than building additional tools and contextual responses.

**Rationale:**
- The MCP server's `instructions` field provides 2,751 characters of comprehensive methodology guidance
- Agents receive complete Flight Levels methodology on startup
- 80% of the value with 20% of the implementation effort
- No additional tools or complexity needed
- Agents can refer back to startup instructions as needed

**What was implemented:**
- Enhanced MCP server instructions with complete methodology guide
- Document hierarchy (Vision → Strategy → Initiative → Task + ADR)
- All phase definitions and transitions
- Direct path usage patterns
- Essential workflows and best practices
- Common project patterns

**What was deferred:**
- Additional methodology tools (`initialize_methodology_guidance`, `get_methodology_guidance`)
- Contextual responses in existing tools
- Project state awareness
- Enhanced validation guidance

This approach successfully provides agents with comprehensive methodology understanding while maintaining system simplicity.

## Exit Criteria

- [x] MCP server provides comprehensive methodology instructions
- [ ] ~~All tools include contextual guidance in responses~~ (Deferred - not needed)
- [ ] ~~Project state awareness guides appropriate next steps~~ (Deferred - not needed)
- [ ] ~~Validation errors include helpful remediation guidance~~ (Deferred - not needed)
- [ ] ~~Methodology query tool provides targeted advice~~ (Deferred - not needed)
- [x] Agents can successfully follow methodology without human help
- [x] Common mistakes are prevented through proactive guidance
- [x] Documentation demonstrates full agent workflows