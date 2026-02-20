---
id: strengthen-mcp-instruction
level: task
title: "Strengthen MCP instruction language on document editing requirements"
short_code: "METIS-T-0059"
created_at: 2026-01-28T14:46:15.840308+00:00
updated_at: 2026-01-28T15:00:18.854811+00:00
parent: METIS-I-0019
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0019
---

# Strengthen MCP instruction language on document editing requirements

## Parent Initiative

[[METIS-I-0019]]

## Objective

Make the MCP server instructions more authoritative about the requirement that agents MUST edit documents with meaningful content after creating them - not just create empty templates.

## Problem Statement

Agents are creating Metis documents but frequently skipping the step of populating them with actual information. The current MCP instruction language is not strong enough to prevent this behavior. The frequency of empty/template-only documents is high enough to be annoying and reduces the value of the tracking system.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] MCP instruction language uses stronger, more directive phrasing (MUST, REQUIRED, etc.)
- [ ] Clear statement that creating a document without populating content is incomplete
- [ ] Add explicit workflow guidance: create -> read -> edit with content
- [ ] Consider adding a warning/reminder about this in tool descriptions

## Implementation Notes

### Files to Update
- MCP server instruction text (likely in plugin configuration or server setup)
- Possibly the skill documentation that references Metis usage

### Suggested Language Changes
- Change "should edit" to "MUST edit"
- Add explicit statement: "Document creation is not complete until content sections are populated"
- Emphasize read-before-edit pattern applies to newly created documents too

## Status Updates

### 2026-01-28: Completed

**Files modified:**

1. `crates/metis-docs-mcp/instructions.md`
   - Added CRITICAL note to `create_document` tool: must follow up with read + edit to populate content
   - Added first Key Principle: "ALWAYS populate document content" with explicit workflow
   - Added "Creating documents without content" as #1 mistake to avoid in Common Mistakes section

2. `plugins/metis/hooks/session-start-hook.sh`
   - Added new "CRITICAL: Creating Documents" section with 3-step workflow
   - Added warning: "A document with template placeholders is INCOMPLETE"

3. `plugins/metis/hooks/pre-compact-hook.sh`
   - Added "(MUST populate content after!)" to create_document tool listing
   - Added "CRITICAL: Creating Documents" reminder in context

**Language changes:**
- "should" → "MUST"
- Added "CRITICAL" labels
- Added explicit workflow: create → read → edit
- Added warning about placeholder text being "useless"/"incomplete"