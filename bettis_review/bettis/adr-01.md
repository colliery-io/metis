# ADR-01: Avro as Serialization Format for Data Contracts

## Context

A data contract is being established between producers and consumers. One piece of the puzzle is the serialization format that the data sent between producers and consumers is defined in. Many different serialization formats exist, including JSON, Avro, Protobuf, Cap'n Proto, FlatBuffers, Fury, and many more.

Regardless of which serialization format is chosen, defined schemas must be:

- **Evolvable** — schemas change over time without breaking consumers
- **Open source** — to prevent vendor lock-in
- **Well supported** in multiple languages

Overall, the serialization format determines what the schema definition language looks like, how schemas are distributed, and what the on-wire message format is.

---

## Decision

**Avro** will be the serialization format for all data contracts on the Bettis platform.

---

## Alternatives Analysis

| | **JSON** | **Protobuf** | **Cap'n Proto / FlatBuffers / Fury** | **Avro** |
|---|---|---|---|---|
| **Pros** | Simple. Human readable. Low barrier to entry. Supported by all programming languages. | Compact binary encoding. Strong, well-defined schema evolution rules. First-class gRPC integration. Battle-tested and mature across all major languages. Rich logical types. | Zero-cost encoding/decoding. Same in-memory data structure regardless of language. Best performance. | Compact binary encoding. Strong, well-defined schema evolution rules. First-class Kafka integration, along with native analytics engine support (Spark, Hive, Iceberg, Parquet, etc.). No code generation required. Rich logical types. |
| **Cons** | Verbose, very inefficient (memory footprint extremely large from duplicated keys). Weak type system (no date/time, no distinction between int/float/double/etc). | Code generation is mandatory: every schema change requires regenerating and distributing client libraries — heavier coupling in polyglot environments. Not natively supported by most analytics engines (Spark, Hive, etc.). | Immature but quickly gaining maturity in the ecosystem. Higher barrier to entry, not as developer friendly. | Weaker adoption in the RPC landscape compared to Protobuf. Slightly less efficient binary encoding than Protobuf. Relies on schema registry for resolution. |
| **Risk Level** | High | Medium | High | **Medium** |
| **Implementation Cost** | Low, but will quickly run into issues with memory footprint losses, likely creating higher cost in the short term. | High. Additional tooling around distributed compiled multi-language definitions needs to be developed and tested for usage with multiple teams. Significantly higher barrier to entry. | High. High initial investment. Additional tooling needed. | **Medium**. Moderate initial investment to stand up a schema registry (Confluent, Apicurio, or AWS Glue Schema Registry), configure CI-based compatibility checks, and establish schema governance workflows. Low marginal cost for Kafka, Spark, and Flink integration. |

---

## Rationale

1. **Schema evolution is non-negotiable for data contracts**, and Avro's reader/writer schema resolution model — backed by schema registry compatibility enforcement — provides the strongest foundation for governing contracts across independently deployed producers and consumers.

2. **Downstream analytics on Spark, Flink, and lakehouse formats** (Iceberg/Delta) natively consume Avro, eliminating format conversion pipelines.

3. **Protobuf is the strongest alternative** and would be preferred if our primary concern were service-to-service gRPC communication instead of significant Kafka usage.

4. **JSON remains appropriate for external-facing APIs** and human-readable interfaces but lacks the enforcement guarantees required for internal data contracts.

---

## Consequences

### Positive

- All data contract schemas will be defined in Avro JSON schema and registered in the schema registry with backward compatibility mode as the default
- CI pipelines will gate on schema compatibility checks before any producer schema change is deployed

### Negative

- We accept the operational dependency on a highly available schema registry and will invest in its monitoring and redundancy
- We accept the slightly higher inefficiency of the binary size of messages when compared with Protobuf

### Neutral

- A backward-compatible-by-default schema registry, with incompatible schemas requiring creation of a new schema

---

## Review Schedule

Not a permanent decision. Review once per quarter.

### Review Triggers

- Avro becomes an operational burden on consuming applications
- Schema registry being a required running component becomes a risk
- Operational support of tooling for Avro becomes prohibitive

### Scheduled Review

- **Next Review Date:** TBD
- **Review Criteria:** See triggers
- **Sunset Date:** No sunset date
