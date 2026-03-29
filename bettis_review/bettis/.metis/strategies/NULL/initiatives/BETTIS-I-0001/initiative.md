---
id: prd-architectural-design
level: initiative
title: "PRD & Architectural Design"
short_code: "BETTIS-I-0001"
created_at: 2026-02-10T15:35:17.797186+00:00
updated_at: 2026-02-10T15:35:17.797186+00:00
parent: BETTIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: prd-architectural-design
---

# PRD & Architectural Design

## Context

The organization is building Bettis, an enterprise messaging bus that extends beyond Kafka to include database schemas (RDS) under the same contract-first governance (see BETTIS-V-0001). Multiple teams are already in weekly meetings discussing data contracts, but the conversation has been siloed around specific concerns (props table schemas, serialization formats, the alts problem) without a unifying product requirements document or architectural design to anchor decisions against. Meanwhile, Bytebase is being evaluated for DDL management — the same pattern (governed schema definitions, evolution rules, access control) applied to a different transport.

Guillaume's PRD draft, the Slack channel discussions, and prior art from a previous implementation all provide valuable input — but no single document currently frames the full platform requirements across both messaging and database boundaries, or identifies the architectural decisions that need to be made before implementation can begin.

This initiative produces the PRD and architectural design documents that will serve as conversational cornerstones for the technical leads group. The goal is to frame decisions, not make them — presenting the what, the why, and the constraining forces so the group can converge on a design that works for everyone.

## Goals & Non-Goals

**Goals:**
- Produce a PRD that defines Bettis platform requirements across all three capability areas (schema registry & distribution, boundary & access management, observability & archival) spanning both messaging (Kafka) and database (RDS) transports
- Produce an architectural design document that frames the key technical decisions, the options available, the trade-offs of each, and the constraints that narrow the decision space
- Identify and formally document each architectural decision that needs to be made as an ADR — capturing the question, the forces at play, and the options under consideration
- Ensure both documents are grounded in the actual requirements of the teams who will use Bettis (SWE, TRA, DS) — not designed around a single team's perspective

**Non-Goals:**
- Making technology selections — the documents frame decisions, they don't resolve them. That's the job of the technical leads group.
- Schema design for specific boundaries (Props, Action Queue, etc.) — those are customer concerns, not platform concerns
- Implementation planning or timeline — premature until architectural decisions are made
- Replacing or superseding Guillaume's PRD — his work is input to this, not something being overwritten

## Known Architectural Decisions Required

Five primary decision areas have been identified and fully framed in the architectural design document (03-architecture.md). Each is structured with constraints and required capabilities for the technical leads group to research. Additional emerging areas are catalogued for future ADRs.

### Primary Decision Areas (Framed)

1. **Serialization Format** — Root dependency. What format(s) for data contracts on the messaging transport? Constrains evolution, distribution, and organization decisions downstream.
2. **Schema Evolution Strategy** — How schemas change over time across both messaging and database boundaries. Compatibility rules, versioning scheme, breaking change process.
3. **Contract Organization** — Where schema definitions live, how they're organized, how the repo structure supports distributed ownership with centralized governance.
4. **Schema Distribution & Registry** — How validated schemas reach producers and consumers. Role of Confluent Schema Registry (deployed, not adopted) and Bytebase (under evaluation for DDL).
5. **Schema Lifecycle CI/CD** — Leaf dependency. How schema changes flow through validation, registration, and distribution. Key tension: standard GitOps patterns may bottleneck hot-path schema evolutions; custom tooling may be warranted for fast-path compatible changes.

### Emerging Decision Areas (Identified, Not Yet Framed)

- Message envelope design
- Boundary naming conventions
- Large payload patterns (signal-with-URI)
- Archival storage strategy
- Authentication & credential management
- Consumer framework
- DDL governance tooling (Bytebase integration)
- Corrective event patterns (supports REQ-3.1.6)
- Multi-environment strategy (supports REQ-1.3.4)
- Error handling patterns (supports REQ-3.4.1–3.4.3)
- Cross-transport lineage

## Approach

This initiative will proceed through the standard discovery → design flow:

1. **Discovery** — Gather requirements from each team (SWE, TRA, DS). Understand what each team needs from the platform, what they've already built or proposed, and where their constraints conflict. Review Guillaume's PRD, the Slack discussions, and the prior art as input.

2. **Design** — Draft the PRD and architectural design documents. For each architectural decision, produce an ADR that frames the question, the options, and the trade-offs. Circulate for review.

3. **Review** — Present to the technical leads group for feedback and iteration. The documents are conversation starters, not final answers.

## Deliverables & Current State

All four design documents are drafted and under active refinement:

- **00-overview.md** — What Bettis is, why it exists, platform capabilities, constraints, principles. *Drafted.*
- **01-context.md** — Actors, external systems, context diagram, key interactions. *Drafted.*
- **02-prd.md** — Platform requirements across five sections: Schema Registry & Distribution (REQ-1.x), Boundary & Access Management (REQ-2.x), Observability & Archival (REQ-3.x), Cross-Cutting Requirements (REQ-4.x), Non-Functional Requirements (NFR-5.x). Includes schema composability, multi-environment support, boundary retirement, error boundaries, corrective/retraction events, operational SLOs. *Drafted — ready for technical leads review.*
- **03-architecture.md** — Five primary decision areas framed with constraints and required capabilities (not options/recommendations). Dependency graph. Eleven emerging decision areas catalogued. *Drafted — ready for technical leads review.*
- **ADRs** — To be created by the technical leads group as they deliberate each decision area.