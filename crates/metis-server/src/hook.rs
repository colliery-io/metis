//! Session-start hook: brief an agent on the work in its repo (design D5/D8).
//!
//! On session start the hook detects the checkout's git remote, resolves it to
//! a Metis project/repo, fetches the repo-scoped briefing, and returns the
//! SessionStart hook payload to inject as context. It must never break a
//! session: any missing precondition (no git, no config, unmatched remote,
//! server down) yields `None` or a one-line notice, never an error to the user.

use std::path::Path;
use std::process::Command;

use metis_core::db;
use metis_core::service::repo::{self, Briefing, RepoContext};
use serde_json::json;

use crate::user_config::UserConfig;

/// Produce the SessionStart hook JSON for a checkout at `cwd`, or `None` to stay
/// silent (not a Metis-managed environment / nothing to say).
///
/// Errors are surfaced to the caller (the CLI maps them to a silent exit) so a
/// transient server problem never blocks the session.
pub fn session_start(cwd: &Path, cfg: &UserConfig) -> anyhow::Result<Option<String>> {
    let Some(database_url) = cfg.database_url.as_deref() else {
        return Ok(None); // no client config → not a Metis environment
    };
    let Some(remote) = git_remote(cwd) else {
        return Ok(None); // not a git repo, or no remotes
    };

    let pool = db::connect(database_url)?;
    let mut conn = pool.get()?;

    let Some(ctx) = repo::resolve(&mut conn, &remote)? else {
        return Ok(Some(wrap(notice(&remote))));
    };
    let briefing = repo::briefing(&mut conn, &ctx.repo)?;
    Ok(Some(wrap(render_context(&ctx, &briefing))))
}

/// The repo's remote URL: `origin` if present, else the first remote.
fn git_remote(cwd: &Path) -> Option<String> {
    if let Some(url) = git_remote_url(cwd, "origin") {
        return Some(url);
    }
    let out = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .arg("remote")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let first = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())?
        .to_string();
    git_remote_url(cwd, &first)
}

fn git_remote_url(cwd: &Path, name: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["remote", "get-url", name])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!url.is_empty()).then_some(url)
}

/// Wrap context text in the SessionStart hook envelope (serde handles escaping).
fn wrap(context: String) -> String {
    json!({
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": context,
        }
    })
    .to_string()
}

/// Notice shown when the repo's remote isn't registered with the server.
fn notice(remote: &str) -> String {
    format!(
        "This repository (remote `{remote}`) is not registered with Metis. \
         An admin can register it with `metis admin` / the REST API \
         (`POST /api/v1/projects/<slug>/repos`) so work here shows up in your \
         session briefing."
    )
}

/// Render the briefing as session context, in the item/short-code vocabulary.
fn render_context(ctx: &RepoContext, b: &Briefing) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "This repository is part of the **Metis** project `{}` (repo `{}`). Metis is the \
         system of record for work here — track plans, decisions, and progress as work items, \
         and update active items as you go (they survive context compaction; the conversation \
         does not).\n\n",
        ctx.project, ctx.repo
    ));

    s.push_str("## Work in flight for this repo\n");
    if b.active.is_empty() && b.ready.is_empty() {
        s.push_str("No active or ready work items reference this repo.\n");
    } else {
        if !b.active.is_empty() {
            s.push_str("\n**Active:**\n");
            for it in &b.active {
                s.push_str(&format!("- {} — {}\n", it.short_code, it.title));
            }
        }
        if !b.ready.is_empty() {
            s.push_str("\n**Ready to pick up:**\n");
            for it in &b.ready {
                s.push_str(&format!("- {} — {}\n", it.short_code, it.title));
            }
        }
    }

    s.push_str(
        "\n## MCP tools\n\
         Items are addressed by short code (e.g. `METIS-T-0001`). Use:\n\
         - `list_items` / `read_item` — find and read work\n\
         - `create_item` (project slug + type) / `edit_item` — create and update\n\
         - `transition_phase` — advance an item (the type's workflow is enforced)\n\
         - `link_item`, `archive_item`\n\
         - `resolve_repo`, `briefing` — repo context\n\
         - `create_project`, `read_project`\n",
    );

    s
}
