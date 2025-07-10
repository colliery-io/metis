---
id: task-template-definition
title: "Template Definition System"
level: task
status: completed
created_at: 2025-07-03T12:20:00Z
updated_at: 2025-07-03T13:30:00Z
parent: initiative-core-document-management
blocked_by: 
phase: completed
tags:
  - "#task"
  - "#phase/completed"
exit_criteria_met: true
assignee: 
estimated_hours: 6
pr_links: []
archived: false
---

# Template Definition System

## Parent Initiative

[[Core Document Management Library]]

## Objective

Define and implement the template system for all Metis document types (Vision, Strategy, Initiative, Task, ADR) with self-documenting frontmatter that uses comment/uncomment approach for phase transitions and tags.

## Acceptance Criteria

- [x] Template directory structure created under `src/templates/`
- [x] Frontmatter templates for all 5 document types with full phase flows
- [x] Content scaffold templates with guided sections for each document type
- [x] Exit criteria templates with document-type specific requirements
- [x] Comment/uncomment approach for phase and tag management
- [ ] ~~Template validation for required fields per document type~~ - Moved to [[Implement DocumentContext]]
- [ ] ~~Tera template engine integration for dynamic values~~ - Moved to [[Build TemplateEngine]]
- [ ] ~~Template compilation at build time using `include_dir!`~~ - Moved to [[Build TemplateEngine]]

## Template Structure Design

### Directory Layout
```
templates/
├── vision/
│   ├── frontmatter.yaml
│   ├── content.md
│   └── postmatter.md
├── strategy/
│   ├── frontmatter.yaml
│   ├── content.md
│   └── postmatter.md
├── initiative/
│   ├── frontmatter.yaml
│   ├── content.md
│   └── postmatter.md
├── task/
│   ├── frontmatter.yaml
│   ├── content.md
│   └── postmatter.md
└── adr/
    ├── frontmatter.yaml
    ├── content.md
    └── postmatter.md
```

### Comment/Uncomment Approach

**Strategy for phase and tag fields:**
- Show all valid phases for the document type as commented lines
- User uncomments the current phase and comments out the previous one
- Same approach for phase-specific tags
- Makes phase progression self-documenting

**Example Strategy Frontmatter:**
```yaml
id: strategy-{{ slug }}
level: strategy
status: shaping
created_at: {{ created_at }}
updated_at: {{ updated_at }}
parent: "[[{{ parent_title }}]]"
blocked_by: 

# Phase progression for strategies
# phase: shaping
phase: design
# phase: ready
# phase: active
# phase: completed

tags:
  - "#strategy"
  # - "#phase/shaping"
  - "#phase/design"
  # - "#phase/ready"
  # - "#phase/active"
  # - "#phase/completed"

exit_criteria_met: false
success_metrics: []
risk_level: {{ risk_level | default(value="medium") }}
stakeholders: []
review_date: {{ review_date }}
```

### Document Type Specific Phases

**Vision:** draft → review → published
**Strategy:** shaping → design → ready → active → completed
**Initiative:** discovery → design → ready → decompose → active → completed
**Task:** todo → doing → completed
**ADR:** draft → discussion → decided → superseded

### Template Variables

**Common variables:**
- `{{ slug }}` - URL-friendly version of title
- `{{ created_at }}` - ISO timestamp
- `{{ updated_at }}` - ISO timestamp  
- `{{ parent_title }}` - Parent document title for linking

**Document-specific variables:**
- `{{ risk_level }}` - Strategy risk level (low/medium/high/critical)
- `{{ complexity }}` - Initiative complexity (s/m/l/xl)
- `{{ assignee }}` - Task assignee
- `{{ estimated_hours }}` - Task time estimate
- `{{ decision_maker }}` - ADR decision maker
- `{{ review_date }}` - Strategy review date

## Implementation Notes

### Template Engine Setup

Use Tera for templating with compile-time bundling:

```rust
use tera::Tera;
use include_dir::{include_dir, Dir};

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
}
```

### Context Validation

Each document type requires specific fields:

- **Strategy:** risk_level required
- **Initiative:** complexity required  
- **Task:** assignee or estimated_hours recommended
- **ADR:** decision_maker required
- **Vision:** no additional requirements

### Content Scaffolds

Each document type gets a content template with:
- Guided section headers
- Placeholder text explaining what to write
- Example content where helpful
- Clear instructions for completion

### Exit Criteria Templates

Document-type specific exit criteria that reflect the business rules:
- Strategy: Problem defined, metrics identified, approach sketched
- Initiative: Goals clear, design complete, ready for decomposition
- Task: Acceptance criteria defined, work completed
- ADR: Decision documented, rationale provided, impact assessed

## Status Updates

### 2025-07-03 - COMPLETED

**Template files created successfully:**

All template files created in `src/templates/` with three-part structure:
- **Frontmatter templates**: YAML with comment/uncomment phase progression
- **Content templates**: Guided scaffolding for each document type
- **Exit criteria templates**: Level-appropriate readiness checkboxes

**Key features implemented:**
- Comment/uncomment approach for phases and tags (per ADR-004)
- Document-type specific phase flows (vision, strategy, initiative, task, adr)
- Exit criteria focused on readiness for child document creation (per ADR-003)
- Template variables for dynamic content (title, slug, timestamps, etc.)

**Scope properly separated:**
- Template engine implementation moved to [[Build TemplateEngine]]
- Context validation moved to [[Implement DocumentContext]]
- Document rendering moved to [[Implement render() Function]]