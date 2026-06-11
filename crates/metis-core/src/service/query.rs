//! Structured work item queries.
//!
//! Filters by project, type, phase, assignee, tag, and repo, with limit/offset
//! paging. Free-text search (`q`) is deliberately absent here — it is the one
//! sanctioned per-backend divergence (FTS5 vs tsvector) and lands with the
//! search/query task. All filters below are single-arm diesel.

use diesel::prelude::*;
use diesel_dualdb::types::Uuid;
use diesel_dualdb::DualConnection;

use super::{Result, ServiceError};
use crate::models::{ItemSummary, WorkItemRow};
use crate::schema::{projects, repos, work_item_repos, work_item_tags, work_items};

/// Filters for [`list`]. Unset fields don't constrain.
#[derive(Debug, Clone, Default)]
pub struct ItemFilter {
    /// Restrict to a project slug.
    pub project: Option<String>,
    /// Restrict to a work item type.
    pub item_type: Option<String>,
    /// Restrict to a phase.
    pub phase: Option<String>,
    /// Restrict to an assignee.
    pub assignee: Option<uuid::Uuid>,
    /// Restrict to items carrying this tag.
    pub tag: Option<String>,
    /// Restrict to items referencing this repo slug.
    pub repo: Option<String>,
    /// Include archived items (default false).
    pub include_archived: bool,
    /// Max rows (default 100).
    pub limit: Option<i64>,
    /// Offset for paging.
    pub offset: Option<i64>,
}

/// List items matching `filter`, newest first.
pub fn list(conn: &mut DualConnection, filter: &ItemFilter) -> Result<Vec<ItemSummary>> {
    // Resolve scoping ids up front (clean NotFound rather than empty results).
    let project_id = match &filter.project {
        Some(slug) => Some(
            projects::table
                .filter(projects::slug.eq(slug))
                .select(projects::id)
                .first::<Uuid>(conn)
                .optional()?
                .ok_or_else(|| ServiceError::NotFound(format!("project {slug:?}")))?,
        ),
        None => None,
    };

    let tag_item_ids = match &filter.tag {
        Some(tag) => Some(
            work_item_tags::table
                .filter(work_item_tags::tag.eq(tag))
                .select(work_item_tags::item_id)
                .load::<Uuid>(conn)?,
        ),
        None => None,
    };

    let repo_item_ids = match &filter.repo {
        Some(slug) => Some(
            work_item_repos::table
                .inner_join(repos::table.on(repos::id.eq(work_item_repos::repo_id)))
                .filter(repos::slug.eq(slug))
                .select(work_item_repos::item_id)
                .load::<Uuid>(conn)?,
        ),
        None => None,
    };

    let mut q = work_items::table.into_boxed();

    if let Some(pid) = project_id {
        q = q.filter(work_items::project_id.eq(pid));
    }
    if let Some(t) = &filter.item_type {
        q = q.filter(work_items::item_type.eq(t));
    }
    if let Some(p) = &filter.phase {
        q = q.filter(work_items::phase.eq(p));
    }
    if let Some(a) = filter.assignee {
        q = q.filter(work_items::assignee.eq(Uuid(a)));
    }
    if !filter.include_archived {
        q = q.filter(work_items::archived.eq(false));
    }
    if let Some(ids) = tag_item_ids {
        q = q.filter(work_items::id.eq_any(ids));
    }
    if let Some(ids) = repo_item_ids {
        q = q.filter(work_items::id.eq_any(ids));
    }

    let rows: Vec<WorkItemRow> = q
        .order(work_items::created_at.desc())
        .limit(filter.limit.unwrap_or(100))
        .offset(filter.offset.unwrap_or(0))
        .select(WorkItemRow::as_select())
        .load(conn)?;

    Ok(rows.iter().map(ItemSummary::from).collect())
}
