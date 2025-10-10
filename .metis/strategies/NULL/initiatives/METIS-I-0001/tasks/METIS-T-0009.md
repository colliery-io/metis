---
id: add-project-management-with-file
level: task
title: "Add project management with file picker integration"
short_code: "METIS-T-0009"
created_at: 2025-10-08T11:28:54.395351+00:00
updated_at: 2025-10-10T00:50:24.373998+00:00
parent: METIS-I-0001
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: 
initiative_id: METIS-I-0001
---

# Add project management with file picker integration

## Parent Initiative

[[METIS-I-0001]]

**Objective:**
Implement project management functionality that allows users to add, switch between, and manage multiple Metis projects through a file picker interface integrated with the application sidebar.

**Acceptance Criteria:**
- [ ] File picker dialog allows users to select .metis directories
- [ ] Selected projects appear in application sidebar with project names  
- [ ] Application validates selected directories contain valid .metis structure
- [ ] Users can switch between projects via sidebar selection
- [ ] Projects list persists between application sessions
- [ ] Invalid project selections show helpful error messages
- [ ] Users can remove projects from the sidebar
- [ ] Application state properly switches when changing projects

**Implementation Plan:**
1. Enhance ProjectBrowser component with "Add Project" functionality
2. Create project sidebar with list of available projects
3. Add project switching logic to ProjectContext
4. Implement project persistence using localStorage or Tauri storage
5. Add project removal functionality
6. Enhance validation feedback for invalid project selections

**Definition of Done:**
- Users can manage multiple projects from a single GUI instance
- Project switching is seamless and maintains proper state
- File picker integration follows native OS patterns
- All validation scenarios are handled gracefully

## Status Updates

*To be added during implementation*