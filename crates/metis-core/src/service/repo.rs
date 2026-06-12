//! Repo registry, git-remote resolution, and the session briefing.
//!
//! The registry maps a project's repositories to canonical match keys (see
//! [`crate::repo::normalize_git_url`]) so a client's `git remote` can be
//! resolved to a project/repo context. The briefing returns the work an agent
//! should see when it opens a session in a given repo.

use diesel::prelude::*;
use diesel_dualdb::types::{Json, Timestamp, Uuid};
use diesel_dualdb::DualConnection;
use serde::{Deserialize, Serialize};

use super::project;
use super::{Result, ServiceError};
use crate::models::{ItemSummary, RepoRow, WorkItemRow};
use crate::repo::normalize_git_url;
use crate::schema::{repos, work_item_repos, work_items};

/// Where a resolved remote points.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoContext {
    /// Owning project slug.
    pub project: String,
    /// Repo slug within the project.
    pub repo: String,
}

/// Repo-scoped session briefing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Briefing {
    /// Owning project slug.
    pub project: String,
    /// Repo slug the briefing is scoped to.
    pub repo: String,
    /// Items in flight (phase `active`).
    pub active: Vec<ItemSummary>,
    /// Items ready to pick up (phase `todo` or `ready`).
    pub ready: Vec<ItemSummary>,
}

/// Register a repository against a project. Normalized match keys are computed
/// from `git_urls`; the raw URLs are stored too.
pub fn register(
    conn: &mut DualConnection,
    project_slug: &str,
    repo_slug: &str,
    git_urls: Vec<String>,
) -> Result<RepoRow> {
    let project = project::get(conn, project_slug)?;
    let normalized: Vec<String> = git_urls.iter().map(|u| normalize_git_url(u)).collect();
    let id = uuid::Uuid::new_v4();
    diesel::insert_into(repos::table)
        .values((
            repos::id.eq(Uuid(id)),
            repos::project_id.eq(Uuid(project.id.0)),
            repos::slug.eq(repo_slug),
            repos::git_urls.eq(Json(git_urls)),
            repos::normalized_urls.eq(Json(normalized)),
            repos::created_at.eq(Timestamp(chrono::Utc::now())),
        ))
        .execute(conn)?;
    repos::table
        .filter(repos::id.eq(Uuid(id)))
        .select(RepoRow::as_select())
        .first(conn)
        .map_err(Into::into)
}

/// List a project's registered repos.
pub fn list(conn: &mut DualConnection, project_slug: &str) -> Result<Vec<RepoRow>> {
    let project = project::get(conn, project_slug)?;
    repos::table
        .filter(repos::project_id.eq(Uuid(project.id.0)))
        .select(RepoRow::as_select())
        .load(conn)
        .map_err(Into::into)
}

/// Resolve a remote URL to its project/repo context, or `None` if unregistered.
///
/// Matches on the normalized form, so any registered URL spelling of the repo
/// matches any spelling of the client's remote. If more than one repo matches
/// (e.g. the same repo registered under two projects), the first by creation is
/// returned.
pub fn resolve(conn: &mut DualConnection, remote: &str) -> Result<Option<RepoContext>> {
    let key = normalize_git_url(remote);

    // Scan repos and match in Rust: normalized_urls is a JSON array, and a
    // portable "JSON contains" predicate isn't worth a per-backend divergence
    // for a registry of this size.
    let rows: Vec<(Uuid, String, Json<Vec<String>>)> = repos::table
        .order(repos::created_at.asc())
        .select((repos::project_id, repos::slug, repos::normalized_urls))
        .load::<(Uuid, String, Json<Vec<String>>)>(conn)?;

    for (project_id, repo_slug, Json(normalized)) in rows {
        if normalized.iter().any(|n| n == &key) {
            let project = project_slug_by_id(conn, project_id.0)?;
            return Ok(Some(RepoContext {
                project,
                repo: repo_slug,
            }));
        }
    }
    Ok(None)
}

/// Build the repo-scoped briefing: non-archived items touching the repo,
/// bucketed by phase (active / ready). The phase names match the Flight Levels
/// defaults; other configs may use different phase vocabularies.
pub fn briefing(conn: &mut DualConnection, repo_slug: &str) -> Result<Briefing> {
    let (repo_id, project_id) = repos::table
        .filter(repos::slug.eq(repo_slug))
        .select((repos::id, repos::project_id))
        .first::<(Uuid, Uuid)>(conn)
        .optional()?
        .ok_or_else(|| ServiceError::NotFound(format!("repo {repo_slug:?}")))?;

    let rows: Vec<WorkItemRow> = work_items::table
        .inner_join(work_item_repos::table.on(work_item_repos::item_id.eq(work_items::id)))
        .filter(work_item_repos::repo_id.eq(repo_id))
        .filter(work_items::archived.eq(false))
        .order(work_items::updated_at.desc())
        .select(WorkItemRow::as_select())
        .load(conn)?;

    let mut active = Vec::new();
    let mut ready = Vec::new();
    for row in &rows {
        match row.phase.as_str() {
            "active" => active.push(ItemSummary::from(row)),
            "todo" | "ready" => ready.push(ItemSummary::from(row)),
            _ => {}
        }
    }

    Ok(Briefing {
        project: project_slug_by_id(conn, project_id.0)?,
        repo: repo_slug.to_string(),
        active,
        ready,
    })
}

fn project_slug_by_id(conn: &mut DualConnection, project_id: uuid::Uuid) -> Result<String> {
    use crate::schema::projects;
    projects::table
        .filter(projects::id.eq(Uuid(project_id)))
        .select(projects::slug)
        .first::<String>(conn)
        .map_err(Into::into)
}
