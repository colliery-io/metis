---
id: typography-overhaul
level: task
title: "Typography overhaul"
short_code: "METIS-T-0046"
created_at: 2025-12-28T19:39:43.785596+00:00
updated_at: 2025-12-28T19:52:21.828032+00:00
parent: METIS-I-0017
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0017
---

# Typography overhaul

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[METIS-I-0017]]

## Objective **[REQUIRED]**

Replace generic "AI slop" typography (Inter, system-ui, Arial) with distinctive, characterful fonts that elevate the interface.

### Font Stack Changes
- **Display font (headers)**: `Satoshi`, `Manrope`, or similar geometric sans
- **Body font**: `DM Sans`, `Outfit`, or refined sans-serif
- **Monospace (short codes)**: `JetBrains Mono`, `IBM Plex Mono`

### Theme-Specific Typography (optional)
- **Light theme**: Editorial feel with `Newsreader` or serif headers
- **Dark theme**: Modern geometric with `Satoshi` or `General Sans`
- **Hyper theme**: Futuristic with `Clash Display` or tight-tracked geometric

### Implementation
1. Add font imports to `index.html` or via `@font-face` in CSS
2. Update `styles.css` `:root` font-family declarations
3. Create CSS variables: `--font-display`, `--font-body`, `--font-mono`
4. Create type scale with consistent sizing (clamp for fluid typography)

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] No Inter, Arial, Helvetica, or system fonts in use
- [ ] Display font loaded and applied to headers/titles
- [ ] Body font loaded and applied to content text
- [ ] Monospace font applied to short codes (e.g., METIS-T-0046)
- [ ] Font loading doesn't cause visible FOUT/FOIT
- [ ] All three themes render with appropriate typography

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates **[REQUIRED]**

*To be added during implementation*