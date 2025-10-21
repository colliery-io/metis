---
id: sync-needs-to-check-config-table
level: task
title: "Sync Needs to Check Config Table"
short_code: "METIS-T-0009"
created_at: 2025-10-21T12:42:37.385397+00:00
updated_at: 2025-10-21T18:04:56.364346+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Sync Needs to Check Config Table

Because this system is being used in git repositories it is possible (probably even) that git collisions will occur. When this happens, especially if it happens within

the database, it can corrupt the sqlite database and require a re initialization. We handle file syncronization just fine but do not recover the configuration options of the project

well right now.

## Objective **\[REQUIRED\]**

*Metis syncronization methods need to check the configuration table for validity:*

- *in the event of a database getting nuked we should be able to reconstruct required fields (like prefix, counter, project structure)*
- *in the event of a mismatch, we should update the database configurations to match what is seen on disk (i.e. counter, prefix)*

\
TBH - testing for correct project structure may be very difficult. We may have to use a mix of heuristics + just informing users that the database was

corrupted and needs to be reviewed after our guesses. This could introduce work for all interfaces as a result.

## 

## Backlog Item Details **\[CONDITIONAL: Backlog Item\]**

{Delete this section when task is assigned to an initiative}

### Type

- \[ x \] Bug - Production issue that needs fixing
- \[ \] Feature - New functionality or enhancement
- \[ \] Tech Debt - Code improvement or refactoring
- \[ \] Chore - Maintenance or setup work

### Priority

- \[ \] P0 - Critical (blocks users/revenue)
- \[ x \] P1 - High (important for user experience)
- \[ \] P2 - Medium (nice to have)
- \[ \] P3 - Low (when time permits)

### Impact Assessment **\[CONDITIONAL: Bug\]**

- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**:
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **\[CONDITIONAL: Feature\]**

- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **\[CONDITIONAL: Tech Debt\]**

- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **\[REQUIRED\]**

- [ ] `.metis/config.toml` is created on workspace initialization with prefix and flight levels
- [ ] Sync operation reads config.toml and updates database configuration if different
- [ ] Counter recovery scans filesystem and updates database counters to prevent duplicate short codes
- [ ] System can fully recover from complete database loss using config.toml + markdown files
- [ ] When database is corrupt/missing, sync recreates it with correct configuration
- [ ] Counters are always >= highest short code number found in filesystem
- [ ] Migration creates config.toml for existing workspaces from current DB values
- [ ] All existing tests pass with new configuration system
- [ ] New integration tests verify corruption recovery scenarios

## Test Cases **\[CONDITIONAL: Testing Task\]**

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

## Documentation Sections **\[CONDITIONAL: Documentation Task\]**

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

### API Documentation **\[CONDITIONAL: API Documentation\]**

- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **\[CONDITIONAL: Technical Task\]**

### Technical Approach

**Hybrid Configuration Recovery (Option 3)**

The solution uses a two-pronged approach:

1. **External configuration file** (`.metis/config.toml`) stores user preferences (prefix, flight levels)
2. **Automatic counter recovery** scans filesystem to rebuild counters from existing short codes

This makes the database a rebuildable index/cache, while config.toml becomes the lightweight source of truth.

#### Phase 1: External Configuration File Structure

- Add `toml` dependency to `metis-docs-core/Cargo.toml`
- Create `ConfigFile` struct in `src/domain/configuration.rs`:
  - Fields: `project_prefix`, `flight_levels`
  - Methods: `load()`, `save()`, `default()`
- Define `.metis/config.toml` schema:

  ```toml
  [project]
  prefix = "METIS"
  
  [flight_levels]
  strategies_enabled = false
  initiatives_enabled = true
  ```

#### Phase 2: Counter Recovery System

- Add `recover_counters_from_filesystem()` method in `SyncService`:
  - Scan all `.md` files in workspace
  - Extract short codes from frontmatter
  - Parse to find max counter per document type
  - Return `HashMap<DocType, u32>`
- Add `set_counter_if_lower()` method in `ConfigurationRepository`:
  - Only update counter if DB value &lt; filesystem max
  - Log warning when recovery occurs

#### Phase 3: Configuration Synchronization in Sync

- Update `SyncService::sync_directory()` to:
  1. Load `.metis/config.toml`
  2. Sync config to database (config file = source of truth)
  3. Recover counters from filesystem
  4. Perform normal file sync operations
- Add validation/warnings for mismatches

#### Phase 4: Initialization Updates

- Update `WorkspaceInitializationService::initialize_workspace()`:
  - Create `.metis/config.toml` with default values
  - Populate from provided prefix parameter
- Migration for existing workspaces:
  - On first sync, if no config.toml exists, create from current DB values

#### Phase 5: Testing

- Unit tests for `ConfigFile` load/save
- Integration tests for counter recovery
- Full recovery test: delete DB, run sync, verify complete recovery
- Merge conflict simulation test

### Dependencies

- `toml` crate for parsing configuration files
- Existing `ConfigurationRepository` and `SyncService` infrastructure
- Document frontmatter parsing (already implemented)

### Risk Considerations

1. **Migration Risk**: Existing workspaces need smooth migration to config file

   - Mitigation: This is a minimally used system, we’ll rely on a manual update by users and acknowledge this as a breaking change on upgrade to 0.7.0

2. **Counter Accuracy**: Filesystem scan must be thorough to avoid duplicate short codes

   - Mitigation: Take max counter found + 1, scan all files recursively

3. **Config File Conflicts**: Git merges could create conflicts in config.toml

   - Mitigation: TOML is text-based and much easier to merge than binary DB; user can resolve manually

## Status Updates **\[REQUIRED\]**

*To be added during implementation*