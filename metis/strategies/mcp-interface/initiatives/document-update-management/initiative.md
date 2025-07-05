---
id: initiative-document-update-management
level: initiative
status: completed
created_at: 2025-07-03T15:30:00Z
updated_at: 2025-07-03T19:45:00Z
parent: strategy-mcp-interface
blocked_by: 
tags:
  - "#initiative"
  # - "#phase/discovery"
  # - "#phase/design"
  # - "#phase/ready"
  # - "#phase/decompose"
  # - "#phase/active"
  - "#phase/completed"
exit_criteria_met: true
technical_lead: 
estimated_complexity: m
related_adrs: 
---

# Document Update Management Initiative

## Context

The MCP Interface strategy requires agents to perform surgical updates on existing Metis documents. Currently, the core library only supports document creation, validation, and phase transitions, but lacks the ability to update document content, exit criteria, or relationships.

Agents need three specific update capabilities to be productive:
1. **Update document content** - Add or modify sections as work progresses
2. **Update exit criteria checkboxes** - Mark criteria complete (`- [ ]` → `- [x]`)
3. **Update blocked_by relationships** - Remove blockers as dependencies are resolved

Without these capabilities, agents would be limited to read-only operations and creating new documents, severely limiting their usefulness in iterative workflows.

## Goals & Non-Goals

**Goals:**
- Implement surgical document update functions in the core library
- Support content section updates without corrupting document structure
- Enable exit criteria checkbox toggling with validation
- Support blocked_by relationship management
- Maintain all existing validation and business rules
- Provide clear error handling for update operations

**Non-Goals:**
- Full rich-text editing or complex formatting
- Frontmatter field updates beyond blocked_by
- Template modification or custom field additions
- Multi-document batch operations
- Real-time collaborative editing
- Version control or change tracking (beyond existing sync capabilities)

## Detailed Design

### Core Update Functions

```rust
// Update specific document sections
pub async fn update_document_content(
    document_path: &Path,
    section_heading: &str,
    new_content: &str,
) -> Result<()>;

// Toggle exit criteria checkbox completion
pub async fn update_exit_criterion(
    document_path: &Path,
    criterion_text: &str,
    completed: bool,
) -> Result<()>;

// Update blocked_by relationships
pub async fn update_blocked_by(
    document_path: &Path,
    blocked_by: Vec<String>,
) -> Result<()>;
```

### Content Update Strategy

**Section-based updates** using markdown heading navigation:
- Find `## {section_heading}` in document
- Replace content until next `##` heading or end of document
- Preserve frontmatter and overall document structure
- Validate document after updates

**Exit Criteria Updates**:
- Parse existing criteria in `## Exit Criteria` section
- Find criterion by text matching
- Toggle checkbox state: `- [ ]` ↔ `- [x]`
- Maintain criterion text and ordering

**Blocked By Updates**:
- Update frontmatter `blocked_by` field
- Validate YAML structure remains intact
- Support add/remove operations

### Error Handling

- **Document Not Found**: Clear file system errors
- **Section Not Found**: Specific heading missing errors
- **Criterion Not Found**: Exit criteria text matching errors
- **Invalid Structure**: Document corruption prevention
- **Validation Failures**: Post-update validation errors

### Implementation Approach

1. **Create `src/updates.rs` module** with focused update operations
2. **Extend markdown parsing** to support section detection and replacement
3. **Integrate with existing validation** to ensure updates don't break documents
4. **Add comprehensive tests** for all update scenarios
5. **Update core module exports** to include new functions

## Alternatives Considered

1. **Full Document Replacement**
   - Pros: Simple implementation
   - Cons: Risk of data loss, no surgical precision
   - Decision: Rejected - too risky for agent operations

2. **Rich Text Editor Integration**
   - Pros: More powerful editing capabilities
   - Cons: Complex, beyond MCP needs, formatting complications
   - Decision: Rejected - surgical updates are sufficient

3. **Database-only Updates**
   - Pros: No file parsing complexity
   - Cons: Bypasses file system, breaks sync model
   - Decision: Rejected - must maintain file-first approach

## Implementation Plan

1. **Phase 1: Content Updates** (Week 1)
   - Implement section-based content replacement
   - Add markdown parsing for heading detection
   - Create update_document_content() function

2. **Phase 2: Exit Criteria Updates** (Week 2)
   - Extend exit criteria parsing for updates
   - Implement checkbox state toggling
   - Create update_exit_criterion() function

3. **Phase 3: Relationship Updates** (Week 3)
   - Implement frontmatter blocked_by updates
   - Add YAML manipulation safely
   - Create update_blocked_by() function

4. **Phase 4: Integration & Testing** (Week 4)
   - Comprehensive testing across all update types
   - Integration with existing validation
   - Error handling and edge case coverage

## Testing Strategy

- **Unit Tests**: Each update function with various content scenarios
- **Integration Tests**: Updates combined with validation and phase transitions
- **Error Handling Tests**: Invalid documents, missing sections, malformed criteria
- **Regression Tests**: Ensure updates don't break existing functionality
- **File System Tests**: Verify file integrity after updates

## Exit Criteria

- [ ] update_document_content() function updates sections safely
- [ ] update_exit_criterion() function toggles checkboxes correctly
- [ ] update_blocked_by() function manages relationships properly
- [ ] All update operations preserve document validity
- [ ] Updates integrate with existing validation pipeline
- [ ] Comprehensive test coverage for all update scenarios
- [ ] Clear error messages for all failure modes
- [ ] Functions ready for MCP server integration