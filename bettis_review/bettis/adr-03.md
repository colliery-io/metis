# ADR-03: BACKWARD_TRANSITIVE as Default Schema Compatibility Mode

## Context

Bettis data contracts ([ADR-02](adr-02.md)) define schema compatibility as a per-boundary setting in each `contract.yaml` manifest. With the schema registry now in place and domains beginning to onboard their first Avro schemas, we need to establish a **platform-wide default compatibility mode** that new boundaries inherit unless explicitly overridden.

The compatibility mode governs what kinds of schema changes the registry will accept. Choosing too loose a default (e.g., `NONE`) risks breaking downstream consumers silently. Choosing too strict a default (e.g., `FULL_TRANSITIVE`) creates friction for producers making routine, safe changes. The default must balance **consumer safety** against **producer agility** for the majority of boundaries on the platform.

### Field Evolution Profile

The `ds-props` schema — the most mature and representative boundary in the system — serves as a useful reference for the kinds of evolution the platform needs to support. Its fields fall into six categories with distinct evolution profiles:

| Category | Field Examples | Evolution Risk |
|---|---|---|
| **ds-props computed output** | `projection`, `line`, `line_over_prob`, `is_optimal`, `is_balanced` | **Low** — intentional, team-controlled |
| **Event/entity metadata** | `sport_id`, `event_id`, `player_id`, `team_abbr`, `home` | **Low** — stable identifiers (cleanup needed: `name` vs `player_name`) |
| **Stat taxonomy** | `stat_type_id`, `stat_type`, `stat_group_id`, `is_derivative` | **Medium** — recent restructuring; new stat types add rows not columns |
| **Market status** | `market_open_id`, `market_suspended_id`, `manual_lock_market_suspended` | **Low** — column set stable; recent over/under suspension was additive |
| **Hashing/tracking** | `data_change_hash`, `market_uid`, `market_hash_uid` | **Low** — additive by nature |
| **Run metadata** | `sport`, `prop_type`, `run_mode`, `schema_version`, `timestamp` | **Very low** — envelope fields |

The dominant evolution pattern is **additive** (new optional fields, new enum values, new rows) with very little structural churn. The one medium-risk area — stat taxonomy — evolves by adding rows (new stat types), not by changing column shape. This profile strongly favors a backward-compatible default with transitive guarantees.

---

## Decision

**`BACKWARD_TRANSITIVE`** will be the platform default compatibility mode for all new schema boundaries registered in the Bettis schema registry. Individual boundaries may override this in their `contract.yaml` manifest with explicit justification reviewed by the platform team.

Under `BACKWARD_TRANSITIVE`:

- Every new schema version must be readable by consumers compiled against **any** previous version of that schema
- Compatibility is checked against **all prior versions**, not just the immediately preceding one
- **Permitted changes:** adding optional fields, adding new enum symbols (with a default), removing fields that have defaults
- **Prohibited changes:** removing required fields, renaming fields, changing field types, reordering union branches

### Override Mechanism

The override uses the existing `compatibility` field in `contract.yaml`:

```yaml
# Platform default — can be omitted
schema:
  file: schema.avsc
  compatibility: backward_transitive
```

```yaml
# Explicit override — requires platform team approval
schema:
  file: schema.avsc
  compatibility: full_transitive
```

---

## Alternatives Analysis

| | **BACKWARD** (non-transitive) | **BACKWARD_TRANSITIVE** | **FORWARD_TRANSITIVE** | **FULL_TRANSITIVE** | **NONE** |
|---|---|---|---|---|---|
| **What it checks** | New schema reads data from the immediately prior version only | New schema reads data from **all** prior versions | All prior schemas read data from the new version | Both backward and forward against all versions | No checks |
| **Pros** | Simplest mental model. Permits broader changes between adjacent versions. | Prevents "walking incompatibilities" where A→B and B→C are each compatible but A→C is not. Safe for consumers at any version. Standard Confluent recommendation for Kafka topics. | Producers can evolve freely; old consumers keep working. Useful for append-only logs. | Maximum safety — no version of any reader or writer can break. | Maximum flexibility. Useful for development or internal-only boundaries. |
| **Cons** | Permits version chains that are pairwise compatible but globally incompatible. Consumers that skip versions can break. | Slightly more restrictive than non-transitive — some two-step migrations require careful sequencing of schema versions. | Uncommon mental model. Consumers cannot add required fields. Doesn't protect against producer-side breaking changes. | Very restrictive — many safe, routine changes are rejected. Producer velocity suffers. Teams will request overrides frequently. | No safety net. Breaking changes are discovered at runtime by consumers. |
| **Risk Level** | Medium — silent breakage for lagging consumers | **Low** | Medium — unusual guarantee direction | Very low (breakage risk) / High (friction risk) | Very high |
| **Fit for Bettis** | Workable but leaves a gap for consumers that don't upgrade in lockstep | **Strong fit** — matches the additive-evolution profile and protects consumers regardless of version lag | Poor fit — Bettis consumers are the primary concern, not producers | Overly restrictive for the current evolution profile | Unacceptable for cross-boundary contracts |

---

## Rationale

`BACKWARD_TRANSITIVE` is the strongest fit for the current platform profile:

1. **The dominant evolution pattern is additive and low-risk.** Five of six field categories in the reference schema have low or very low evolution risk. The changes that actually happen — adding optional fields, adding new enum values, introducing new tracking hashes — are all natively compatible with `BACKWARD_TRANSITIVE`. The default will rarely be the bottleneck.

2. **Transitive matters more than non-transitive at this stage.** Bettis consumers will not all upgrade in lockstep. A non-transitive `BACKWARD` mode permits version chains where consumer A (two versions behind) silently breaks even though each intermediate upgrade was individually compatible. `BACKWARD_TRANSITIVE` closes this gap by checking against all prior versions, not just the last one.

3. **The stat taxonomy risk is already mitigated by the data model.** The one medium-risk area — stat type restructuring — evolves by adding rows, not by changing the schema shape. New stat types produce new records with the same column structure. `BACKWARD_TRANSITIVE` handles this naturally.

4. **`FULL_TRANSITIVE` was rejected as a default** because it would prohibit removing deprecated fields. The `ds-props` schema already has a known cleanup (`name` vs `player_name`) that `FULL_TRANSITIVE` would block without a compatibility override. Requiring overrides for routine deprecation creates unnecessary friction and review load on the platform team.

5. **`NONE` was rejected** because the entire purpose of data contracts is to prevent breaking changes from reaching consumers undetected. A default of `NONE` would undermine the contract system from the start.

6. **Per-boundary overrides preserve escape velocity.** Any boundary that genuinely needs a different mode (e.g., an internal log topic that benefits from `NONE`, or a billing boundary that warrants `FULL_TRANSITIVE`) can override in its `contract.yaml`. The override requires platform team review, creating a natural checkpoint for intentional deviations.

---

## Consequences

### Positive

- New boundaries get a safe, consumer-protective default without any per-boundary configuration — teams that don't think about compatibility still get a reasonable mode
- Consumers can lag behind producers by multiple schema versions without risk of deserialization failures
- "Walking incompatibility" bugs — where pairwise-compatible changes accumulate into a global incompatibility — are structurally prevented
- The override mechanism in `contract.yaml` keeps the decision visible and auditable per boundary

### Negative

- Some multi-step schema migrations require more careful version sequencing — a field rename (`add new` → `deprecate old` → `remove old`) must be spread across versions where each step individually satisfies transitive backward compatibility
- Teams accustomed to `NONE` or non-transitive `BACKWARD` in other environments may initially find the constraints surprising when a schema change is rejected by CI
- The platform team must document the "safe change" cookbook (what's allowed, what requires sequencing, what requires an override) before onboarding the first external domain team

### Neutral

- `BACKWARD_TRANSITIVE` is the Confluent-recommended default for Kafka topics — most tooling, documentation, and community knowledge assumes this mode, reducing the learning curve for teams familiar with the ecosystem
- The choice of `BACKWARD` over `FORWARD` reflects an explicit platform stance: **consumer stability is prioritized over producer flexibility**. This is a values decision, not a technical one, and should be revisited if the platform's consumer/producer dynamics change

---

## Review Schedule

### Review Triggers

- A boundary requests an override to `NONE` or `FORWARD` and the justification reveals a pattern not well-served by `BACKWARD_TRANSITIVE` (suggests the default itself may be wrong, not just the boundary)
- The stat taxonomy or another field category begins evolving in ways that require frequent multi-step migration sequences, indicating the default is creating more friction than safety
- A second schema format (SQL DDL) is onboarded and its evolution patterns differ substantially from Avro's (may need format-specific defaults)

### Scheduled Review

- **Next Review Date:** 6 months after the 5th domain's schemas are registered, or when the first override to a non-backward mode is approved — whichever comes first
- **Review Criteria:** How many overrides have been requested? Are they clustered in a specific pattern? Have any breaking changes slipped through despite the mode? Is the multi-step migration cookbook sufficient, or are teams routinely blocked by the transitive check?
- **Sunset Date:** N/A — this is a foundational platform default. It should be reviewed on triggers, not expired.
