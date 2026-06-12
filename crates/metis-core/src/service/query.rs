//! Structured work item queries, plus free-text search via `q`.
//!
//! Structured filters (project, type, phase, assignee, tag, repo) are single-arm
//! diesel. When `q` is set, results are restricted to full-text matches (see
//! [`crate::service::search`], the sanctioned per-backend divergence) and
//! returned in relevance order; the structured filters still apply.

use std::collections::HashMap;

use diesel::prelude::*;
use diesel_dualdb::types::Uuid;
use diesel_dualdb::DualConnection;
use serde::{Deserialize, Serialize};

use super::{search, Result, ServiceError};
use crate::models::{ItemSummary, WorkItemRow};
use crate::schema::{projects, repos, work_item_repos, work_item_tags, work_items};

/// Filters for [`list`]. Unset fields don't constrain. Serializable so saved
/// views can persist a query.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ItemFilter {
    /// Restrict to a project slug.
    pub project: Option<String>,
    /// Restrict to a work item type. Serialized as `type` (matches the
    /// `GET /items?type=` param and what users write in a saved view).
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    /// Restrict to a phase.
    pub phase: Option<String>,
    /// Restrict to an assignee.
    pub assignee: Option<uuid::Uuid>,
    /// Restrict to items carrying this tag.
    pub tag: Option<String>,
    /// Restrict to items referencing this repo slug.
    pub repo: Option<String>,
    /// Free-text query (full-text search); results come back in relevance order.
    pub q: Option<String>,
    /// Include archived items (default false).
    pub include_archived: bool,
    /// Max rows (default 100).
    pub limit: Option<i64>,
    /// Offset for paging.
    pub offset: Option<i64>,
}

/// Hard ceiling on returned rows, regardless of the requested `limit` — a
/// guardrail so no client (REST or MCP) can pull an unbounded result set.
pub const MAX_LIMIT: i64 = 500;

/// Max FTS matches considered before applying structured filters + paging.
const SEARCH_FETCH_CAP: i64 = MAX_LIMIT;

/// Clamp a requested limit to `[1, MAX_LIMIT]` (default 100).
fn clamp_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(100).clamp(1, MAX_LIMIT)
}

/// Clamp a requested offset to `>= 0` (default 0).
fn clamp_offset(offset: Option<i64>) -> i64 {
    offset.unwrap_or(0).max(0)
}

/// List items matching `filter`. Without `q`, newest first; with `q`, by search
/// relevance.
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

    // Free-text: get the ranked short codes first; restrict + reorder below.
    let search_codes = match filter.q.as_deref().map(str::trim) {
        Some(q) if !q.is_empty() => Some(search::search(conn, q, SEARCH_FETCH_CAP)?),
        _ => None,
    };
    if matches!(&search_codes, Some(c) if c.is_empty()) {
        return Ok(vec![]); // searched, nothing matched
    }

    let mut query = work_items::table.into_boxed();
    if let Some(pid) = project_id {
        query = query.filter(work_items::project_id.eq(pid));
    }
    if let Some(t) = &filter.item_type {
        query = query.filter(work_items::item_type.eq(t));
    }
    if let Some(p) = &filter.phase {
        query = query.filter(work_items::phase.eq(p));
    }
    if let Some(a) = filter.assignee {
        query = query.filter(work_items::assignee.eq(Uuid(a)));
    }
    if !filter.include_archived {
        query = query.filter(work_items::archived.eq(false));
    }
    if let Some(ids) = tag_item_ids {
        query = query.filter(work_items::id.eq_any(ids));
    }
    if let Some(ids) = repo_item_ids {
        query = query.filter(work_items::id.eq_any(ids));
    }

    if let Some(codes) = search_codes {
        // Restrict to the FTS hits, then reorder by relevance in Rust (the
        // SQL `IN (...)` loses the rank order) and page.
        query = query.filter(work_items::short_code.eq_any(codes.clone()));
        let rows: Vec<WorkItemRow> = query.select(WorkItemRow::as_select()).load(conn)?;
        let rank: HashMap<&str, usize> = codes
            .iter()
            .enumerate()
            .map(|(i, c)| (c.as_str(), i))
            .collect();
        let mut summaries: Vec<ItemSummary> = rows.iter().map(ItemSummary::from).collect();
        summaries.sort_by_key(|s| {
            rank.get(s.short_code.as_str())
                .copied()
                .unwrap_or(usize::MAX)
        });
        let offset = clamp_offset(filter.offset) as usize;
        let limit = clamp_limit(filter.limit) as usize;
        Ok(summaries.into_iter().skip(offset).take(limit).collect())
    } else {
        let rows: Vec<WorkItemRow> = query
            .order(work_items::created_at.desc())
            .limit(clamp_limit(filter.limit))
            .offset(clamp_offset(filter.offset))
            .select(WorkItemRow::as_select())
            .load(conn)?;
        Ok(rows.iter().map(ItemSummary::from).collect())
    }
}
