# Metis Flight Levels Work Management System

## Overview
Metis implements Flight Levels methodology for managing work at different altitudes - from strategic vision down to individual tasks. Each level operates at a different time horizon and abstraction level, with work flowing downward through phases and feedback flowing upward.

## The Work Management System

### Flight Level 3: Vision (Strategic Direction)
**Purpose**: Define WHY the work exists and WHERE you're heading
**Time Horizon**: 6 months to 2+ years
**Key Question**: "What outcomes do we want to achieve?"

**Lifecycle Process**:
- **Draft**: Capture initial vision, stakeholders, and success criteria
  - Use `create_document` with `document_type: "vision"`
  - Focus on outcomes, not solutions
  - Define what success looks like
- **Review**: Refine vision with stakeholder feedback
  - Use `edit_document` to incorporate feedback
  - Validate alignment with organizational goals
- **Published**: Vision is stable and drives strategic planning
  - Use `transition_phase` to move to published
  - Vision becomes foundation for strategy creation

**When to Create**: At project start or when strategic direction changes
**Tools**: `create_document`, `edit_document`, `transition_phase`

### Flight Level 2: Strategy (How to Achieve Vision)
**Purpose**: Define HOW to achieve the vision through coordinated approaches
**Time Horizon**: 3-12 months
**Key Question**: "What coordinated approaches will deliver the vision?"

**Lifecycle Process**:
- **Shaping**: Explore different approaches and define strategy scope
  - Requires published Vision as parent
  - Set `risk_level` (low/medium/high) to guide resource allocation
  - Identify key assumptions and dependencies
- **Design**: Detail the strategic approach and success criteria
  - Define clear outcomes and measures
  - Identify required initiatives
  - Use `edit_document` to build strategy details
- **Ready**: Strategy is validated and ready for initiative creation
  - All dependencies identified and addressed
  - Resource requirements understood
- **Active**: Initiatives are being executed under this strategy
  - Monitor progress through initiative completion
  - Adjust strategy based on learning
- **Completed**: All initiatives complete and strategy outcomes achieved
  - Use `archive_document` when strategy is fully delivered

**When to Create**: When you have a published vision and need coordinated approaches
**Tools**: `create_document` (with `parent_id` and `risk_level`), `transition_phase`

### Flight Level 1: Initiative (Concrete Projects)
**Purpose**: Deliver specific capabilities or outcomes that advance strategy
**Time Horizon**: 1-6 months
**Key Question**: "What concrete projects will deliver strategic outcomes?"

**Lifecycle Process**:
- **Discovery**: Understand problem space and define solution approach
  - Requires active Strategy as parent
  - Set `complexity` (xs/s/m/l/xl) to guide team allocation
  - Research constraints and opportunities
- **Design**: Define detailed solution and implementation plan
  - Create concrete deliverables and acceptance criteria
  - Identify task breakdown structure
- **Ready**: Solution validated and ready for task decomposition
  - All assumptions tested
  - Dependencies resolved or managed
- **Decompose**: Break initiative into executable tasks
  - Use `create_document` to create individual tasks
  - Each task should be completable in 1-2 weeks
- **Active**: Tasks are being executed
  - Monitor task completion and adjust as needed
  - Handle blockers and dependencies
- **Completed**: All tasks complete and initiative outcomes delivered
  - Validate outcomes against strategy requirements

**When to Create**: When strategy is active and you need specific project delivery
**Tools**: `create_document` (with `parent_id` and `complexity`), `transition_phase`

### Flight Level 0: Task (Individual Work Items)
**Purpose**: Execute specific work that contributes to initiative delivery
**Time Horizon**: 1-14 days
**Key Question**: "What specific work needs to be done?"

**Lifecycle Process**:
- **Todo**: Task is defined and ready for execution
  - Requires initiative in decompose or active phase as parent
  - Clear acceptance criteria and deliverables
  - Assigned to specific team member
- **Doing**: Task is actively being worked on
  - Progress tracked and blockers identified
  - Use `edit_document` to update blocked_by section if dependencies block progress
- **Completed**: Task deliverables are finished and validated
  - Outcomes contribute to initiative progress
  - Use `archive_document` when no longer relevant

**When to Create**: During initiative decompose phase or when new work is identified
**Tools**: `create_document` (with `parent_id`), `edit_document`, `transition_phase`

### Cross-Level: ADR (Architectural Decision Records)
**Purpose**: Capture significant technical/architectural decisions at any level
**Time Horizon**: Permanent record
**Key Question**: "What decisions need to be documented and why?"

**Lifecycle Process**:
- **Draft**: Initial decision proposal with context and options
  - No parent required - can relate to any level
  - Set `decision_maker` for accountability
- **Discussion**: Stakeholder review and debate
  - Gather input and refine decision rationale
- **Decided**: Final decision made and communicated
  - Decision is binding and guides implementation
- **Superseded**: Decision replaced by newer ADR
  - Maintain historical record

**When to Create**: When significant decisions impact multiple initiatives or have long-term consequences
**Tools**: `create_document` (with `decision_maker`), `transition_phase`

## Process Flow Patterns

### Starting New Work
1. **Always begin with Vision**: Use `create_document` with `document_type: "vision"`
2. **Assess vision completion**: Read vision document to check if exit criteria are met before transitioning
3. **Create 2-4 strategies**: Each addressing different aspects of the vision
4. **Activate strategies sequentially**: Based on priority and dependencies

### Managing Active Work
1. **Use `list_documents` regularly**: Monitor work across all levels
2. **Check for blockers**: Use `search_documents` to find blocked items
3. **Read documents to assess completion**: Check exit criteria and progress directly
4. **Update parent documents**: When child work completes, update parent status

### Handling Dependencies and Blockers
1. **Identify blockers early**: Update blocked_by sections in document content
2. **Escalate blocked work**: Move decisions up flight levels when needed
3. **Create ADRs for decisions**: Document significant choices that unblock work
4. **Adjust timelines**: Update parent documents when dependencies cause delays

### Work Completion and Archival
1. **Assess completion**: Read documents to evaluate if exit criteria are met
2. **Update parent status**: Reflect child completion in parent documents
3. **Archive when appropriate**: Use `archive_document` for completed work trees
4. **Capture learnings**: Update processes based on what was learned

## Tool Usage Guidelines

### Query and Discovery Tools
- **`list_documents`**: Regular work monitoring, finding what needs attention
- **`search_documents`**: Finding specific work, identifying patterns or blockers

### Work Creation Tools  
- **`initialize_project`**: Starting new project workspace
- **`create_document`**: Creating work at any level (always specify parent except for Vision/ADR)

### Work Management Tools
- **`transition_phase`**: Moving work forward through lifecycle phases
- **`read_document`**: Read document content and structure before making edits
- **`edit_document`**: Make targeted changes using search-and-replace (always read documents first)

### Maintenance Tools
- **`archive_document`**: Removing completed work trees from active management

Remember: Flight Levels is about managing work at the right altitude. Keep vision strategic, strategies coordinated, initiatives concrete, and tasks actionable. Let feedback flow upward to adjust higher-level work based on ground truth.