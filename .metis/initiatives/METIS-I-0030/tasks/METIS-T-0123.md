---
id: update-metis-init-to-install-git
level: task
title: "Update metis init to install git hooks and add overlay to .gitignore"
short_code: "METIS-T-0123"
created_at: 2026-03-29T23:01:31.328476+00:00
updated_at: 2026-03-30T00:54:47.444644+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Update metis init to install git hooks and add overlay to .gitignore

## Parent Initiative

[[METIS-I-0030]]

## Objective

Install the post-commit hook into `.git/hooks/` and add `.metis/.pending/` to `.metis/.gitignore`. Hook installation should happen both via `metis init` AND automatically on any Metis operation when the hook is missing (lazy install).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `metis init` detects `.git/` directory and installs post-commit hook
- [ ] Hook also auto-installs lazily: any Metis operation (CLI, MCP) checks for the hook on startup and installs if missing
- [ ] If `.git/hooks/post-commit` already exists, appends or chains (don't clobber existing hooks)
- [ ] `.metis/.gitignore` updated to include `.pending/` directory
- [ ] Hook file is executable (`chmod +x`)
- [ ] Idempotent — safe to run repeatedly, checks for marker comment before appending
- [ ] Works when `.git/hooks/` doesn't exist yet (create it)

## Implementation Notes

- Lazy install: add a `ensure_git_hook_installed()` function to the workspace initialization/detection path so it runs on every Metis startup
- The check is cheap: `file_exists(".git/hooks/post-commit")` + grep for marker comment
- Modify `crates/metis-docs-cli/src/commands/init.rs` (lines 79-82 currently write `.gitignore`)
- Also wire into workspace detection in `metis-docs-core` so MCP server picks it up too
- For hook chaining: if existing hook exists, check if it already contains our marker comment; if not, append
- Consider shipping the hook as an embedded string constant or a template file
- Blocked by: METIS-T-0122 (hook must exist to install)

## Status Updates

- `ensure_git_hook_installed()` in `flush.rs`: discovers .git dir, creates hooks dir if needed, installs or appends hook
- Hook content embedded as `HOOK_CONTENT` const with `METIS_POST_COMMIT_HOOK` marker for idempotent detection
- When appending to existing hook: skips shebang line, appends body only
- Sets chmod 755 on unix
- `metis init`: updated .gitignore to include `.pending/`, calls `ensure_git_hook_installed()`
- Lazy install: `workspace::has_metis_vault()` calls `ensure_git_hook_installed()` on workspace discovery (fires on every CLI command)
- Updated actual `.metis/.gitignore` in this repo to include `.pending/`
- Full workspace compiles clean