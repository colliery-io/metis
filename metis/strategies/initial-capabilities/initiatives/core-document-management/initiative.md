---
id: initiative-core-document-management
level: initiative
status: active
created_at: 2025-07-02T18:05:00Z
updated_at: 2025-07-03T12:15:00Z
parent: strategy-initial-capabilities
blocked_by: 
phase: active
tags:
  - "#initiative"
  - "#phase/active"
exit_criteria_met: false
technical_lead: 
estimated_complexity: m
related_adrs: 
---

# Core Document Management Library Initiative

## Context

The Initial Capabilities strategy requires core business logic functions that implement the Metis methodology for document lifecycle management. This initiative will build the library that sits on top of the Storage & Indexing System to provide high-level operations for creating, validating, and transitioning documents through their defined phases.

This library must enforce the business rules defined in our document templates (Vision, Strategy, Initiative, Task, ADR) while providing a clean, type-safe API that can be used by any interface implementation (CLI, MCP server, web UI, etc.).

## Goals & Non-Goals

**Goals:**
- Implement `render()` function to create documents from templates and context
- Implement `validate()` function to check document structure and compliance
- Build phase transition functions that enforce business rules
- Create exit criteria validation for phase gates
- Support frontmatter parsing and validation
- Provide core document lifecycle operations

**Non-Goals:**
- Complex relationship management or hierarchy operations
- Change tracking and audit trails (future enhancement)
- User interface implementations (CLI, web, etc.)
- File system watching or real-time synchronization
- Advanced validation beyond core document structure
- Storage operations (handled by Storage & Indexing System)

## Detailed Design

### Core Functions

The library provides these four essential functions:

#### 1. Document Rendering

```rust
// Main render function: create document from template and write to filesystem
pub async fn render(
    document_type: DocumentType,
    context: DocumentContext,
    docs_root: &Path,
) -> Result<PathBuf>; // Returns path to created file

// Context for document creation (frontmatter metadata only)
#[derive(Debug, Clone)]
pub struct DocumentContext {
    pub title: String,
    pub parent_id: Option<String>,
    pub custom_filename: Option<String>,
    // Document-type specific frontmatter fields
    pub risk_level: Option<RiskLevel>,        // For strategies
    pub complexity: Option<Complexity>,       // For initiatives  
    pub assignee: Option<String>,            // For tasks
    pub decision_maker: Option<String>,      // For ADRs
    pub estimated_hours: Option<u32>,        // For tasks
    pub review_date: Option<String>,         // For strategies (ISO date)
    pub stakeholders: Vec<String>,           // For vision/strategy
}

// Validated enums for frontmatter fields
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    S, // Small
    M, // Medium
    L, // Large
    XL, // Extra Large
}

impl DocumentContext {
    // Validate context for specific document type
    pub fn validate_for_type(&self, doc_type: DocumentType) -> Result<()> {
        match doc_type {
            DocumentType::Strategy => {
                if self.risk_level.is_none() {
                    return Err(DocumentError::MissingRequiredField("risk_level".to_string()));
                }
            },
            DocumentType::Initiative => {
                if self.complexity.is_none() {
                    return Err(DocumentError::MissingRequiredField("complexity".to_string()));
                }
            },
            DocumentType::Task => {
                if self.assignee.is_none() && self.estimated_hours.is_none() {
                    // At least one should be provided for tasks
                }
            },
            DocumentType::Adr => {
                if self.decision_maker.is_none() {
                    return Err(DocumentError::MissingRequiredField("decision_maker".to_string()));
                }
            },
            DocumentType::Vision => {
                // Vision has fewer required frontmatter fields
            },
        }
        Ok(())
    }
}

// Internal functions
fn generate_filepath(
    doc_type: DocumentType,
    title: &str,
    docs_root: &Path,
) -> PathBuf; // e.g., docs_root/strategies/initial-capabilities.md

fn apply_template(
    doc_type: DocumentType,
    context: &DocumentContext,
) -> Result<String>; // Returns complete markdown with frontmatter
```

#### 2. Document Validation

```rust
// Main validate function: check document structure and compliance
pub fn validate(document_path: &str) -> Result<ValidationResult>;

// Alternative validation from content string
pub fn validate_content(content: &str) -> Result<ValidationResult>;

#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub document_type: Option<DocumentType>,
    pub frontmatter_errors: Vec<String>,
    pub structure_errors: Vec<String>,
    pub phase_errors: Vec<String>,
}
```

#### 3. Phase Transitions

```rust
// Phase transition with validation
pub fn transition_phase(
    document_path: &str,
    new_phase: &str,
    force: bool,
) -> Result<String>; // Returns updated document content

// Check if phase transition is allowed
pub fn can_transition_to_phase(
    document_path: &str,
    target_phase: &str,
) -> Result<bool>;
```

#### 4. Exit Criteria Validation

```rust
// Exit criteria checking
pub fn validate_exit_criteria(document_path: &str) -> Result<ExitCriteriaResult>;

// Alternative from content
pub fn validate_exit_criteria_content(content: &str) -> Result<ExitCriteriaResult>;

#[derive(Debug)]
pub struct ExitCriteriaResult {
    pub met: bool,
    pub total_criteria: usize,
    pub completed_criteria: usize,
    pub missing_criteria: Vec<String>,
}
```

### Business Rules Implementation

#### Phase Transition Rules

```rust
impl PhaseManager {
    // Enforce phase flow rules based on document type
    fn validate_phase_transition(
        current_phase: &str,
        target_phase: &str,
        doc_type: DocumentType,
    ) -> Result<()> {
        match doc_type {
            DocumentType::Vision => {
                // draft → review → published
                Self::validate_vision_phases(current_phase, target_phase)
            },
            DocumentType::Strategy => {
                // shaping → design → ready → active → complete
                Self::validate_strategy_phases(current_phase, target_phase)
            },
            DocumentType::Initiative => {
                // discovery → design → ready → decompose → active → complete
                Self::validate_initiative_phases(current_phase, target_phase)
            },
            DocumentType::Task => {
                // todo → doing → complete
                Self::validate_task_phases(current_phase, target_phase)
            },
            DocumentType::Adr => {
                // draft → discussion → decided → superseded
                Self::validate_adr_phases(current_phase, target_phase)
            },
        }
    }
}
```

#### Exit Criteria Parsing

```rust
impl ExitCriteriaParser {
    // Parse markdown checkboxes from content
    pub fn parse_exit_criteria(content: &str) -> Result<Vec<ExitCriterion>> {
        // Find "## Exit Criteria" section
        // Parse checkbox items: - [ ] or - [x]
        // Return structured criteria with completion status
    }
    
    pub fn update_exit_criteria(
        content: &str,
        criterion_index: usize,
        completed: bool,
    ) -> Result<String> {
        // Update specific checkbox in content
        // Return modified content
    }
}

#[derive(Debug, Clone)]
pub struct ExitCriterion {
    pub index: usize,
    pub text: String,
    pub completed: bool,
}
```

#### Template System

**Three-Part Template Architecture:**
- **Front Matter**: YAML frontmatter with templated metadata (validated context)
- **Content**: Static scaffold/questionnaire for manual completion
- **Post Matter**: Static exit criteria (no templating needed)

```rust
use tera::Tera;
use include_dir::{include_dir, Dir};

// Compile-time template bundling
static TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();
        
        // Load all templates from bundled directory
        for file in TEMPLATES_DIR.files() {
            let path = file.path().to_string_lossy();
            let content = file.contents_utf8().unwrap();
            tera.add_raw_template(&path, content)?;
        }
        
        Ok(Self { tera })
    }
    
    pub fn render_document(
        &self,
        doc_type: DocumentType,
        context: &DocumentContext,
    ) -> Result<String> {
        // Validate context for document type
        context.validate_for_type(doc_type)?;
        
        let frontmatter = self.render_frontmatter(doc_type, context)?;
        let content = self.load_content_scaffold(doc_type)?;
        let postmatter = self.load_postmatter(doc_type)?;
        
        Ok(format!("---\n{}\n---\n\n{}\n\n{}", frontmatter, content, postmatter))
    }
    
    fn render_frontmatter(&self, doc_type: DocumentType, context: &DocumentContext) -> Result<String> {
        let template_name = format!("{}/frontmatter.yaml", doc_type.template_dir());
        self.tera.render(&template_name, &context.to_tera_context()?)
    }
    
    fn load_content_scaffold(&self, doc_type: DocumentType) -> Result<String> {
        // Static content scaffold - no templating needed
        let template_name = format!("{}/content.md", doc_type.template_dir());
        self.tera.get_template(&template_name)
            .map(|t| t.source.clone())
            .ok_or_else(|| DocumentError::TemplateNotFound(template_name))
    }
    
    fn load_postmatter(&self, doc_type: DocumentType) -> Result<String> {
        // Static exit criteria - no templating needed
        let template_name = format!("{}/postmatter.md", doc_type.template_dir());
        self.tera.get_template(&template_name)
            .map(|t| t.source.clone())
            .ok_or_else(|| DocumentError::TemplateNotFound(template_name))
    }
}

impl DocumentType {
    fn template_dir(&self) -> &'static str {
        match self {
            DocumentType::Vision => "vision",
            DocumentType::Strategy => "strategy", 
            DocumentType::Initiative => "initiative",
            DocumentType::Task => "task",
            DocumentType::Adr => "adr",
        }
    }
}

// Template directory structure:
// templates/
// ├── strategy/
// │   ├── frontmatter.yaml
// │   ├── content.md
// │   └── postmatter.md
// ├── initiative/
// │   ├── frontmatter.yaml
// │   ├── content.md
// │   └── postmatter.md
// └── ... (other document types)
```

**Example Template Files:**

```yaml
# templates/strategy/frontmatter.yaml
id: strategy-{{ slug }}
level: strategy
status: shaping
created_at: {{ created_at }}
updated_at: {{ updated_at }}
parent: "[[{{ parent_title }}]]"
blocked_by: 
phase: shaping
tags:
  - "#strategy"
  - "#phase/shaping"
exit_criteria_met: false
success_metrics: []
risk_level: {{ risk_level | default(value="medium") }}
stakeholders: []
review_date: {{ review_date }}
```

```markdown
<!-- templates/strategy/content.md -->
# {Strategy Title} Strategy

## Problem Statement

{Describe the problem and why it matters - 1-2 paragraphs}

## Success Metrics

- {Measurable outcome 1}
- {Measurable outcome 2}

## Solution Approach

{High-level approach without implementation details}

## Scope

**In Scope:**
- {What we will address}

**Out of Scope:**
- {What we won't address}

## Risks & Unknowns

- {Major risk or unknown 1}
- {Major risk or unknown 2}

## Implementation Dependencies

{Describe the critical path and initiative dependencies}

## Change Log

### YYYY-MM-DD - Initial Strategy
- **Change**: Created initial strategy document
- **Rationale**: {Why this strategy was needed}
- **Impact**: Baseline established for strategic direction
- **Next Review**: {Review date}
```

```markdown
<!-- templates/strategy/postmatter.md -->
## Exit Criteria

- [ ] Problem statement is clear and agreed upon
- [ ] Success metrics are measurable and defined
- [ ] Solution approach is sketched at high level
- [ ] Scope boundaries are documented and validated
- [ ] Major risks are identified and assessed
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    #[error("Document not found: {id}")]
    NotFound { id: String },
    
    #[error("Invalid phase transition from {from} to {to} for document type {doc_type}")]
    InvalidPhaseTransition {
        from: String,
        to: String,
        doc_type: String,
    },
    
    #[error("Exit criteria not met: {missing_count} of {total_count} criteria incomplete")]
    ExitCriteriaNotMet {
        missing_count: usize,
        total_count: usize,
    },
    
    #[error("Invalid relationship: {relationship_type} from {from_id} to {to_id}")]
    InvalidRelationship {
        relationship_type: String,
        from_id: String,
        to_id: String,
    },
    
    #[error("Frontmatter validation failed: {errors:?}")]
    FrontmatterValidation { errors: Vec<String> },
    
    #[error("Storage error: {source}")]
    Storage { source: anyhow::Error },
}
```

## Alternatives Considered

1. **Procedural Functions vs. Struct-based API**
   - Pros of procedural: Simple, functional approach
   - Pros of struct-based: Better state management, easier testing
   - Decision: Hybrid approach - stateless functions that take store as parameter

2. **Static Templates vs. Dynamic Configuration**
   - Pros of static: Compile-time validation, no runtime errors
   - Pros of dynamic: Flexible, customizable templates
   - Decision: Static templates based on ADR definitions for consistency

3. **Immediate Validation vs. Lazy Validation**
   - Pros of immediate: Fast feedback, prevents invalid states
   - Pros of lazy: Better performance, allows incremental edits
   - Decision: Immediate validation with optional force override

4. **Event-driven vs. Direct Function Calls**
   - Pros of events: Loose coupling, extensible
   - Pros of direct: Simpler, easier to debug
   - Decision: Direct function calls for core operations, events for future extensions

## Implementation Plan

1. **Phase 1: Templates & Rendering** (Week 1)
   - Define DocumentType enum and DocumentContext struct
   - Implement static template system based on ADR definitions
   - Create render() function with template application

2. **Phase 2: Document Validation** (Week 2)
   - Frontmatter parsing and validation logic
   - Document structure validation
   - Implement validate() and validate_content() functions

3. **Phase 3: Phase Management** (Week 3)
   - Phase transition rule enforcement
   - Exit criteria parsing from markdown checkboxes
   - Implement transition_phase() and can_transition_to_phase() functions

4. **Phase 4: Exit Criteria System** (Week 4)
   - Complete exit criteria validation logic
   - Testing and integration
   - Documentation and examples

## Testing Strategy

- **Unit Tests**: All four core functions with various input scenarios
- **Template Tests**: Ensure all document types render correctly from templates
- **Validation Tests**: Test document validation across valid and invalid documents
- **Phase Transition Tests**: Validate all allowed and disallowed phase flows
- **Exit Criteria Tests**: Test checkbox parsing and completion validation
- **Error Handling Tests**: Ensure proper error messages for all failure cases

## Exit Criteria

- [ ] render() function creates valid documents from templates for all document types
- [ ] validate() function correctly identifies valid and invalid documents
- [ ] transition_phase() enforces business rules and prevents invalid transitions
- [ ] validate_exit_criteria() accurately parses and checks markdown checkboxes
- [ ] All functions provide clear error messages for invalid inputs
- [ ] Business rules match specifications in document template ADRs
- [ ] API is simple, focused, and type-safe
- [ ] Core functionality works independently of storage layer
- [ ] Documentation includes examples for all four main functions

## Tasks

### Phase 1: Templates & Rendering
- [ ] [[Template Definition System]] - Define template structure with comment/uncomment approach
- [ ] [[Implement DocumentContext]] - Context struct and validation based on template needs
- [ ] [[Build TemplateEngine]] - Tera integration with compile-time template loading
- [ ] [[Implement render() Function]] - Main document creation function

### Phase 2: Document Validation
- [ ] [[Implement Document Validation]] - validate() and validate_content() functions

### Phase 3: Phase Management
- [ ] [[Implement Phase Transitions]] - transition_phase() with comment/uncomment logic

### Phase 4: Exit Criteria System
- [ ] [[Implement Exit Criteria]] - validate_exit_criteria() with checkbox parsing