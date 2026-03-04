# Preset Selection

Metis supports two configuration presets that determine which flight levels are active. Choosing the right preset depends on your team size, coordination needs, and project complexity.

## The Two Presets

| Preset | Hierarchy | Levels |
|--------|-----------|--------|
| **Streamlined** | Vision -> Initiative -> Task | 3 levels |
| **Direct** | Vision -> Task | 2 levels |

## Streamlined Preset

**Hierarchy**: Vision -> Initiative -> Task

**Use when:**
- Single team owns the entire vision
- Path from vision to execution is clear
- Coordination happens informally (standups, slack, pairing)
- Work naturally groups into distinct projects with their own lifecycles

**Example scenarios:**
- Small engineering teams (2-8 people)
- Technically-focused projects with clear scope
- Startups where everyone knows the strategy implicitly
- Open source projects with established direction

**What you give up:**
- Initiative-level grouping of related tasks (compared to Direct, you keep this)
- Coordination documentation (happens informally instead)

**This is the most common choice** for small teams doing technical execution.

## Direct Preset

**Hierarchy**: Vision -> Task

**Use when:**
- Solo work or very small scope
- Simple projects with obvious decomposition
- Personal task tracking
- Prototypes or experiments

**Example scenarios:**
- Personal side projects
- Proof of concept work
- Learning/tutorial projects
- Single-feature implementations

**What you give up:**
- Initiative-level grouping of related tasks
- Project-level tracking and phases
- Decomposition discipline

**Warning**: Direct preset can lead to disconnected tasks if the project grows. Be ready to upgrade.

## Decision Framework

Ask these questions in order:

### 1. Will work be grouped into distinct projects with their own lifecycles?

- **Yes** -> Streamlined preset (you need Initiative grouping)
- **No** -> Continue to question 2

### 2. Is this solo work or a very simple project?

- **Yes** -> Direct preset
- **No** -> Default to Streamlined (safer choice)

### When in doubt: Streamlined

Streamlined is the safe default. It provides enough structure to track projects without unnecessary overhead. You can always document coordination decisions through ADRs or initiative documentation if needed.

## Changing Presets

> **Note**: Preset configuration requires the Metis CLI. There is no MCP tool for changing presets—this is intentional, as preset changes affect project structure and should be deliberate decisions.

**Check current preset:**
```bash
metis config show
```

**Change preset:**
```bash
metis config set --preset <name>  # streamlined or direct
```

**Upgrading** (Direct -> Streamlined):
- Existing documents remain valid
- New parent relationships become available
- Consider whether orphan items should be grouped

**Downgrading** (Streamlined -> Direct):
- Initiative-level documents become orphaned
- May need to archive or restructure
- Generally not recommended mid-project

## Signs You Chose Wrong

**Too much overhead (preset too heavy):**
- Initiatives with single tasks
- Phases that get skipped routinely
- Initiative grouping adding no clarity

**Not enough structure (preset too light):**
- Tasks that don't relate to each other
- No clear project boundaries
- Work items that take months (should be initiatives)
- Coordination happening in side channels instead of documented

**Judgment call**: If you're fighting the structure, the structure might be wrong. But also consider whether you're avoiding necessary discipline.
