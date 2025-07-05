---
id: adr-003-exit-criteria-format
level: adr
status: decided
created_at: 2025-07-02T15:20:00Z
updated_at: 2025-07-02T15:20:00Z
parent: 
blocked_by: 
phase: decided
exit_criteria_met: false
decision_maker: team
supersedes:
---

# ADR-003: Exit Criteria Format

**Status**: Decided  
**Date**: 2025-07-02  
**Driver**: Process Documentation Initiative

## Context

We need a standardized way to define and track exit criteria that determine when a document is ready to spawn child documents at the next level. The format must be human-readable, machine-parseable, and support validation of completion before allowing decomposition into sub-documents. We use "exit" because its exiting the level not a phase. 

## Decision

We will use **GitHub-style markdown checkboxes** in a dedicated "Exit Criteria" section.

**Format Requirements:**
- Use `- [ ]` for unchecked and `- [x]` for checked criteria
- Each criterion must be specific and objectively verifiable
- Criteria appear in dedicated "## Exit Criteria" section
- Maximum 7 criteria per document (cognitive load limit)

**Example:**
```markdown
## Exit Criteria

- [ ] Problem statement is clear and agreed upon
- [x] Success metrics are measurable and defined
- [ ] Solution approach is sketched at high level
- [ ] Scope boundaries are documented and validated
```

**Validation Rules:**
- All checkboxes must be checked before decomposition to child documents
- Criteria cannot be removed once defined (only refined)
- Each document level has standard criteria determining readiness for decomposition
- System validates checkbox completion before allowing creation of child documents

**Level Transition Examples:**
- **Strategy Exit Criteria**: Are strategy objectives and goals understood well enough to begin discovering initiatives that work towards those goals?
- **Initiative Exit Criteria**: Is the initiative's scope and approach clear enough to decompose into specific, actionable tasks?
- **Task Exit Criteria**: Is the task well-defined enough that work can begin, or does it need breaking down into subtasks?

## Consequences

**Positive:**
- Familiar format (GitHub, Obsidian, most markdown processors)
- Human-readable and editable
- Easy to parse programmatically with regex
- Visual progress indication
- Forces specific, measurable criteria

**Negative:**
- Manual checkbox management required
- Easy to accidentally modify checkboxes
- Limited formatting options within criteria text
- No built-in approval workflow

## Validation

We'll know this was right if:
- Teams consistently use exit criteria before decomposing documents into child documents
- Criteria are specific enough to be objectively verified
- Tooling can reliably parse and validate completion
- Level transitions result in better prepared child documents
- Child documents have clearer context and requirements from completed parent exit criteria