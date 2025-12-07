# Greenfield Project Pattern

Starting a new project from scratch with Metis.

## When to Use

- New product or system
- Major rewrite (treat as new rather than migration)
- Proof of concept that may become production
- New team forming around a new problem

## The Pattern

### 1. Initialize the Workspace

```
initialize_project(project_path="/path/to/project", prefix="PROJ")
```

Choose a prefix that's:
- Short (2-6 characters)
- Memorable
- Unique if you work across multiple projects

### 2. Define the Vision

The vision answers: "Why does this project exist? What are we trying to achieve?"

Write it as a north star document:
- **Objectives**: What outcomes do we want?
- **Values**: Why does this matter? Why do it well?
- **Success criteria**: How will we know we've succeeded?

Don't overcomplicate it. A clear paragraph beats a vague page.

**Transition the vision through phases:**
- draft -> review (get stakeholder input)
- review -> published (when stable)

Only create strategies/initiatives against a published vision.

### 3. Choose Your Preset

For most greenfield projects:

| Team Size | Coordination Needs | Recommended Preset |
|-----------|-------------------|-------------------|
| Solo | None | Direct |
| 2-5 | Informal | Streamlined |
| 5+ | Formal | Full (or Streamlined) |

When in doubt, start with Streamlined. You can always add Strategy later.

### 4. Create Initial Initiatives

Identify the first capability increments you need. Common greenfield initiatives:

- **Foundation/Setup**: Dev environment, CI/CD, basic architecture
- **Core Feature**: The main thing this project does
- **Integration**: Connecting to other systems
- **Release/Launch**: Getting to production

Don't create all initiatives upfront. Create enough to start work, then pull more as backlogs empty.

### 5. Decompose and Execute

For each initiative:
1. Discovery: What do we need to learn?
2. Design: What's our approach?
3. Decompose: What are the tasks?
4. Execute: Do the work

Pull tasks as capacity exists. Don't overload.

## Greenfield-Specific Considerations

### Start Small
Resist the urge to plan everything. Greenfield projects have maximum uncertainty. Learn as you go.

### Foundation First
Some foundational work (dev environment, basic architecture) is necessary before feature work. But don't gold-plate the foundation - build what you need to start.

### Expect Pivots
Early initiatives may invalidate later plans. That's fine. The hierarchy makes it easy to see what needs to change.

### ADRs from Day One
Greenfield projects make many architectural decisions. Capture them in ADRs while context is fresh.

## Example Flow

```
Day 1:
- initialize_project(prefix="ACME")
- create_document(type="vision", title="ACME Platform")
- Edit vision with objectives and values
- transition_phase(ACME-V-0001) # draft -> review

Day 2:
- Review vision with stakeholders
- transition_phase(ACME-V-0001) # review -> published
- create_document(type="initiative", title="Development Foundation", parent="ACME-V-0001")
- create_document(type="initiative", title="Core Feature: User Management", parent="ACME-V-0001")

Week 1:
- Discovery and design on Foundation initiative
- transition_phase(ACME-I-0001) through to decompose
- Create tasks for Foundation
- Start executing Foundation tasks

Week 2+:
- Complete Foundation tasks
- Begin Core Feature discovery
- Pull new work as capacity allows
```

## Common Mistakes

- **Too much planning upfront**: You don't know what you don't know. Start building.
- **No vision document**: "We'll figure it out" leads to drift. Capture the north star.
- **Skipping foundation**: "We'll fix it later" creates debt that compounds.
- **Too many initiatives at once**: Focus. Finish things.
