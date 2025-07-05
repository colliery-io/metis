---
id: adr-009-hierarchical-folder-structure
level: adr
status: decided
created_at: 2025-07-03T13:00:00Z
updated_at: 2025-07-03T13:00:00Z
decision_date: 2025-07-03
decision_maker: Engineering Team
parent: "[[Vision]]"
blocked_by: 
phase: decided
tags:
  - "#adr"
  - "#phase/decided"
exit_criteria_met: false
---

# ADR-009: Hierarchical Folder Structure for Document Organization

## Status

Decided

## Context

As we began implementing the second initiative (Core Document Management Library), the flat folder structure for tasks became unwieldy. With 7 tasks for a single initiative and more initiatives planned, finding and managing related documents was becoming difficult.

Current flat structure:
```
docs/planning-metis/
├── initiatives/
│   ├── storage-indexing-system.md
│   └── core-document-management.md
└── tasks/
    ├── create-database-schema.md
    ├── implement-document-store.md
    ├── template-definition.md
    └── ... (rapidly growing list)
```

This structure makes it hard to:
- See which tasks belong to which initiative
- Navigate related documents
- Understand document relationships at a glance
- Scale to hundreds of tasks across many initiatives

## Decision

We will organize documents in a hierarchical folder structure that mirrors their parent-child relationships, using descriptive filenames for the main document in each folder.

New structure:
```
docs/planning-metis/
├── vision.md
└── strategies/
    └── {strategy-name}/
        ├── strategy.md
        └── initiatives/
            └── {initiative-name}/
                ├── initiative.md
                └── tasks/
                    └── {task-name}.md
```

Naming conventions:
- Strategy folders contain `strategy.md`
- Initiative folders contain `initiative.md`
- Task files keep their descriptive names
- Vision remains at root as `vision.md`
- ADRs remain in flat `decisions/` folder (they're cross-cutting)

## Rationale

### Why hierarchical over flat structure?

1. **Natural organization** - Files are grouped by their actual relationships
2. **Scalability** - Can handle hundreds of documents without chaos
3. **Discoverability** - Easy to find all tasks for an initiative
4. **Maintenance** - Related documents move together

### Why not use phase-based folders?

- Phases are already tracked via tags (`#phase/active`, etc.)
- Moving files between phase folders as they progress would be cumbersome
- Phase information is metadata, not structural

### Why descriptive names (strategy.md, initiative.md)?

- Immediately clear what type of document when viewing folder contents
- Consistent pattern across all levels
- No ambiguity about which is the "main" document
- Follows principle of explicit over implicit

### Why not _index.md or README.md?

While these are valid conventions:
- `_index.md` is common in static site generators but has no special meaning in Obsidian
- `README.md` is GitHub-centric
- `strategy.md`/`initiative.md` is more descriptive and self-documenting

## Consequences

### Positive

- Clear visual hierarchy matching conceptual hierarchy
- Easier navigation and discovery
- Natural grouping of related work
- Reduced cognitive load when finding documents
- Better scales to large projects

### Negative

- Deeper nesting (4-5 levels for tasks)
- More complex file paths in links
- Initial migration effort required
- Some tools may handle deep nesting poorly

### Neutral

- File paths become longer but more descriptive
- Requires updating link paths when reorganizing
- May need tooling updates to support new structure

## Implementation

1. Create new folder structure
2. Move existing documents to new locations
3. Update all internal links and frontmatter
4. Update any tooling that assumes flat structure
5. Document the convention in project guidelines

## References

- [[Storage & Indexing System]] - First initiative to experience scaling issues
- [[Core Document Management Library]] - Second initiative that triggered this decision