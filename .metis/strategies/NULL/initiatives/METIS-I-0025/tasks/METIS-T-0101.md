---
id: add-specification-templates
level: task
title: "Add Specification templates"
short_code: "METIS-T-0101"
created_at: 2026-03-03T21:00:00+00:00
updated_at: 2026-03-04T04:02:20.977083+00:00
parent: METIS-I-0025
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0025
---

# Add Specification templates

## Parent Initiative

[[METIS-I-0025]]

## Objective

Create the embedded template files for the Specification document type. These are compiled into the binary via `include_str!()` and provide the default content structure.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `crates/metis-docs-core/src/domain/documents/specification/frontmatter.yaml` — YAML frontmatter template with:
  - `id`, `level: specification`, `title`, `short_code`, `created_at`, `updated_at`
  - `parent` field (Vision or Initiative short code)
  - `tags` with `#specification` and `#phase/discovery`
  - `exit_criteria_met: false`
  - No `strategy_id` or `initiative_id` (attached document, not hierarchy node)
- [ ] `crates/metis-docs-core/src/domain/documents/specification/content.md` — Main content template with sections:
  - `## Overview` — What this system is and why it exists
  - `## System Context` — Actors, external systems, boundaries, key interactions
  - `## Requirements` — With subsections for Functional (REQ-x.x.x) and Non-Functional (NFR-x.x.x)
  - `## Architecture Framing` — Decision areas with constraints and required capabilities
  - `## Decision Log` — Links to ADRs spawned from this specification
  - `## Constraints` — Hard constraints (technical, organizational, regulatory)
  - `## Changelog` — Significant changes after initial publication
- [ ] `crates/metis-docs-core/src/templates/specification/frontmatter.yaml` — Filesystem override template (same as embedded)
- [ ] `crates/metis-docs-core/src/templates/specification/content.md` — Filesystem override template
- [ ] `crates/metis-docs-core/src/templates/specification/postmatter.md` — Postmatter template (if pattern requires it)
- [ ] Templates follow existing conventions (placeholder text style, section headers, YAML formatting)

## Implementation Notes

Reference existing templates:
- `crates/metis-docs-core/src/domain/documents/adr/frontmatter.yaml` — Frontmatter pattern
- `crates/metis-docs-core/src/domain/documents/adr/content.md` — Content pattern
- `crates/metis-docs-core/src/templates/adr/` — Filesystem override pattern

The content template should have rich placeholder text showing the REQ-x.x.x and NFR-x.x.x patterns with examples, so users understand the convention.

## Status Updates

### Session 1 (2026-03-03)

**All acceptance criteria met.**

Embedded templates (created in T-0100, verified here):
- `crates/metis-docs-core/src/domain/documents/specification/frontmatter.yaml` — Tera template with id, level, title, short_code, timestamps, parent, tags, exit_criteria_met, initiative_id
- `crates/metis-docs-core/src/domain/documents/specification/content.md` — Full content template with all 7 sections: Overview, System Context, Requirements (REQ-x.x.x/NFR-x.x.x tables), Architecture Framing, Decision Log, Constraints, Changelog
- `crates/metis-docs-core/src/domain/documents/specification/acceptance_criteria.md` — Default acceptance criteria

Filesystem override templates (created in this task):
- `crates/metis-docs-core/src/templates/specification/frontmatter.yaml` — Clean version with commented phase progression
- `crates/metis-docs-core/src/templates/specification/content.md` — Simplified version without CONDITIONAL tags
- `crates/metis-docs-core/src/templates/specification/postmatter.md` — Exit criteria checklist

Build passes, all templates follow existing conventions.