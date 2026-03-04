---
id: add-specification-document-type
level: initiative
title: "Add Specification Document Type"
short_code: "METIS-I-0025"
created_at: 2026-03-03T19:13:27.267297+00:00
updated_at: 2026-03-03T19:13:27.267297+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: add-specification-document-type
---

# Add Specification Document Type

## Context

Metis currently has a gap between Vision (strategic direction) and Initiative (concrete projects). Complex systems routinely produce heavyweight design artifacts — PRDs with dozens of numbered requirements, system context documents, architecture framing with decision areas — that don't fit in either existing document type. Vision is too high-level and static; Initiative is execution-focused and time-bounded.

With the Strategy layer removed (METIS-I-0024), the hierarchy slot and `S` short code prefix are available. The Specification document type fills this gap as a living system-level design document that captures what the system must do, how it's structured, and what architectural decisions frame the design.

This is a recurring pattern across projects: any system complex enough to warrant multiple initiatives needs a place to capture and evolve the system requirements, context, and architecture framing that those initiatives execute against.

## Goals & Non-Goals

**Goals:**
- Add a Specification document type that sits between Vision and Initiative in the hierarchy
- Provide a template with structured sections for system design artifacts (overview, system context, requirements, architecture framing, constraints, decision log)
- Use `S` short code prefix (e.g., `METIS-S-0001`)
- Support a lifecycle that allows the specification to evolve after publishing (living document)
- Parent to Vision (published), child Initiatives can be created against a published Specification
- Requirements as markdown convention (REQ-x.x.x, NFR-x.x.x pattern) in the template — not structured tracking
- ADRs remain independent but cross-referenced from the Specification's Decision Log section
- Integrate across CLI, MCP tools, and plugin documentation

**Non-Goals:**
- Structured/parsed requirement tracking — requirements are a template convention, not database entities
- ADR parenting — ADRs stay parentless, Specifications reference them by short code
- Multi-specification orchestration — one spec per vision is the expected common case
- Replacing or subsuming Initiative's design phase — Specifications are system-level, Initiatives handle project-level design

## Specification Design

### Hierarchy Position

```
Vision (published) → Specification → Initiative → Task
                          ↕
                         ADR (cross-referenced, not parented)
```

### Phases

`discovery → drafting → review → published`

- **Discovery** — Gathering requirements from stakeholders, reviewing prior art, understanding constraints
- **Drafting** — Writing the PRD, system context, architecture framing
- **Review** — Circulating to technical leads, iterating on feedback
- **Published** — Accepted as current system design. Initiatives can be created against it. **Unlike Vision, published Specifications remain editable** — requirements get added, mutated, or dropped as the system evolves. A changelog section in the template tracks significant changes.

### Template Sections

The Specification template provides structured sections. Unused sections should be deleted (same convention as other Metis templates):

- **Overview** — What this system is and why it exists. More technical than Vision, focused on capabilities and constraints.
- **System Context** — Actors, external systems, boundaries, key interactions. Who uses the system and how.
- **Requirements** — Numbered, structured requirements organized by capability area:
  - Functional Requirements (REQ-x.x.x pattern)
  - Non-Functional Requirements (NFR-x.x.x pattern)
  - Each requirement has a unique identifier, description, and optionally a rationale
- **Architecture Framing** — Decision areas identified with constraints and required capabilities. This is not where decisions are made — it's where the questions are framed. Each decision area spawns an ADR when the team deliberates.
- **Decision Log** — Links to ADRs spawned from this specification, with status and summary. Maintained as the spec evolves.
- **Constraints** — Hard constraints the design must work within. Technical, organizational, or regulatory.
- **Changelog** — Significant changes to the specification after initial publication. Date, what changed, why.

### Configuration & Presets

With Strategy removed, presets become:
- **streamlined** (default): Vision → Specification → Initiative → Task (specifications enabled)
- **direct**: Initiative → Task (specifications disabled)

The config key will be `specifications_enabled: bool` in `FlightLevelConfig`.

### Short Code

`S` prefix — e.g., `PROJ-S-0001`. One spec per vision is typical but multiple are allowed.

### Database

- `level = 'specification'` in the documents table
- No new columns needed beyond what exists for other document types
- Parent reference to Vision via existing `parent` field

### Filesystem Layout

```
.metis/
├── specifications/
│   └── PROJ-S-0001/
│       └── specification.md
├── initiatives/
│   └── ...
└── vision.md
```

## Alternatives Considered

**Expand Vision to carry sub-sections** — Rejected. Vision becomes overloaded. A 600+ line PRD doesn't belong in the same document as the strategic direction. Different authors, different review cycles, different audiences.

**Use Initiative with a "design" template variant** — Rejected. Initiatives are time-bounded execution units. A Specification is a living document that multiple initiatives execute against over months or years. Different lifecycle, different purpose.

**Keep Strategy and rename to Specification** — Rejected (see METIS-I-0024). Strategy's phases, template, and semantics don't match. Cleaner to build fresh.

## Implementation Plan

Depends on METIS-I-0024 (strategy removal) being complete first.

1. Add Specification to domain model — struct, phases, Document trait impl
2. Add Specification template with all sections
3. Add Specification to configuration and presets
4. Add database support — level type, parent validation
5. Add Specification to services — creation, discovery, transition, archive
6. Add CLI command — `metis create specification`
7. Add MCP tool support — `"specification"` as valid document_type
8. Add tests across all crates
9. Update plugin documentation — hierarchy descriptions, phase tables, workflow examples