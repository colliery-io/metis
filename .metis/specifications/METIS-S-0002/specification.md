---
id: code-architecture-review-framework
level: specification
title: "Code Architecture Review Framework Specification"
short_code: "METIS-S-0002"
created_at: 2026-03-28T14:13:58.962661+00:00
updated_at: 2026-03-28T14:13:58.962661+00:00
parent: METIS-I-0029
blocked_by: []
archived: false

tags:
  - "#specification"
  - "#phase/discovery"


exit_criteria_met: false
initiative_id: NULL
---

# Code Architecture Review Framework Specification

## Overview

A systematic code architecture review framework conducted by an orchestrated set of agents. Evaluates a codebase across seven lenses in two passes, producing a structured report with actionable recommendations. Invoked via `/review-architecture`.

### Purpose

This framework provides a systematic approach to evaluating a codebase across seven lenses, organized in two passes. The goal is to answer two fundamental questions:

1. **Is this built in a way that's understandable, maintainable, and usable?**
2. **Does it work the way we think it does?**

## The Seven Lenses

### Pass 1: Foundational (Internal Quality)

These lenses evaluate the codebase *as a piece of software engineering* — independent of how it's deployed or consumed.

| Lens | Core Question | Evaluates |
|------|---------------|-----------|
| **Legibility** | Can a newcomer understand what this does and why? | Naming, structure, code organization, comments, separation of concerns, cognitive load, implicit vs explicit knowledge |
| **Correctness** | Does this do what it claims, under all expected conditions? | Test coverage, edge cases, invariants, error handling, contracts, type safety, undefined behavior |
| **Evolvability** | Can this be changed safely and confidently? | Modularity, coupling, cohesion, abstraction boundaries, dependency management, test quality (not just coverage), feature flags, migration paths |
| **Performance** | Does this use resources appropriately for its workload? | Algorithmic complexity, memory allocation patterns, I/O efficiency, concurrency model, hot paths, premature optimization vs necessary optimization |

### Pass 2: External (Interface & Operational Quality)

These lenses evaluate the codebase *as something that gets deployed, consumed, and attacked* — its surface area and runtime behavior.

| Lens | Core Question | Evaluates |
|------|---------------|-----------|
| **API Design** | Is the exposed interface right for its consumers? | Ergonomics, consistency, discoverability, error semantics, versioning strategy, backward compatibility, documentation alignment, principle of least surprise |
| **Operability** | Can this be run, observed, debugged, and scaled in production? | Configuration management, logging, metrics, tracing, health checks, graceful degradation, deployment strategy, failure modes, runbooks |
| **Security** | Is this safe against misuse, abuse, and attack? | Trust boundaries, input validation, authentication/authorization model, secrets management, dependency vulnerabilities, data sensitivity, threat model alignment |

## Orchestration Model

This review is conducted by a **Coordinator Agent** that launches specialized agents for each phase. All agents use the filesystem as working memory, writing findings to structured markdown files.

### File Structure

```
/review/
├── 00-system-overview.md      # Phase 1: Discovery
├── 01-legibility.md           # Phase 2: Foundational
├── 02-correctness.md
├── 03-evolvability.md
├── 04-performance.md
├── 05-api-design.md           # Phase 3: External
├── 06-operability.md
├── 07-security.md
├── 08-cross-cutting.md        # Phase 4: Cross-cutting analysis
├── 09-report.md               # Phase 5: Synthesis
└── 10-recommendations.md
```

### Severity Classification

All agents use this severity taxonomy:

| Severity | Definition | Action |
|----------|------------|--------|
| **Critical** | Blocks production use or represents active risk. Data loss, security vulnerability, correctness failure in core path. | Must fix before release. |
| **Major** | Significant impact on quality, maintainability, or safety. Not immediately dangerous but will cause pain. | Fix in current cycle. |
| **Minor** | Should be fixed but doesn't block progress. Code smells, inconsistencies, minor inefficiencies. | Fix when touching related code. |
| **Observation** | Worth noting. May inform future decisions, patterns to watch, or context for later. | Document only. |

### Finding Format

All agents write findings in this consistent format:

```markdown
## [LENS]-[ID]: [One-line summary]

**Severity**: Critical | Major | Minor | Observation
**Location**: [file paths, function names, line numbers, or architectural scope]
**Confidence**: High | Medium | Low

### Description

[Detailed explanation of the finding. What is the issue? Why does it matter through this lens?]

### Evidence

[Specific code references, snippets, or traces that support this finding. Be concrete.]

### Suggested Resolution

[How might this be addressed? Not a full design, but enough to show a path forward.]
```

## Agent Specifications

### Coordinator Agent

The Coordinator Agent manages the entire review process. It does not perform analysis itself — it launches specialist agents, monitors progress, and ensures cross-phase dependencies are respected.

**Responsibilities:**
1. **Phase Management**: Execute phases in order, respecting dependencies
2. **Agent Launching**: Launch specialist agents with correct context and instructions
3. **Progress Tracking**: Verify each phase completes before proceeding
4. **Conflict Resolution**: Handle cross-cutting findings that span multiple lenses
5. **Quality Assurance**: Ensure all agents follow the finding format and severity taxonomy

**Phase Execution:**

- **Phase 1: Discovery** — Launch Discovery Agent. Wait for `00-system-overview.md`. Do not proceed until it contains substantive content.
- **Phase 2: Foundational Review** — Launch Legibility, Correctness, Evolvability, and Performance agents IN PARALLEL. Each must read `00-system-overview.md` before beginning. Wait for all four files before proceeding.
- **Phase 3: External Review** — Launch API Design, Operability, and Security agents IN PARALLEL. Each must read the system overview AND all Phase 2 findings. Wait for all three files before proceeding.
- **Phase 4: Cross-Cutting Analysis** — Launch Cross-Cutting Agent. Reads all findings from Phases 2 and 3. Identifies findings spanning multiple lenses, traces symptoms to root causes. Wait for completion.
- **Phase 5: Synthesis** — Launch Synthesis Agent. Reads all findings and cross-cutting analysis. Produces the final report and recommendations.

**Error Handling:**
- If an agent fails to produce output: retry once, then note the gap in synthesis and proceed.
- If an agent produces malformed output: extract what's usable, note format issues for manual review.

**Completion:** The review is complete when both `09-report.md` and `10-recommendations.md` exist with substantive content.

### Phase 1: Discovery Agent

Explores the entire codebase and produces a System Overview document. This document is read by all subsequent agents, so it must be thorough and accurate.

**Exploration Checklist:**
1. **Repository Structure** — Top-level directories, organizational principle
2. **Entrypoints** — Main entrypoints, startup/shutdown, modes of operation
3. **Configuration** — Config files, environment variables, loading, validation, defaults
4. **Dependencies** — External dependencies, internal module relationships, dependency graph
5. **Primary Workflows** — User-facing workflows traced from input to output, happy and error paths
6. **Public Interface Surface** — APIs (REST, gRPC, library, CLI), config surface, events
7. **Build, Test, and Deployment** — Build process, test types, deployment configurations
8. **Patterns and Conventions** — Architectural patterns, consistency, naming conventions, implicit conventions
9. **Context and Constraints** — Design decision documentation, ADRs, constraints, historical context

**Output:** `00-system-overview.md` with sections: Summary, Repository Structure, Key Entrypoints, Architecture, Primary Workflows, Public Interface Surface, Dependency Graph, Build and Deployment, Conventions and Implicit Knowledge, Open Questions.

**Key rule:** Don't evaluate yet. This phase is about understanding, not judgment.

### Phase 2: Foundational Review Agents

#### Legibility Agent

**Lens:** Can a newcomer understand what this does and why?

**Evaluation Criteria:**
- **Naming** — Accuracy, consistency, abbreviation clarity, similar/different naming
- **Structure** — File/module organization, findability, hierarchy, grouping
- **Abstraction** — Well-chosen boundaries, unnecessary indirection, missing abstraction, leaky abstractions
- **Cognitive Load** — Mental model requirements, function/file length, clever tricks, control flow clarity
- **Implicit Knowledge** — Undocumented conventions, historical reasons, newcomer surprises
- **Documentation** — Comments explaining why not what, accuracy, module/function docs

**Process:** Start at main entrypoints, follow primary workflows noting where understanding breaks down, examine module structure, look at naming patterns, identify most complex areas.

**Output:** `01-legibility.md` — Summary, Key Themes, Findings

#### Correctness Agent

**Lens:** Does this do what it claims, under all expected conditions?

**Evaluation Criteria:**
- **Test Coverage** — Right things tested, behavior vs implementation tests, edge cases, error paths
- **Test Quality** — Real assertions vs smoke tests, determinism, isolation, flakiness
- **Error Handling** — Failure modes, propagation, actionability, recovery, silent failures
- **Invariants and Contracts** — Encoded vs assumed invariants, type-level guarantees, preconditions/postconditions
- **Edge Cases** — Empty inputs, null/None, maximums, concurrency, malformed inputs, boundary conditions
- **Undefined Behavior** — Unvalidated assumptions, unsafe code soundness, race conditions
- **Logic** — Algorithm correctness, off-by-one errors, boolean logic, state machine completeness

**Process:** Start with test suite, trace primary workflows asking "what could go wrong," examine error handling consistency, find complex logic and scrutinize it.

**Output:** `02-correctness.md` — Summary, Test Coverage Assessment, Key Risk Areas, Findings

#### Evolvability Agent

**Lens:** Can this be changed safely and confidently?

**Evaluation Criteria:**
- **Modularity** — Clear boundaries, self-contained modules, appropriate sizing, clear responsibilities
- **Coupling** — Interconnectedness, change isolation, hidden dependencies, data structure coupling
- **Cohesion** — Related functionality grouping, mixed-concern modules, split functionality
- **Abstraction Boundaries** — Right placement, implementation replaceability, interface stability, leakiness
- **Dependency Management** — Version management, dependency isolation, replaceability, circular dependencies
- **Test Architecture** — Refactoring support, implementation coupling, documentation value
- **Change Patterns** — Cost of representative changes (new feature, bug fix, refactor), blast radius
- **Migration Support** — Gradual migration, side-by-side operation, data format changes

**Process:** Identify architectural boundaries, assess coupling, mentally trace representative changes, examine dependency graph, evaluate test architecture.

**Output:** `03-evolvability.md` — Summary, Architecture Assessment, Change Cost Analysis, Findings

#### Performance Agent

**Lens:** Does this use resources appropriately for its workload?

**Evaluation Criteria:**
- **Algorithmic Complexity** — Hot path complexity, data structure appropriateness, unnecessary computation, caching
- **Memory** — Hot path allocations, leaks/unbounded growth, unnecessary copies
- **I/O** — Batching, N+1 patterns, blocking vs async appropriateness, connection pooling
- **Concurrency** — Beneficial vs unnecessary concurrency, lock contention, model appropriateness
- **Hot Paths** — Identification and optimization appropriateness, cold path over-optimization
- **Resource Lifecycle** — Acquisition/release, exhaustion risks, backpressure
- **Premature Optimization** — Unnecessary complexity for performance, legibility compromises

**Process:** Identify expected workload, find hot paths, analyze algorithmic complexity, assess resource usage, evaluate concurrency model, flag premature optimization.

**Output:** `04-performance.md` — Summary, Workload Assessment, Hot Path Analysis, Findings

### Phase 3: External Review Agents

#### API Design Agent

**Lens:** Is the exposed interface right for its consumers?

**Evaluation Criteria:**
- **Consistency** — Naming conventions, similar operation handling, error formats
- **Ergonomics** — Easy to use correctly, hard to use incorrectly, simple common operations
- **Discoverability** — Logical grouping, helpful errors, self-documenting
- **Error Semantics** — Useful errors, actionable guidance, stable error types, transient vs permanent
- **Abstraction Level** — Right level, not too low/high, escape hatches
- **Versioning and Compatibility** — Strategy, backward compatibility, breaking change communication, deprecation
- **Documentation Alignment** — Accuracy, undocumented features, inaccurate docs, example correctness
- **Principle of Least Surprise** — Expected behavior, gotchas, sensible defaults, surprising side effects

**Output:** `05-api-design.md` — Summary, Interface Inventory, Consistency Assessment, Findings

#### Operability Agent

**Lens:** Can this be run, observed, debugged, and scaled in production?

**Evaluation Criteria:**
- **Configuration** — Management, startup validation, sensible defaults, environment support, runtime changes
- **Observability: Logging** — Usefulness, log levels, context, sensitive data exclusion, structure
- **Observability: Metrics** — Four golden signals, business metrics, health-from-metrics
- **Observability: Tracing** — Distributed tracing, request tracing, trace ID propagation
- **Health and Readiness** — Endpoints, liveness vs readiness, dependency checks, orchestration usefulness
- **Failure Modes** — Identification, dependency failure behavior, graceful degradation, diagnosability
- **Graceful Lifecycle** — Clean startup, graceful shutdown, SIGTERM handling, warm-up
- **Deployment** — Configurations, rollback, blue-green/canary, risk
- **Runbook Readiness** — On-call operability, code-free operational tasks, emergency procedures

**Output:** `06-operability.md` — Summary, Observability Assessment, Failure Mode Analysis, Findings

#### Security Agent

**Lens:** Is this safe against misuse, abuse, and attack?

**Evaluation Criteria:**
- **Trust Boundaries** — Identification, clarity, enforcement consistency, defense in depth
- **Input Validation** — Untrusted input entry points, validation coverage, allowlist vs blocklist, injection potential
- **Authentication** — Identity establishment, credential handling, required coverage, bypass risks
- **Authorization** — Permission checking, consistency across entry points, privilege escalation, IDOR, proximity to data
- **Secrets Management** — Storage method, rotatability, exposure in logs/errors
- **Data Sensitivity** — Classification, handling, encryption at rest/in transit, leakage
- **Dependency Security** — Vulnerability auditing, update handling, trusted sources, transitive dependencies
- **Cryptography** — Appropriate use, modern algorithms, key management, TLS configuration
- **Blast Radius** — Compromise impact, segmentation/isolation, containment, audit logging

**Output:** `07-security.md` — Summary, Trust Boundary Map, Threat Model Observations, Findings

### Phase 4: Cross-Cutting Agent

Identifies findings that span multiple lenses and traces symptoms to root causes.

**Evaluation Tasks:**
1. **Cross-Lens Findings** — Findings noted by multiple agents or with multi-lens implications. Which lenses affected? What's the relationship? Should severity be reconsidered?
2. **Root Cause Analysis** — Patterns where multiple findings trace to a single root cause. Architectural decisions, missing abstractions, process issues.
3. **Tension Identification** — Places where lenses are in tension (performance vs legibility, security vs usability, evolvability vs operations). Tension isn't bad — the question is whether tradeoffs were conscious and appropriate.
4. **Systemic Patterns** — Patterns no single agent would see. Consistent issues across the codebase, missing capabilities, architectural drift.

**Output:** `08-cross-cutting.md` — Summary, Cross-Lens Findings, Root Causes, Tensions, Systemic Patterns, Severity Adjustments

### Phase 5: Synthesis Agent

Produces the final deliverables.

**Deliverable 1: `09-report.md`**
- Executive Summary (1-2 paragraphs, overall assessment, key themes, top concerns)
- Summary Table (findings count by lens and severity)
- Findings by Lens (summary + all findings + positive patterns per lens)
- Cross-Cutting Concerns (root causes, severity adjustments, systemic patterns)
- Appendix: System Overview

**Deliverable 2: `10-recommendations.md`**
- Overview (how to read, prioritization rationale)
- Immediate Actions (must address before further development)
- Short-Term Actions (address in next cycle)
- Structural Improvements (larger efforts to schedule)
- Architectural Recommendations (systemic improvements beyond individual findings)
- Summary Roadmap (sequenced view of recommended work)

**Recommendation Format:**
Each recommendation includes: Addresses (finding IDs), Severity of addressed findings, Effort estimate (Hours/Days/Weeks), What to do, Why it matters, Suggested approach, Dependencies.

**Requirements:**
- Every Critical and Major finding must have a corresponding recommendation
- Recommendations must be actionable
- Group related findings into single recommendations where appropriate
- Be realistic about effort
- Consider dependencies
- Acknowledge what needs more investigation

## Handling Cross-Cutting Findings

### During Review (Phases 2-3)

Agents flag potential cross-cutting implications:

```markdown
## LEG-03: Inconsistent error handling patterns

**Cross-cutting note**: May have implications for Correctness (error behavior) and Operability (debugging). Flagging for Cross-Cutting Agent.
```

### During Cross-Cutting Analysis (Phase 4)

The Cross-Cutting Agent creates explicit links:

```markdown
## Cross-Lens Finding: Error Handling

**Related findings**: LEG-03, COR-07, OPS-02
**Primary root cause**: LEG-03 (inconsistent patterns)
**Impact cascade**: Inconsistent patterns (Legibility) → Incorrect error propagation (Correctness) → Missing error visibility (Operability)
**Adjusted severity**: LEG-03 should be considered Critical given downstream impact
```

### During Synthesis (Phase 5)

The Synthesis Agent creates single recommendations addressing related findings:

```markdown
## REC-04: Standardize Error Handling

**Addresses**: LEG-03, COR-07, OPS-02
**Severity of addressed findings**: Critical (adjusted), Major, Major
**Effort**: Days
```
