# Phase Lifecycle Reference

Every Metis document moves through a defined sequence of phases. Transitions must be to adjacent phases — you cannot skip phases.

## Vision Phases

```
draft → review → published
```

| From | Valid Targets |
|------|---------------|
| `draft` | `review` |
| `review` | `published` |
| `published` | *(terminal)* |

| Phase | Description |
|-------|-------------|
| `draft` | Content being written |
| `review` | Under stakeholder review |
| `published` | Finalized and active |

---

## Initiative Phases

```
discovery → design → ready → decompose → active → completed
```

| From | Valid Targets |
|------|---------------|
| `discovery` | `design` |
| `design` | `ready` |
| `ready` | `decompose` |
| `decompose` | `active` |
| `active` | `completed` |
| `completed` | *(terminal)* |

| Phase | Description |
|-------|-------------|
| `discovery` | Requirements gathering |
| `design` | Technical approach planning |
| `ready` | Design reviewed, ready to decompose |
| `decompose` | Breaking into tasks |
| `active` | Tasks being executed |
| `completed` | Delivered |

---

## Task Phases

```
backlog → todo → active → completed
                    ↕
                 blocked
```

| From | Valid Targets |
|------|---------------|
| `backlog` | `todo` |
| `todo` | `active`, `blocked` |
| `active` | `completed`, `blocked` |
| `blocked` | `todo`, `active` |
| `completed` | *(terminal)* |

| Phase | Description |
|-------|-------------|
| `backlog` | Not yet scheduled (standalone items) |
| `todo` | Ready to work on |
| `active` | In progress |
| `blocked` | Waiting on dependency |
| `completed` | Done |

Tasks are the only document type supporting backward transitions (from `blocked` to `todo` or `active`).

---

## ADR Phases

```
draft → discussion → decided → superseded
```

| From | Valid Targets |
|------|---------------|
| `draft` | `discussion` |
| `discussion` | `decided` |
| `decided` | `superseded` |
| `superseded` | *(terminal)* |

| Phase | Description |
|-------|-------------|
| `draft` | Initial writeup |
| `discussion` | Under debate |
| `decided` | Finalized and approved |
| `superseded` | Replaced by newer ADR |

---

## Specification Phases

```
discovery → drafting → review → published
```

| From | Valid Targets |
|------|---------------|
| `discovery` | `drafting` |
| `drafting` | `review` |
| `review` | `published` |
| `published` | *(terminal, but content remains editable)* |

| Phase | Description |
|-------|-------------|
| `discovery` | Requirements gathering |
| `drafting` | Writing content |
| `review` | Stakeholder review |
| `published` | Approved (content remains editable) |

Note: The `drafting` phase is available via MCP but may not be recognized by the CLI `metis transition` command.

---

## Phase Storage

The current phase is stored as a tag in the document's YAML frontmatter:

```yaml
tags:
  - "#initiative"
  - "#phase/design"
```

The first `#phase/` tag in the array is the authoritative current phase. When you transition, Metis replaces this tag with the new phase and updates the `updated_at` timestamp.

## Auto-Advance

When you omit the target phase in a transition command, Metis automatically moves to the next phase in the sequence:

```bash
metis transition PROJ-T-0001    # todo → active (auto-advance)
```

For tasks in `blocked` state, auto-advance moves to `active`.

## Exit Criteria

Each document type has acceptance criteria that define what must be true before advancing. These are tracked in the `exit_criteria_met` frontmatter field and the `## Acceptance Criteria` section.

**Important behavioral difference:** The MCP tool enforces exit criteria by default (use the `force` parameter to bypass). The CLI does not enforce exit criteria — transitions always succeed if the phase adjacency rule is met.
