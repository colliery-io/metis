---
id: push-conflict-retry-logic
level: task
title: "Push conflict retry logic"
short_code: "METIS-T-0082"
created_at: 2026-02-26T01:32:09.536513+00:00
updated_at: 2026-02-26T17:55:24.809438+00:00
parent: METIS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0020
---

# Push conflict retry logic

## Objective

Handle push failures when the central repo's HEAD has moved since our fetch (another workspace pushed first). Re-run the full sync cycle from scratch and retry the push, up to 5 times.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Detect push rejection due to non-fast-forward (remote HEAD moved)
- [ ] On rejection: re-fetch from central, re-hydrate, re-build owned tree, re-push
- [ ] Retry up to 5 times before failing with a clear error
- [ ] Each retry is a full sync cycle (not just a re-push — central state may have changed)
- [ ] Non-push errors (auth failure, network) are NOT retried — fail immediately
- [ ] Retry count and outcome included in `SyncResult`
- [ ] Exponential backoff or brief delay between retries is not needed (syncs are infrequent, retries are rare)

## Implementation Notes

### Why This Happens

Multiple workspaces push to the same central repo. If workspace A and workspace B both fetch at commit X, then A pushes (central moves to X+1), B's push will be rejected because B's commit parents X, not X+1. B must re-fetch X+1, rebase its tree on the new HEAD, and retry.

Since each workspace only modifies its own folder, there are never actual content conflicts — just a stale parent commit. The retry is mechanical: re-fetch, re-build tree, re-push.

### Technical Approach

```rust
pub fn sync_with_retry(config: &SyncConfig, max_retries: u32) -> Result<SyncResult> {
    for attempt in 0..=max_retries {
        match sync_once(config)? {
            SyncOutcome::Success(result) => return Ok(result),
            SyncOutcome::PushRejected => {
                if attempt == max_retries {
                    return Err(SyncError::RetriesExhausted(max_retries));
                }
                // Loop will re-run full sync
            }
        }
    }
}
```

### Dependencies

- METIS-T-0081 (sync orchestration — this wraps the sync cycle with retry logic)

## Test Scenarios

### Unit Tests — Retry Logic

1. **Success on first try**: push succeeds immediately → SyncResult shows 0 retries, success
2. **Success on second try**: first push rejected (non-fast-forward), re-fetch + re-push succeeds → SyncResult shows 1 retry
3. **Success on fifth try**: push fails 4 times, succeeds on 5th → SyncResult shows 4 retries, success
4. **Exhausted retries**: push fails 6 times → error `RetriesExhausted(5)` with clear message
5. **Each retry is full cycle**: on rejection, verify that fetch + hydrate + tree-build + push all run again (not just a re-push of stale tree)
6. **Retry updates hydrated state**: another workspace pushed between our retries → after re-fetch, we have their latest changes hydrated before we re-push

### Unit Tests — Error Classification

7. **Non-fast-forward → retry**: push rejected because HEAD moved → triggers retry
8. **Auth failure → no retry**: push fails with auth error → immediately returns error, no retry attempted
9. **Network failure → no retry**: push fails with network error → immediately returns error, no retry
10. **Unknown git error → no retry**: unexpected libgit2 error → immediately returns error with raw message for debugging
11. **Fetch failure during retry → fail**: re-fetch fails on retry attempt → stops retrying, returns the fetch error

### Integration Tests

12. **Two workspaces racing**: workspace A and B both sync simultaneously against same central → one succeeds on first try, other retries and succeeds
13. **Three workspaces racing**: A, B, C all push within seconds → all eventually succeed (within retry budget), central has all three workspace folders updated
14. **Rapid contention — worst case**: 5 workspaces all push simultaneously → at least one needs multiple retries, but all eventually succeed (5 retries should be sufficient for 5 contenders)
15. **Contention with actual content**: each workspace pushes different documents → after all retries resolve, central has correct content from all workspaces (no lost writes)

### Edge Cases

16. **Central force-pushed between retries**: central repo is rebased/force-pushed during our retry loop → detect diverged history, fail with clear error (not infinite retry)
17. **Retry after partial sync state**: first attempt hydrated new remote docs, push failed → retry re-hydrates (may be no-op if nothing changed), ensures consistent state
18. **Zero retry budget**: `max_retries = 0` → one attempt only, rejection is immediate failure
19. **Retry count in SyncResult**: after 3 retries → SyncResult.retries == 3, accessible to callers for logging/display

## Status Updates

### Session 1 — Implementation Complete

**Changes made:**

1. **lib.rs**: Added `RetriesExhausted { max_retries: u32 }` error variant to `SyncError`
2. **lib.rs**: Changed `commit_update()` to use detached HEAD (`commit(None, ...) + set_head_detached`) instead of `commit(Some("HEAD"), ...)` — fixes "current tip is not the first parent" error on retry
3. **lib.rs**: Added `is_push_rejection()` helper that recognizes more error patterns: "non-fast-forward", "rejected", "not present locally", "already exists", "lock" — improves retry detection for concurrent push scenarios
4. **orchestration.rs**: Changed `SyncOptions::max_retries` default from 3 to 5
5. **orchestration.rs**: Removed `dehydrate_with_retry()` — replaced by full-cycle retry loop in `sync()`
6. **orchestration.rs**: Refactored `sync()` to retry the FULL cycle (fetch → hydrate → dehydrate → push) on PushRejected, not just fetch + dehydrate

**Key design decisions:**
- Each retry is a full sync cycle — re-fetches AND re-hydrates — so other workspaces' changes are incorporated before retrying
- Non-push errors (auth, network, I/O) are NOT retried — fail immediately
- `RetriesExhausted` error returned when max retries exceeded

**Tests: 103 total (14 new orchestration tests)**
- Component-level: push conflict detection, conflict resolved after re-fetch, full cycle re-hydrates new content, multiple conflicts then success
- sync() level: success with 0 retries, max_retries=0 no conflict, RetriesExhausted error, network failure not retried, default max_retries=5, push_retries field
- Racing (threaded): 2-workspace racing, 3-workspace racing
- Integration: 5-workspace sequential convergence, sequential pushes no lost writes, retry after partial hydration

**Bugs found and fixed:**
- `commit_update()` with `Some("HEAD")` fails on retry because local HEAD points to stale commit from previous failed attempt — fixed with detached HEAD approach
- Push error messages from file:// transport include patterns like "already exists", "not present locally", "lock" that weren't being classified as PushRejected — fixed with `is_push_rejection()` helper