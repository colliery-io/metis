# Preset Selection

Metis supports three configuration presets that determine which flight levels are active. Choosing the right preset depends on your team size, coordination needs, and project complexity.

## The Three Presets

| Preset | Hierarchy | Levels |
|--------|-----------|--------|
| **Full** | Vision -> Strategy -> Initiative -> Task | All 4 |
| **Streamlined** | Vision -> Initiative -> Task | Skip Strategy |
| **Direct** | Vision -> Task | Skip Strategy + Initiative |

## Full Preset

**Hierarchy**: Vision -> Strategy -> Initiative -> Task

**Use when:**
- Multiple teams or stakeholders need coordination
- A single vision requires different strategic approaches
- Resource allocation decisions span multiple initiatives
- You need explicit documentation of strategic trade-offs

**Example scenarios:**
- Enterprise product development with platform, mobile, and web teams
- Multi-quarter programs with competing priorities
- Cross-functional initiatives (engineering + design + marketing)

**What Strategy adds:**
- Explicit coordination logic between initiatives
- Resource allocation visibility
- Strategic trade-off documentation
- Clear ownership of approach decisions

## Streamlined Preset

**Hierarchy**: Vision -> Initiative -> Task

**Use when:**
- Single team owns the entire vision
- Path from vision to execution is clear
- Coordination happens informally (standups, slack, pairing)
- Strategic decisions are obvious or already made

**Example scenarios:**
- Small engineering teams (2-8 people)
- Technically-focused projects with clear scope
- Startups where everyone knows the strategy implicitly
- Open source projects with established direction

**What you give up:**
- Explicit strategic coordination (happens informally instead)
- Documentation of approach trade-offs
- Multi-team resource allocation visibility

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

### 1. Do multiple teams need to coordinate on this vision?

- **Yes** -> Full preset (you need Strategy for coordination)
- **No** -> Continue to question 2

### 2. Will work be grouped into distinct projects with their own lifecycles?

- **Yes** -> Streamlined preset (you need Initiative grouping)
- **No** -> Continue to question 3

### 3. Is this solo work or a very simple project?

- **Yes** -> Direct preset
- **No** -> Default to Streamlined (safer choice)

### When in doubt: Streamlined

Streamlined is the safe default. It provides enough structure to track projects without the overhead of explicit strategy documentation. You can always add strategic coordination through ADRs or initiative documentation if needed.

## Changing Presets

Presets can be changed via `metis config set --preset <name>`.

**Upgrading** (Direct -> Streamlined -> Full):
- Existing documents remain valid
- New parent relationships become available
- Consider whether orphan items should be grouped

**Downgrading** (Full -> Streamlined -> Direct):
- Higher-level documents become orphaned
- May need to archive or restructure
- Generally not recommended mid-project

## Signs You Chose Wrong

**Too much overhead (preset too heavy):**
- Strategy documents that just say "do the obvious thing"
- Initiatives with single tasks
- Phases that get skipped routinely

**Not enough structure (preset too light):**
- Tasks that don't relate to each other
- No clear project boundaries
- Work items that take months (should be initiatives)
- Coordination happening in side channels instead of documented

**Judgment call**: If you're fighting the structure, the structure might be wrong. But also consider whether you're avoiding necessary discipline.
