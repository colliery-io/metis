---
id: task-implement-exit-criteria-updates
level: task
status: completed
created_at: 2025-07-03T18:10:00Z
updated_at: 2025-07-03T19:15:00Z
parent: initiative-document-update-management
blocked_by: 
tags:
  - "#task"
  # - "#phase/todo"
  # - "#phase/doing"
  - "#phase/completed"
exit_criteria_met: true
assignee: 
estimated_hours: 4
pr_links: []
---

# Implement Exit Criteria Updates

## Parent Initiative
[[Document Update Management Initiative]]

## Objective
Implement checkbox toggling functionality that allows precise updates to exit criteria completion status by finding and modifying specific criteria lines.

## Acceptance Criteria
- [x] `update_exit_criterion()` function implemented in `updates.rs` module
- [x] Function finds criteria by text matching in `## Exit Criteria` section
- [x] Function toggles checkbox state: `- [ ]` ↔ `- [x]`
- [x] Function maintains criterion text and ordering exactly
- [x] Function handles partial text matching for flexibility
- [x] Function updates `exit_criteria_met` frontmatter field when all criteria complete
- [x] Function provides clear errors for ambiguous or missing criteria
- [x] Function preserves all other document content unchanged
- [x] Comprehensive error handling for parsing and text matching
- [x] Unit tests covering all checkbox state transitions

## Implementation Notes

### Core Function Signature
```rust
pub async fn update_exit_criterion(
    document_path: &Path,
    criterion_text: &str,
    completed: bool,
) -> Result<()>
```

### Implementation Approach
1. **Load Document**: Read and parse markdown with frontmatter
2. **Find Exit Criteria Section**: Locate `## Exit Criteria` section
3. **Parse Checkboxes**: Extract all checkbox lines with text
4. **Text Matching**: Find target criterion by substring or exact match
5. **State Update**: Toggle checkbox between `[ ]` and `[x]`
6. **Frontmatter Update**: Update `exit_criteria_met` based on all criteria status
7. **Document Write**: Save updated document atomically

### Text Matching Strategy
- Support both exact and partial text matching
- Case-insensitive matching for flexibility
- Return error if multiple criteria match (ambiguous)
- Return error if no criteria match
- Handle criteria with markdown formatting in text

### Checkbox Detection
- Parse lines starting with `- [ ]` or `- [x]`
- Support various checkbox formats: `- [ ]`, `- [X]`, `- [x]`
- Preserve indentation and spacing around checkboxes
- Handle nested checkbox lists if present

### Frontmatter Updates
- Count total criteria and completed criteria
- Set `exit_criteria_met: true` only when all criteria are complete
- Set `exit_criteria_met: false` if any criteria incomplete
- Update `updated_at` timestamp in frontmatter

### Error Scenarios
- Document or exit criteria section not found
- Criterion text matches multiple items (ambiguous)
- Criterion text matches no items
- Invalid checkbox format or corrupted document
- Frontmatter parsing or update failures

## Testing Requirements

### Unit Tests
- Toggle checkbox from unchecked to checked
- Toggle checkbox from checked to unchecked
- Find criteria by exact text match
- Find criteria by partial text match
- Handle multiple matching criteria (should error)
- Handle no matching criteria (should error)
- Update frontmatter when all criteria complete
- Update frontmatter when criteria incomplete

### Integration Tests
- Full document read/write operations
- Integration with existing exit criteria validation
- Concurrent updates to same document
- Large documents with many criteria

### Edge Cases
- Documents without exit criteria section
- Empty exit criteria section
- Malformed checkbox lines
- Criteria with special characters or markdown
- Very long criterion text

## Status Updates
*To be added during implementation*

## Exit Criteria
- [ ] All acceptance criteria have been met
- [ ] Implementation has been tested thoroughly
- [ ] Function correctly updates frontmatter fields
- [ ] Work is ready for MCP server integration