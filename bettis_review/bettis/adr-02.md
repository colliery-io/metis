# ADR-02: Monorepo Structure for Data Contract Definitions

## Context

Bettis is introducing data contracts to govern data exchange across organizational boundaries (Kafka messaging and RDS databases). As a greenfield initiative, we need to establish where schema definitions (Avro `.avsc` files today, SQL DDL in the future) live in source control, how they are organized, and how the repository structure supports **distributed team ownership** with **centralized platform governance**.

There is no existing schema organization today. Teams have no standardized location for schema definitions, no enforced ownership model, and no governed change workflow for data contracts. As the number of teams defining schemas grows (4 today, potentially 10 within 1–2 years), the organizational model must be established before adoption begins — retrofitting structure onto an already-populated schema landscape is significantly harder than starting organized.

This decision directly affects the day-to-day developer workflow for every team, the CI/CD pipeline design for schema validation and distribution, and the viability of cross-boundary schema changes (e.g., updating a shared type definition that multiple domain schemas depend on).

---

## Decision

All data contract definitions will live in a **single monorepo**, organized by domain with boundary types co-located under each domain. Shared/internal definitions will live in a platform-owned directory. Ownership and documentation metadata will be tracked via sidecar manifest files (`contract.yaml`) rather than embedded in schema files.

### Repository Structure

```
contracts/
├── _shared/                            # Platform-owned shared/internal definitions
│   ├── types/
│   │   ├── decimal.avsc
│   │   └── dollar.avsc
│   ├── envelopes/
│   │   └── base-envelope.avsc
│   └── contract.yaml
│
├── betradar/                           # Domain: BetRadar
│   ├── messaging/
│   │   └── gamestate/
│   │       ├── schema.avsc
│   │       └── contract.yaml
│   ├── database/
│   │   └── gamestate/                  # ? not sure if this exists, just an example
│   │       ├── schema.sql
│   │       └── contract.yaml
│   ├── internal/
│   │   └── database/
│   │       └── gamestate-logs/         # ? not sure if this exists, just an example
│   │           ├── schema.sql
│   │           └── contract.yaml
│   └── OWNERS.yaml
│
├── push-api/                           # Domain: Push API
│   ├── messaging/
│   │   └── <boundary-name>/
│   │       ├── schema.avsc
│   │       └── contract.yaml
│   ├── database/
│   │   └── <boundary-name>/
│   │       ├── schema.sql
│   │       └── contract.yaml
│   ├── internal/
│   │   └── database/
│   │       └── client-logs/
│   │           ├── schema.sql
│   │           └── contract.yaml
│   └── OWNERS.yaml
│
├── akm/                                # Domain: AKM
│   ├── internal/
│   │   └── database/
│   │       ├── client-organizations/
│   │       │   ├── schema.sql
│   │       │   └── contract.yaml
│   │       └── api-keys/
│   │           ├── schema.sql
│   │           └── contract.yaml
│   └── OWNERS.yaml
│
├── action-queue/                       # Domain: action queue
│   ├── messaging/
│   │   └── <boundary-name>/
│   │       ├── schema.avsc
│   │       └── contract.yaml
│   ├── database/
│   │   └── <boundary-name>/
│   │       ├── schema.sql
│   │       └── contract.yaml
│   └── OWNERS.yaml
│
├── props/                              # Domain: Props
│   ├── messaging/
│   │   └── <boundary-name>/
│   │       ├── schema.avsc
│   │       └── contract.yaml
│   ├── database/
│   │   └── <boundary-name>/
│   │       ├── schema.sql
│   │       └── contract.yaml
│   └── OWNERS.yaml
│
├── suspensions/                        # Domain: Suspensions
│   ├── messaging/
│   │   └── <boundary-name>/
│   │       ├── schema.avsc
│   │       └── contract.yaml
│   ├── database/
│   │   └── <boundary-name>/
│   │       ├── schema.sql
│   │       └── contract.yaml
│   └── OWNERS.yaml
│
├── CODEOWNERS                          # GitHub-enforced review ownership
└── GOVERNANCE.md                       # Platform-wide conventions and rules
```

### What Constitutes a "Domain"

A domain is the system, service, or product that is the **authoritative owner** of the data. It maps to whatever the organization already calls the thing — not to a team, not to a business capability taxonomy, and not to an individual boundary. Examples: `betradar` (an external data integration), `props` (a product area), `akm` (an internal service), `action-queue` (a shared messaging boundary).

This is a pragmatic choice. It avoids requiring agreement on a formal business capability model before contracts can be organized, and it maps to how teams already refer to these systems. If domain boundaries shift as the organization matures, directories can be renamed or consolidated — the review triggers below account for this.

### Key Design Elements

**Ownership via `CODEOWNERS` + `OWNERS.yaml`:** GitHub `CODEOWNERS` enforces who must review changes. Each domain's `OWNERS.yaml` captures the owning team and contact information for discoverability. `_shared/` is owned by the platform team; domain directories are owned by their respective domain teams.

**Sidecar manifests (`contract.yaml`):** Each boundary directory contains a `contract.yaml` that captures ownership, description, schema-to-boundary mapping, compatibility mode, and visibility (public vs. internal). Keeping governance metadata separate from the schema files themselves makes the approach portable across Avro and SQL DDL without coupling metadata conventions to a specific schema format.

**Shared definitions in `_shared/`:** Base types (e.g., `decimal`, `dollar`), common envelopes, and reusable components live under `_shared/`. Domain teams propose additions via PR; the platform team reviews and approves. Changes to `_shared/` trigger revalidation of all dependent domain schemas in CI.

**Co-located boundary types:** A domain's messaging schemas and database schemas live under the same domain directory, separated by a boundary type subdirectory (`messaging/`, `database/`). This keeps a domain's full contract surface discoverable in one place.

**Public vs. internal distinction:** Each domain can contain an `internal/` subdirectory alongside `messaging/` and `database/`. Boundaries under `internal/` are governed by the same standards but are not part of the domain's public contract surface — they represent implementation-detail boundaries (e.g., internal log tables, intermediate processing topics) that still benefit from schema governance but should not be discoverable by external consumers. Schemas in `_shared/` are also inherently internal (composable building blocks). The three-level visibility model is:

- `_shared/` — platform-internal, composable types
- `<domain>/internal/` — domain-internal, governed but not published
- `<domain>/messaging/` or `<domain>/database/` — public, consumable boundaries

### Illustrative `contract.yaml`

```yaml
owner:
  team: DSP
  contact: @DS_Product
boundary:
  type: kafka-topic
  name: props.player-props-inplay.v1
visibility: public
description: >
  Emitted when inplay player prop markets are created or updated.
schema:
  file: schema.avsc
  compatibility: backward
```

*(The exact manifest schema is a downstream design detail — this illustrates the kind of metadata it captures.)*

---

## Change Workflows

| Change Type | Workflow |
|---|---|
| **New boundary schema in your domain** | PR into your domain directory. Domain team + CI approval. |
| **Modify existing boundary schema** | PR into your domain directory. CI validates compatibility. Domain team reviews. |
| **Add/modify a shared definition in `_shared/`** | PR into `_shared/`. Platform team reviews. CI revalidates all dependent schemas. |
| **Cross-domain change** (e.g., shared type update that affects multiple domains) | Single PR touching `_shared/` and affected domain schemas. Platform team + affected domain teams review. Atomic commit. |
| **New domain onboarding** | Create domain directory + `OWNERS.yaml`. Add entry to `CODEOWNERS`. Platform team reviews structure. |

---

## Alternatives Analysis

| | **Single Monorepo** (domain-organized) | **Polyrepo** (one repo per domain) | **Monorepo per Boundary Type** |
|---|---|---|---|
| **Pros** | Single source of truth. Atomic cross-boundary changes. `CODEOWNERS` enforces ownership naturally. One CI pipeline. Discoverability is trivial (browse the repo). Simplest onboarding story. | Clear ownership via repo-level permissions. Full team autonomy over their repo. CI scoped to one team's schemas. | Natural separation of transport-specific tooling. CI pipelines can be transport-specific. |
| **Cons** | All teams share merge contention in one repo. CI must use path-based triggering to avoid unnecessary validation. Repo size grows with all schemas. | Shared definitions require a separate repo consumed as a dependency. Cross-boundary changes are not atomic (multi-repo PRs). Discoverability requires a catalog across repos. Violates single-source-of-truth principle. More CI complexity (N+1 pipelines). | A domain's contracts are split across repos. Shared definitions are duplicated or extracted to a third repo. Cross-transport changes for the same domain are not atomic. |
| **Risk Level** | **Low** | Medium | Medium |
| **Implementation Cost** | **Low** — single repo, single pipeline, standard GitHub tooling | Medium-High — repo-per-domain provisioning, cross-repo dependency management, catalog service | Medium — two repos, two pipelines, cross-repo coordination |

---

## Rationale

The monorepo approach is the strongest fit for the stated constraints:

1. **Single source of truth is a hard requirement.** The monorepo is the only option that achieves this literally — all contract definitions are in one place, with one history, and one review workflow.

2. **Cross-boundary changes must be atomic.** When a shared type like `dollar` changes, the monorepo allows a single PR to update the type and all affected domain schemas. Polyrepo requires coordinated multi-repo changes with no atomicity guarantee.

3. **At 4–10 teams, a monorepo is well within its scaling sweet spot.** Merge contention and repo size are not realistic concerns at this scale. If the organization grows to 50+ teams, this decision can be revisited, but that is not the planning horizon.

4. **Onboarding simplicity.** A new team learns one repo, one directory convention, one review process. There is no question of "which repo has the schemas I need."

5. **GitHub `CODEOWNERS` is sufficient for ownership enforcement.** No custom tooling is needed to implement "distributed ownership, centralized standards" — `CODEOWNERS` gives domain teams authority over their directories while requiring platform team review for shared definitions.

The polyrepo option was rejected primarily because it trades atomicity and discoverability for autonomy that isn't needed at current scale. The monorepo-per-boundary-type option was rejected because it splits a domain's contract surface across repos for no clear benefit.

---

## Consequences

### Positive

- Single, browsable source of truth for all data contracts across the organization
- Cross-domain and shared-definition changes are atomic — a single PR, a single review, a single merge
- Ownership is unambiguous and enforced via `CODEOWNERS` without custom tooling
- New domain onboarding is a directory creation + `CODEOWNERS` entry — no infrastructure provisioning
- Sidecar manifests decouple governance metadata from schema format, easing the future addition of SQL DDL alongside Avro

### Negative

- All teams are contributors in a single repo — merge conflicts are possible during high-activity periods, though unlikely at current scale
- CI pipeline must be path-aware (only validate schemas affected by a given change) to avoid slow feedback loops as the repo grows
- The platform team becomes a review bottleneck for `_shared/` changes — mitigated by keeping the shared surface small and well-defined

### Neutral

- Domain directory names become part of the organizational vocabulary; renaming a domain is possible but creates churn in references, CI config, and `CODEOWNERS`
- The `contract.yaml` manifest format itself becomes a platform-governed schema — its structure will need to be defined and versioned

---

## Review Schedule

### Review Triggers

- Domain boundaries solidify and the current directory structure no longer reflects actual ownership (e.g., teams reorganize, domains merge or split)
- Team count exceeds 10 and merge contention or CI performance becomes a measurable problem
- SQL DDL is actively onboarded and the co-located structure proves awkward for database-specific tooling (e.g., Bytebase integration)

### Scheduled Review

- **Next Review Date:** 6 months after first domain schemas are merged, or when the 5th domain is onboarded — whichever comes first
- **Review Criteria:** Is the directory structure still intuitive? Is CI performance acceptable? Are cross-boundary changes working as expected? Has the domain model stabilized enough to confirm or revise directory names?
- **Sunset Date:** N/A — this is a structural decision, not a temporary measure. It should be reviewed on triggers, not expired.
