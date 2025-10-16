---
id: 003-exit-criteria-format
level: adr
title: "Exit Criteria Format"
number: 3
short_code: "METIS-A-0003"
created_at: 2025-07-02T15:20:00Z
updated_at: 2025-07-02T15:20:00Z
decision_date: 2025-07-02
decision_maker: team
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"

exit_criteria_met: true
strategy_id: NULL
initiative_id: NULL
---

# ADR-003: Exit Criteria Format

**Status**: Decided  
**Date**: 2025-07-02  
**Driver**: Process Documentation Initiative

## Context **[REQUIRED]**

We need a standardized way to define and track exit criteria that determine when a document is ready to spawn child documents at the next level. The format must be human-readable, machine-parseable, and support validation of completion before allowing decomposition into sub-documents. We use "exit" because its exiting the level not a phase.

## Decision **[REQUIRED]**

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

## Alternatives Analysis **[CONDITIONAL: Complex Decision]**

**Markdown Checkboxes**
- Pros: Familiar, parseable, visual
- Cons: Manual management
- Risk Level: Low
- Implementation Cost: Low

**YAML Properties**
- Pros: Structured, consistent
- Cons: Less visual, complex
- Risk Level: Medium
- Implementation Cost: Medium

**Custom Format**
- Pros: Perfect fit for needs
- Cons: Unfamiliar, tooling needed
- Risk Level: High
- Implementation Cost: High

## Rationale **[REQUIRED]**

GitHub-style markdown checkboxes provide the best balance of familiarity, readability, and parseability. They're widely supported across markdown processors and provide immediate visual feedback on progress.

## Consequences **[REQUIRED]**

### Positive
- Familiar format (GitHub, Obsidian, most markdown processors)
- Human-readable and editable
- Easy to parse programmatically with regex
- Visual progress indication
- Forces specific, measurable criteria

### Negative
- Manual checkbox management required
- Easy to accidentally modify checkboxes
- Limited formatting options within criteria text
- No built-in approval workflow

### Neutral
- Requires tooling to validate completion before document decomposition

## Review Schedule **[CONDITIONAL: Temporary Decision]**

### Review Triggers
- Teams report difficulties with checkbox format
- Need for more complex validation workflows
- Integration issues with parsing tools

### Scheduled Review
- **Next Review Date**: 2026-07-02
- **Review Criteria**: Team adoption, validation effectiveness, tooling reliability
- **Sunset Date**: N/A (foundational format decision)