---
id: pre-sync-freshness-check-for
level: task
title: "Pre-sync freshness check for project git repo"
short_code: "METIS-T-0088"
created_at: 2026-02-26T17:13:35.160968+00:00
updated_at: 2026-02-26T17:13:35.160968+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Pre-sync freshness check for project git repo

## Origin

Deferred from METIS-T-0081 (Sync orchestration engine). Was an acceptance criterion on that task but better fits as a separate concern at the CLI/GUI layer rather than the core orchestration engine.

## Objective

Before dehydrating (pushing) owned workspace documents to central, verify that the project's git repo has no unpulled remote commits that touch `.metis/<owned-prefix>/`. This prevents pushing stale local state that would overwrite someone else's changes made via the project repo (as opposed to the central sync repo).

## Acceptance Criteria

- [ ] If inside a git repo, check for unpulled remote commits touching `.metis/<owned-prefix>/`
- [ ] If stale (remote has commits not yet pulled), abort sync with clear warning message
- [ ] If remote has unpulled commits but they don't touch `.metis/`, sync proceeds normally
- [ ] `--force` flag overrides the freshness check and syncs regardless
- [ ] If not inside a git repo, skip the check entirely (sync proceeds)
- [ ] If git repo has no remote configured, skip the check (sync proceeds)
- [ ] Clear error message explains what's stale and how to fix it (e.g., "run `git pull` first")

## Implementation Notes

### Technical Approach

This check runs at the CLI/GUI layer before calling `orchestration::sync()`. It uses the project's own git repo (not the central sync repo) to compare local HEAD with the remote tracking branch.

```rust
// Pseudocode
fn check_freshness(project_dir: &Path, owned_prefix: &str) -> Result<(), FreshnessError> {
    let repo = git2::Repository::discover(project_dir)?;
    let head = repo.head()?.target()?;
    let remote_head = repo.find_reference("refs/remotes/origin/main")?.target()?;
    
    if head == remote_head { return Ok(()); }
    
    // Check if any commits between head..remote_head touch .metis/<prefix>/
    let diff = repo.diff(head, remote_head)?;
    for change in diff {
        if change.path.starts_with(&format!(".metis/{}/", owned_prefix)) {
            return Err(FreshnessError::StaleWorkspace { ... });
        }
    }
    Ok(())
}
```

### Dependencies

- METIS-T-0084 (metis sync CLI command) — integrate the check there
- METIS-T-0086 (GUI sync) — integrate the check in the GUI flow

## Status Updates

*To be added during implementation*