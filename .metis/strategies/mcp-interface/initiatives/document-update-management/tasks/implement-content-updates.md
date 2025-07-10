---
id: task-implement-content-updates
title: Implement Document Content Updates
level: task
status: completed
created_at: 2025-07-03T18:05:00Z
updated_at: 2025-07-03T18:45:00Z
parent: initiative-document-update-management
blocked_by: 
tags:
  - "#task"
  # - "#phase/todo"
  # - "#phase/doing"
  - "#phase/completed"
exit_criteria_met: true
assignee: 
estimated_hours: 6
pr_links: []
archived: false
---

# Implement Document Content Updates

## Parent Initiative
[[Document Update Management Initiative]]

## Objective
Implement section-based content updates that allow surgical modification of document sections without corrupting the overall document structure or frontmatter.

## Acceptance Criteria
- [x] `update_document_content()` function implemented in new `updates.rs` module
- [x] Function finds sections by markdown heading navigation
- [x] Function replaces content until next `##` heading or end of document
- [x] Function preserves frontmatter and overall document structure
- [x] Function validates document after updates to ensure integrity
- [x] Function handles missing sections with clear error messages
- [x] Function supports both relative and absolute file paths
- [x] Function also supports "append" operations where text is just appended at the end of the block
- [x] Comprehensive error handling for file operations and parsing
- [x] Unit tests covering various section update scenarios
- [x] Integration tests with actual markdown files

## Implementation Notes

### Core Function Signature
```rust
pub async fn update_document_content(
    document_path: &Path,
    section_heading: &str,
    new_content: &str,
) -> Result<()>
```

### Implementation Approach
1. **Read and Parse**: Load document and separate frontmatter from content
2. **Section Detection**: Find target section by scanning for `## {section_heading}`
3. **Content Replacement**: Replace from section start to next `##` or end of document
4. **Document Reconstruction**: Combine frontmatter + updated content
5. **Validation**: Ensure updated document passes validation
6. **File Write**: Atomically write updated content back to file

### Section Finding Logic
- Search for exact match of `## {section_heading}` 
- Support case-sensitive matching for consistency
- Handle sections at different nesting levels (##, ###, ####)
- Return clear errors for ambiguous or missing sections

### Content Replacement Strategy
- Replace everything from section heading line to next same-level heading
- Preserve whitespace and formatting around sections
- Support empty content (section deletion)
- Maintain consistent line endings

### Error Scenarios
- Document file not found or not readable
- Invalid markdown structure or corrupted frontmatter
- Section heading not found in document
- Multiple sections with same heading (ambiguous)
- File write permissions or disk space issues
- Document fails validation after update

## Testing Requirements

### Unit Tests
- Update existing sections with new content
- Handle missing sections gracefully
- Preserve frontmatter during content updates
- Update sections at different heading levels
- Replace sections at beginning, middle, and end of document

### Integration Tests
- Full file read/write operations with real markdown
- Document validation integration after updates
- Concurrent update scenarios (if applicable)
- Large document performance testing

### Edge Cases
- Documents with only frontmatter (no content sections)
- Sections with special characters in headings
- Very large content sections
- Documents with mixed heading levels
- Empty documents or single-section documents

## Status Updates
*To be added during implementation*

## Exit Criteria
- [ ] All acceptance criteria have been met
- [ ] Implementation has been tested thoroughly
- [ ] Function integrates with existing validation pipeline
- [ ] Work is ready for MCP server integration