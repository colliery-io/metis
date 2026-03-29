---
id: enterprise-data-contracts
level: vision
title: "Enterprise Messaging Bus"
short_code: "BETTIS-V-0001"
created_at: 2026-02-10T14:47:24.749226+00:00
updated_at: 2026-02-10T15:35:10.052249+00:00
archived: false

tags:
  - "#vision"
  - "#phase/published"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Bettis — Enterprise Messaging Bus

## Purpose

Bettis is an enterprise messaging bus that provides the foundational infrastructure for structured, governed data exchange across all system boundaries in the organization. It is the system responsible for how data contracts are defined, how schemas are distributed to producers and consumers, how access is managed, how schema evolution is governed, and how data flow is made observable.

The bus extends beyond Kafka — it encompasses every transport layer where data crosses a boundary. Kafka topics and database schemas (RDS) are both stops on the same bus, governed by the same contract-first discipline: defined, versioned, validated, and distributed with the same rigor as application code.

## Current State

**Messaging (Kafka):**
- Services exchange data over Kafka with no unified schema management
- Confluent Schema Registry is deployed but not adopted — schemas appear in the registry but aren't linked to topics or enforced
- Each team defines and serializes data independently; no shared contract definitions
- No standard message envelope — messages lack consistent metadata for tracing, correlation, or provenance
- No topic naming convention across domains
- No access control model for who can produce to or consume from which topics
- No schema evolution enforcement — changes to data structures can break downstream consumers without warning
- No message archival or lineage infrastructure
- The "alts problem" demonstrates that current approaches can't handle production-scale data volumes (~70k row data frames exceed Kafka's ~2MB message limit)

**Database schemas (DDL):**
- Database schema changes are managed ad-hoc across teams with no unified governance
- DDL changes can break downstream consumers (applications, analytics pipelines, reports) without warning
- Bytebase is being evaluated for DDL management — applying the same contract-first pattern to database schema definitions, evolution, and access control

**Common across both:**
- No unified view of data contracts across the organization regardless of transport
- Schema ownership, evolution rules, and change management processes vary by team and boundary type

## Future State

Bettis is a platform composed of three core capabilities, applied across transport layers (messaging, database schemas, and future boundary types):

### 1. Schema Registry & Distribution

The central authority for data contract definitions across all boundary types.

- There must be a canonical location where schemas are defined, validated, versioned, and made available to producers and consumers — whether the schema describes a Kafka message, a database table, or another boundary type
- Teams must be able to discover what contracts exist, what data they describe, and what version they are at
- Producers and consumers across multiple languages must be able to work with contract definitions without each team implementing their own serialization or schema logic
- Schema changes must be validated against evolution and compatibility rules before they can affect consumers — broken or incompatible schemas must never reach production
- Schemas must be pullable into team build pipelines as a dependency, not copied or hand-maintained
- Not all schemas should be directly consumable (e.g., base types, shared definitions); the platform needs to control what is discoverable vs. what is internal
- The relationship between a schema definition and the boundary it applies to (topic, table, etc.) must be deterministic and enforced, not a manual mapping that can drift

### 2. Boundary & Access Management

Governs the data topology — what boundaries exist, who can write to them, and who can read from them. For Kafka this means topics and ACLs. For databases this means schemas, tables, and permissions.

- Access control changes must be auditable, reviewable, and reproducible; who has access to what must be knowable at any point in time
- Producers and consumers must be authenticated; the platform must know who is writing and reading at each boundary
- New boundaries must not be visible to consumers before the producer is ready — the platform needs safe onboarding workflows that prevent premature exposure
- Standing up a new consumer should be a configuration change, not a new application deployment
- Naming conventions must be systematic and derivable, not ad-hoc

### 3. Observability & Archival

Provides visibility into data flow across all boundary types.

- Data changes must be recoverable for audit, replay, and debugging
- Archival must capture not just the payload but the transport context appropriate to the boundary type — for Kafka: partition, offset, consumer lag, latency; for databases: migration history, change timestamps, applied-by context
- The platform must detect when boundaries, producers, or the transport layer are unhealthy — proactively, not when a consumer notices something is wrong
- Data must carry enough metadata to trace it from source to destination across system boundaries
- Data must be self-describing enough that a consumer can determine what type of data it contains and how to interpret it

Together these capabilities mean:

- **Producers** know where to define their data contracts, and those contracts are validated before they reach consumers — whether they're publishing Kafka messages or managing database tables
- **Consumers** can discover what data is available and retrieve schemas to work with it without guessing at structure
- **Platform operators** can manage access, monitor health, and enforce governance without manual intervention per boundary
- **Analytics** gets consistent metadata for tracing, lineage, and cross-system correlation

## Success Criteria

- All system boundaries in scope — messaging and database — are governed through Bettis-managed contracts
- Schema distribution is operational for all languages in use across teams
- Zero unplanned breaking changes — incompatible schemas are rejected before they reach consumers, whether the boundary is a Kafka topic or a database table
- Access control for all boundaries is managed through the platform — not applied manually
- Producer and consumer onboarding is self-service via documented workflows
- The platform provides sanctioned patterns for all data exchange needs, including large payloads that exceed transport limits
- Platform health issues are detected proactively, not reported by consumers
- Adding a new boundary type follows the same governance patterns without requiring platform redesign

## Principles

- **Schema as Infrastructure** — Data schemas are managed artifacts with the same rigor as application code: versioned, reviewed, tested, and deployed through a controlled process.
- **Open Formats** — The platform uses open, standardized serialization formats. No vendor lock-in. Compatibility with the broader analytics ecosystem is a hard requirement.
- **Contracts Over Conventions** — Producers and consumers should depend on machine-readable, distributable schema definitions — not documentation, naming conventions, or tribal knowledge. The schema is the single source of truth.
- **Backward Compatibility by Default** — Schema evolution must not break existing consumers. Incompatible changes are rejected unless explicitly overridden with documented justification.
- **Distributed Ownership, Centralized Platform** — Domain teams own their schemas and are accountable for their contracts. The platform team owns the infrastructure, tooling, and governance standards.
- **Observable by Design** — The platform provides the data lineage and tracing substrate that analytics and operations depend on. Observability is built in, not bolted on.

## Constraints

- **Multiple transport layers** — Bettis must support Kafka (MSK) for messaging and database DDL management (Bytebase under evaluation) as distinct boundary types under a common governance model. The platform design must not be so tightly coupled to one transport that adding another requires rearchitecting.
- **Existing infrastructure** — Confluent Schema Registry is deployed for Kafka (though not actively adopted). Bytebase is being evaluated for DDL. Any design must account for what's already in place.
- **Active production systems** — Props, Exchange Trading, databases, and other boundaries are live. Migration must be incremental. Bettis must coexist with current approaches during rollout.
- **Kafka message size limit (~2MB)** — Serialization and batching strategy for messaging must account for this hard constraint, particularly for large data frames.
- **Multi-language environment** — Multiple teams work in different languages. Schema distribution must serve all of them.
- **Multiple teams with different priorities** — SWE, TRA, DS each have distinct requirements for data exchange. The platform must serve all of them without being designed around any single team's workflow.
