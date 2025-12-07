# When to Create an ADR

Architecture Decision Records capture significant decisions. This guide helps you recognize when a decision warrants an ADR.

## The Core Question

**Will someone in the future wonder "why did we do it this way?"**

If yes, write an ADR.

## Decision Triggers

### Technology Choices

Create an ADR when choosing:
- Programming language for a new service
- Framework or library for significant functionality
- Database or storage technology
- Infrastructure platform
- Third-party services or APIs

**Not an ADR**: Choosing which npm package for date formatting (unless it's a team-wide standard).

### Architectural Patterns

Create an ADR when adopting:
- Microservices vs monolith
- Event-driven vs request-response
- Specific design patterns (CQRS, event sourcing, etc.)
- API styles (REST, GraphQL, gRPC)
- Data modeling approaches

**Not an ADR**: Using a factory pattern in one class.

### Standards and Conventions

Create an ADR when establishing:
- Coding standards
- Testing strategies
- Deployment processes
- Security practices
- Error handling approaches

**Not an ADR**: One-off coding style fix.

### Trade-offs

Create an ADR when making conscious trade-offs:
- Performance vs simplicity
- Flexibility vs consistency
- Build vs buy
- Speed vs quality (in specific contexts)

**Not an ADR**: Normal prioritization decisions.

## The ADR Test

Ask these questions:

1. **Is this reversible easily?**
   - Hard to reverse → ADR
   - Easy to reverse → Maybe not

2. **Does this affect multiple initiatives?**
   - Yes → ADR
   - No → Probably not

3. **Did we evaluate alternatives?**
   - Yes, meaningfully → ADR documents why we chose this one
   - No, obvious choice → Probably not

4. **Will this decision outlive the current team?**
   - Yes → ADR preserves context
   - No → Probably not

5. **Is there disagreement worth documenting?**
   - Yes → ADR records the reasoning
   - No → Probably not

If 2+ answers are "yes", write an ADR.

## ADR Anatomy

A good ADR contains:

### Context
What situation prompted this decision? What constraints exist?

### Decision
What did we decide? Be specific.

### Alternatives Considered
What else did we evaluate? Why didn't we choose those?

### Consequences
What are the implications? Both positive and negative.

### Status
- **draft** - Proposal, open for discussion
- **discussion** - Being debated
- **decided** - Final, this is our approach
- **superseded** - Replaced by a newer decision

## Examples

### Should Be ADRs

- "Use PostgreSQL for the primary database"
- "Adopt TypeScript for all frontend code"
- "Use Kubernetes for container orchestration"
- "Implement feature flags using LaunchDarkly"
- "Store files in S3 vs local filesystem"

### Should NOT Be ADRs

- "Use React hooks instead of class components in this file"
- "Name this variable `userId` instead of `user_id`"
- "Add an index to this table"
- "Which test framework to use for a single project" (unless it's a team standard)

## Creating the ADR

```
create_document(
  type="adr",
  title="Use PostgreSQL for primary database",
  decision_maker="Tech Lead"
)
```

Then fill in:
- Context and background
- The decision
- Alternatives considered
- Expected consequences

## ADR Lifecycle

### draft
Initial proposal. Share for early feedback.

### discussion
Active debate. Gather input, refine alternatives.

Transition when:
- Key stakeholders have weighed in
- Alternatives are fully documented
- Team is ready to decide

### decided
Final decision. This is how we do it.

Transition when:
- Decision maker has approved
- Team is aligned (or at least informed)
- Implementation can begin

### superseded
Replaced by a newer ADR.

Happens when:
- Circumstances changed significantly
- Original decision was wrong
- Better option became available

Link to the superseding ADR in the document.

## Common Mistakes

- **ADR for everything**: Not every decision needs documentation. Use judgment.
- **ADR as proposal only**: ADRs should reach `decided` status. If it stays in draft forever, it's not an ADR.
- **No alternatives**: If you didn't consider alternatives, did you really make a decision?
- **Too vague**: "Use good architecture" isn't a decision.
- **Never updating status**: A draft ADR from 6 months ago is just noise.
- **Ignoring consequences**: Every decision has trade-offs. Document them.
