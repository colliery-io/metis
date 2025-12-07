---
id: custom-templates
level: initiative
title: "Custom Templates"
short_code: "METIS-I-0016"
created_at: 2025-12-07T03:01:01.822298+00:00
updated_at: 2025-12-07T03:40:57.436955+00:00
parent: METIS-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: custom-templates
---

# Custom Templates Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

Metis currently embeds document templates in the Rust binary at compile time. Each document type (vision, strategy, initiative, task, ADR) has:
- `content.md` - The markdown body with sections like "Context", "Goals", etc.
- `frontmatter.yaml` - YAML metadata template

Templates use Tera templating engine with variables like `{{ title }}`, `{{ slug }}`, `{{ short_code }}`.

**Problem**: Users cannot customize these templates to match their organization's terminology, required sections, or workflow patterns. A team that uses "Objectives" instead of "Goals" or wants different acceptance criteria formats must accept the defaults or manually edit every document after creation.

**Current state explored**:
- Templates in `crates/metis-docs-core/src/domain/documents/{type}/`
- Tera rendering in document creation service
- Phase definitions are hardcoded in `Phase` enum and `can_transition_to()` methods

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Allow users to customize document content templates (the markdown sections/boilerplate)
- Allow users to customize exit criteria patterns per phase
- Support a fallback hierarchy: Project (.metis/templates/) > Global (~/.config/metis/templates/) > Hardcoded defaults
- Keep templates freeform with light validation (ensure Tera variables are valid)

**Non-Goals:**
- Customizing phase sequences or workflow definitions (phases are fixed)
- Customizing frontmatter structure (affects parsing, too risky)
- Building a template marketplace or sharing mechanism
- GUI template editor

## Discovery Decisions **[RESOLVED]**

1. **Exit criteria structure**: Per document type only
   - `templates/{type}/exit_criteria.md`
   - No per-phase granularity (keeps it simple)

2. **Variable discovery**: Documentation only
   - Document available variables in README/docs
   - No CLI tooling for variable inspection

3. **Validation**: Full render with sample data
   - When loading a custom template, render it with sample values
   - Fail fast with clear error if template is broken
   - Prevents runtime surprises during document creation

4. **CLI commands**: None for now
   - Intentional friction - teams must manually copy/edit templates
   - Forces thoughtfulness about customization
   - Can add convenience commands later if needed

## Use Cases

### UC1: Team customizes task template
- **Actor**: Team lead setting up Metis for their team
- **Scenario**: Wants tasks to have "Definition of Done" instead of "Acceptance Criteria"
- **Expected**: Export default, edit, save to `.metis/templates/task/content.md`, new tasks use it

### UC2: Organization sets global defaults
- **Actor**: Platform team standardizing tooling
- **Scenario**: Wants all projects to use company terminology
- **Expected**: Save templates to `~/.config/metis/templates/`, applies to all projects unless overridden

### UC3: Per-phase exit criteria
- **Actor**: Quality-focused team
- **Scenario**: Wants specific exit criteria checklist for initiative "ready" phase
- **Expected**: Creates exit criteria template that appears in documents at that phase

## Technical Approach

### Template File Structure

```
# Project-level (highest priority)
.metis/templates/
  vision/
    content.md
    exit_criteria.md
  strategy/
    content.md
    exit_criteria.md
  initiative/
    content.md
    exit_criteria.md
  task/
    content.md
    exit_criteria.md
  adr/
    content.md
    exit_criteria.md

# Global-level (fallback)
~/.config/metis/templates/
  {same structure}

# Embedded (final fallback)
  Compiled into binary as today
```

### Template Loading Service

New `TemplateLoader` service in `metis-docs-core`:

```rust
pub struct TemplateLoader {
    project_path: Option<PathBuf>,  // .metis/templates/
    global_path: PathBuf,           // ~/.config/metis/templates/
}

impl TemplateLoader {
    /// Load template with fallback chain
    pub fn load_content_template(&self, doc_type: &str) -> Result<String> {
        // 1. Try project: .metis/templates/{type}/content.md
        // 2. Try global: ~/.config/metis/templates/{type}/content.md
        // 3. Fall back to embedded default
    }
    
    /// Validate template by rendering with sample data
    pub fn validate_template(&self, template: &str, doc_type: &str) -> Result<()> {
        let sample_context = self.sample_context_for_type(doc_type);
        tera.render_str(template, &sample_context)?;
        Ok(())
    }
}
```

### Integration Points

1. **DocumentCreationService** - Use `TemplateLoader` instead of `include_str!()`
2. **Each document type's `new()` method** - Accept template content as parameter
3. **Workspace validation** - Validate custom templates on workspace load

### Validation Flow

On document creation:
1. Load template via fallback chain
2. Render with sample context to validate
3. If validation fails, return clear error with template path
4. If validation passes, render with actual context

### Sample Context Per Type

Each document type defines sample values for validation:
- `title`: "Sample Title"
- `slug`: "sample-title"  
- `short_code`: "TEST-T-0001"
- `created_at`: current timestamp
- etc.

Type-specific fields use sensible defaults (e.g., `complexity: "M"`).

## Alternatives Considered

1. **YAML/TOML config for templates** - Rejected; markdown files are more intuitive and match output format

2. **Per-phase exit criteria** - Rejected for simplicity; can add later if needed

3. **Template inheritance/extends** - Rejected; adds complexity, users can copy-paste sections

4. **CLI template commands** - Deferred; intentional friction for now

## Implementation Plan **[DECOMPOSE PHASE]**

*To be decomposed into tasks after design review*