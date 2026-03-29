# ADR-04: Schema Distribution, Registry & Lifecycle

## Context

With Avro as the format ([ADR-01](adr-01.md)), a monorepo for contract organization ([ADR-02](adr-02.md)), and evolution rules defined ([ADR-03](adr-03.md)), the platform needs to decide how validated schemas are distributed to producers and consumers, what role the Schema Registry plays, and how the CI/CD pipeline enforces governance.

These are one decision, not two. The CI/CD pipeline populates the registry, and the registry distributes schemas. Without the pipeline, the registry has no content. Without the registry, there's nothing to distribute.

Avro's reader/writer resolution model makes the Schema Registry a **participant in the data path**, not optional infrastructure. There is no compilation step. The pipeline's job is: validate schemas, check compatibility, register to the Schema Registry, and sync infrastructure (topics, ACLs).

---

## Decision

### Registry: Karapace (Apache 2.0), Self-Hosted

Karapace is the schema storage and distribution mechanism, backed by a Kafka topic (`_schemas`) for durability. It is API-compatible with the Confluent Schema Registry (tested against 6.1.1) — all Confluent client libraries work against it without modification.

### Distribution: Runtime Resolution

Producers and consumers resolve schemas at runtime via the Schema Registry. No compiled bindings, no language-specific packages, no download step. The Confluent wire format (magic byte + 4-byte schema ID + Avro binary payload) is the on-wire standard.

**Supported clients:**

- `confluent-kafka` (Python)
- `confluent-kafka-go/v2/schemaregistry` (Go)
- `schema-registry-client` crate (Rust)

All support the Confluent wire format, schema caching, and schema references.

### Subject Naming: `TopicNameStrategy`

Subject name is `{topic}-value`. Example: topic `props_player_props_inplay_v1_MarketUpdate` → subject `props_player_props_inplay_v1_MarketUpdate-value`.

### Compatibility: `BACKWARD_TRANSITIVE`

All subjects use `BACKWARD_TRANSITIVE` as established in [ADR-03](adr-03.md). The CI pipeline enforces this at both PR validation and post-merge registration.

### The Registry Is a Derived Artifact, Not Source of Truth

Git (the monorepo, per [ADR-02](adr-02.md)) is the source of truth. The Schema Registry is a **materialized view** optimized for runtime access.

Write access is restricted via Karapace's ACL system — the CI service account is the only principal with `schema_registry_write`. All application and developer accounts are `schema_registry_read` only. This is per-principal and permanent, not a global mode toggle.

The registry is derivable from git at any point. If it drifts or is lost, it can be rebuilt by replaying schema registrations from git history in commit order. The registry cannot reject something git has accepted — CI validates compatibility before merge; the registry's check at registration time is a safety net.

---

## Shared Type References and Version Pinning

Domain schemas frequently reference types defined in `_shared/` (e.g., `decimal.avsc`, `dollar.avsc`). The Schema Registry's reference support requires that referenced schemas are registered first, and that the referencing schema declares exactly which registry subject and version it depends on. This creates a dependency ordering problem and a version coordination problem.

### `refs.yaml`

Each domain that references `_shared/` types maintains a `refs.yaml` file in its domain directory. This file declares the exact version pin for every `_shared/` type the domain depends on:

```yaml
# contracts/props/refs.yaml
references:
  - name: bettis.shared.types.Dollar
    subject: _shared_types_dollar-value
    version: 3
  - name: bettis.shared.types.Decimal
    subject: _shared_types_decimal-value
    version: 2
```

The `name` is the Avro fully-qualified type name used in the domain's `.avsc` files. The `subject` and `version` identify the exact registry entry. These pins are passed as the `references` array in the Schema Registry's registration API.

### Why Explicit Pins

Without pinning, a domain implicitly depends on whatever the latest version of a `_shared/` type happens to be at registration time. This creates two problems:

1. A `_shared/` type change can silently alter a domain's registered schema without any change in the domain's own files
2. Compatibility checks can't be run accurately during PR validation because the reference target is a moving target

With explicit pins, a domain's schema is **fully deterministic** from its own directory contents. Updating a dependency is an intentional act — a PR that bumps the version in `refs.yaml`, which triggers compatibility checks against the new reference version.

### Registration Order

The pipeline builds a dependency DAG from `refs.yaml` across all domains, topologically sorts, and registers `_shared/` types before the schemas that reference them.

### CI Enforcement

The PR validation Refs Validation step checks that every `_shared/` type referenced in a domain's `.avsc` files has a corresponding pin in `refs.yaml`. Missing pins fail the build. Stale pins (entries not referenced by any schema) generate warnings.

When a `_shared/` type changes, the compatibility check expands scope to all schemas across all domains that pin that type, ensuring the change is safe for all downstream consumers.

---

## Two-Phase CI/CD Pipeline

The pipeline operates over the `contracts/` directory structure from [ADR-02](adr-02.md).

### Phase 1: PR Validation

Triggered on every PR touching `contracts/`, `Accessfile.yaml`, or registry configuration. All checks produce a report and exit code (`0` pass, `1` fail). PR cannot merge unless all pass.

| Step | Description |
|---|---|
| **Schema Lint** | JSON validity, naming conventions (`PascalCase` records, `snake_case` fields, correct namespace), required metadata (`doc`, `x-schema-owner`, `x-schema-origin`), nullable field pattern (`["null", T]` + default). |
| **Schema Parse** | Avro structural validity. Reference resolution against `refs.yaml` pins. |
| **Refs Validation** | Pin completeness and staleness checks (see above). |
| **Compatibility Check** | Calls `/compatibility/subjects/{subject}/versions/latest?verbose=true` (read-only). Returns per-schema `COMPATIBLE`, `INCOMPATIBLE` (with field-level errors), or `NEW`. Exit code `1` if any incompatible. |
| **OWNERS Check** | PR author or approver in `OWNERS.yaml` per [ADR-02](adr-02.md). Platform team can approve any domain. |
| **Accessfile Validation** | ACL entries reference valid topic names. |

### Phase 2: Post-Merge

Merge to `main` is the release event. No release branch, no promotion pipeline.

| Step | Description |
|---|---|
| **Register All Schemas** | Full pass: `_shared/` types first (dependency order from `refs.yaml`), then domain schemas with reference pins. Registry deduplicates unchanged schemas. |
| **Create/Update Topics** | New schemas get a Kafka topic (name from namespace + record). Partition count and compaction from defaults or per-domain overrides. |
| **Sync ACLs** | Diff `Accessfile.yaml` against live Kafka ACLs, apply changes. |
| **Produce Schema Cache Artifact** | Snapshot full registry state tagged with git SHA (see below). |
| **Notify** | Summary to Slack/Teams. |

---

## Topic Name Derivation

Deterministic, reversible mapping from Avro namespace to Kafka topic:

| Component | Value |
|---|---|
| **Namespace** | `props.player_props_inplay.v1` |
| **Record** | `MarketUpdate` |
| **Topic** | `props_player_props_inplay_v1_MarketUpdate` |

Dots become underscores in the namespace portion; record name appended as-is.

---

## Environment Strategy: Mirrored Registries

Each environment has its own registry instance, all populated by the same pipeline on merge to `main`. Schema IDs may differ across environments (auto-assigned), but consumers resolve by ID from their local registry. There is no per-environment schema testing — CI validates everything at PR time.

---

## Fast Path vs. Slow Path

| | **Fast Path** (backward-compatible changes) | **Slow Path** (breaking changes, new domains, `_shared/` type changes) |
|---|---|---|
| **Workflow** | Modify `.avsc` in your domain. PR validation runs (<2 min). One domain team approver. Merge triggers registration. | Create `v2/` with new schemas. Requires platform team approval + migration plan. Old version stays active during transition. |
| **Timeline** | Minutes to hours | Days to weeks |

---

## New Schema Onboarding

A single PR takes a team from nothing to a live, governed topic:

- `.avsc` file
- `contract.yaml` sidecar
- `OWNERS.yaml` (if new domain)
- `refs.yaml` entries (if referencing `_shared/` types)
- `Accessfile.yaml` producer ACL entries

CI validates; post-merge creates topic, registers schema, syncs ACLs. Consumers submit their own PR adding consumer ACL entries.

---

## Schema Cache Artifact

Every post-merge produces a cache artifact — a snapshot of all subject → version → schema ID → schema content mappings, tagged with git SHA, published to an artifact store.

- **Runtime resilience:** Applications pre-warm their schema cache at startup. The registry only needs to be reachable for schemas registered after the artifact was built.
- **Local testing:** Developers pull the artifact to test against real schemas without registry access.

The artifact is **not** in the critical path — registration succeeds regardless of artifact publish.

---

## Registry Rebuild

The registry can be reconstructed by replaying commits that touched `contracts/`, producing full version history. For recovery where messages carry specific schema IDs, the `_schemas` Kafka topic backup (which preserves exact ID assignments) is the appropriate path.

---

## Rationale

1. **Why runtime resolution?** Avro's reader/writer model is designed for it — a core reason for choosing Avro in [ADR-01](adr-01.md). Compiled bindings would require code generation and distribution pipelines that fight the standard Kafka + Avro path.

2. **Why Karapace?** Functionally equivalent to Confluent Schema Registry, but Apache 2.0 licensed with native per-principal ACLs. All Confluent client libraries work unmodified. Switching to Confluent is a configuration change if needed.

3. **Why `TopicNameStrategy`?** Bettis enforces one-message-one-topic. `RecordNameStrategy` shares subjects across topics using the same record, preventing independent evolution during repartitioning. `TopicNameStrategy` preserves per-topic compatibility tracking.

4. **Why two phases?** PR validation is fast and read-only. Post-merge handles write-access side effects.

5. **Why register all on every merge?** Self-healing — no drift, no change detection logic. Registry deduplication makes cost negligible.

6. **Why single onboarding PR?** One PR, one review, live topic. Supports NFR-5.4.1 onboarding SLO.

7. **Why git as source of truth?** PRs require review, CI gates enforce rules. The registry's REST API has no equivalent governance. Git provides auditability and recoverability.

---

## Consequences

### Positive

- No code generation or language-specific packaging — distribution is the registry
- Any language with a Schema Registry client participates without platform-specific tooling
- Every change validated automatically: structure, compatibility, naming, ownership, reference integrity
- New schema onboarding is a single PR — time-to-first-message bounded by review latency
- The pipeline is the single enforcement point for all governance ([ADR-03](adr-03.md) evolution rules, compatibility, naming, ownership)
- Self-healing: register-all-on-merge converges registry to git state
- Explicit `refs.yaml` pinning makes shared type dependencies deterministic and auditable

### Negative

- Schema Registry is a runtime dependency for uncached schema IDs — mitigated by cache artifact and Kafka-backed durability
- Schema IDs differ across environments — use subject + version for cross-environment comparison
- PR validation requires a running registry — mitigated by HA deployment
- Post-merge pipeline has privileged write access (topics, ACLs, schemas) — service account must be secured
- Cross-reference compatibility expansion (when `_shared/` types change) is the most complex CI logic

### Neutral

- CI tooling is custom Python scripts (~200–300 lines each), maintained by platform team
- A Bettis Catalog API for consumer-friendly discovery is optional and not in the data path
- `refs.yaml` is a new artifact introduced by this ADR, co-located per domain in `contracts/`

---

## Review Schedule

### Review Triggers

- Karapace becomes an operational burden or falls behind Confluent API compatibility
- Schema registry availability becomes a measured production risk
- Register-all-on-merge pipeline latency exceeds acceptable thresholds
- Team count exceeds the monorepo scaling threshold identified in [ADR-02](adr-02.md)

### Scheduled Review

- **Next Review Date:** Aligned with [ADR-02](adr-02.md) — 6 months after first domain schemas are merged, or when the 5th domain is onboarded, whichever comes first
- **Review Criteria:** Is the two-phase pipeline fast enough? Is the cache artifact being consumed? Is Karapace stable?
- **Sunset Date:** No sunset date
