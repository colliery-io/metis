# Incident Response Pattern

Handling urgent/reactive work within Metis structure.

## When to Use

- Production incidents
- Critical bugs (P0/P1)
- Security vulnerabilities
- Urgent customer escalations
- Regulatory/compliance emergencies

## The Challenge

Incidents are inherently unplanned. They interrupt normal flow. The question is: how do you handle them without abandoning structure entirely?

## The Pattern

### For Immediate Response (Hours)

Don't create documents during the fire. Focus on resolution.

**After the incident:**
```
create_document(
  type="task",
  title="[INCIDENT] Fix payment processing outage",
  backlog_category="bug"
)
```

Mark it completed if already resolved. This creates a record.

### For Extended Incidents (Days)

If the fix requires significant work:

1. **Create a backlog item immediately:**
```
create_document(
  type="task",
  title="[P0] Payment service failing under load",
  backlog_category="bug"
)
```

2. **Transition to doing:**
```
transition_phase(PROJ-T-XXXX)  # backlog -> todo
transition_phase(PROJ-T-XXXX)  # todo -> doing
```

3. **Add context to the document:**
- Impact description
- Timeline of events
- Root cause (when known)
- Resolution steps

4. **Complete when resolved:**
```
transition_phase(PROJ-T-XXXX)  # doing -> completed
```

### For Incidents Requiring Initiative-Level Response

Major incidents may spawn entire initiatives:

```
create_document(
  type="initiative",
  title="Payment System Resilience",
  parent="VISION-ID"
)
```

Use this when:
- Root cause requires architectural changes
- Multiple coordinated tasks are needed
- Prevention measures need design work

The initiative follows normal phases, but may move quickly through discovery/design if the path is clear.

## Priority Levels

Use backlog categories and naming to signal urgency:

| Level | Meaning | Response |
|-------|---------|----------|
| P0 | System down, revenue impact | Drop everything |
| P1 | Major feature broken | Same-day response |
| P2 | Significant bug | Next sprint |
| P3 | Minor issue | Backlog |

Prefix task titles: `[P0]`, `[P1]`, etc.

## Post-Incident

### Create an ADR for Significant Decisions
If the incident response involved architectural decisions:
```
create_document(
  type="adr",
  title="Add circuit breaker to payment service",
  decision_maker="Team Lead"
)
```

### Create Follow-Up Tasks
Incidents often reveal needed improvements:
```
create_document(type="task", title="Add monitoring for payment latency", backlog_category="feature")
create_document(type="task", title="Document payment service runbook", backlog_category="tech-debt")
```

### Link to Postmortem
If you do formal postmortems, reference them in the task/initiative documentation.

## Balancing Incidents with Planned Work

### Dedicated Capacity
Reserve capacity for unplanned work:
- "One person on interrupt duty per rotation"
- "20% buffer for incidents"

### Interrupt Protocols
Define when it's okay to interrupt planned work:
- P0: Always interrupt
- P1: Interrupt if on-call, otherwise queue
- P2+: Queue for normal prioritization

### Track Interrupt Load
If incidents constantly disrupt planned work:
- Reliability initiative needed
- Scope planned work more conservatively
- Address root causes, not just symptoms

## Example: Production Outage

**During incident (don't do Metis work):**
- Focus on resolution
- Communicate in incident channel
- Take notes for later

**After resolution:**
```
# Create record
create_document(
  type="task",
  title="[P0] Database connection pool exhaustion - 2hr outage",
  backlog_category="bug"
)

# It's already fixed, mark complete
transition_phase(PROJ-T-0099) # to todo
transition_phase(PROJ-T-0099) # to doing
transition_phase(PROJ-T-0099) # to completed

# Create follow-ups
create_document(type="task", title="Add connection pool monitoring", backlog_category="feature")
create_document(type="task", title="Implement connection pool auto-scaling", backlog_category="tech-debt")
```

## Common Mistakes

- **Creating documents during active incident**: Focus on resolution first.
- **No record after incident**: Loses learning and history.
- **Every incident becomes an initiative**: Most are simple fixes. Right-size the response.
- **No follow-through on prevention**: Incidents repeat without systemic fixes.
- **Heroics as standard**: If incidents require heroics, something is wrong upstream.
