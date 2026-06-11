//! Row structs (Diesel `Queryable`/`Selectable`) and the DTOs the service layer
//! returns. Row structs use diesel-dualdb's portable wrapper types; DTOs expose
//! plain `uuid::Uuid` / `chrono` / `String` values for callers.

use diesel::prelude::*;
use diesel_dualdb::types::{Json, Timestamp, Uuid};
use serde::{Deserialize, Serialize};

use crate::schema::{exit_criteria, projects, repos, users, work_items};
use crate::workflow::config::ProjectConfig;

// ----- row structs ----------------------------------------------------------

/// A row of `work_items`.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = work_items)]
pub struct WorkItemRow {
    /// Stable identity.
    pub id: Uuid,
    /// Owning project.
    pub project_id: Uuid,
    /// Human-facing code, e.g. `METIS-T-0042`.
    pub short_code: String,
    /// Per-(project,type) sequence number.
    pub seq: i32,
    /// Work item type name.
    pub item_type: String,
    /// Current workflow phase.
    pub phase: String,
    /// Title.
    pub title: String,
    /// Object-store key of the body, if any.
    pub content_key: Option<String>,
    /// Parent item id (hierarchy), if any.
    pub parent_id: Option<Uuid>,
    /// Assigned user id, if any.
    pub assignee: Option<Uuid>,
    /// Creator user id.
    pub created_by: Uuid,
    /// Whether archived.
    pub archived: bool,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last update time.
    pub updated_at: Timestamp,
}

/// A row of `projects`.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = projects)]
pub struct ProjectRow {
    /// Identity.
    pub id: Uuid,
    /// URL-safe project slug.
    pub slug: String,
    /// Display name.
    pub name: String,
    /// Type/workflow configuration.
    pub config: Json<ProjectConfig>,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last update time.
    pub updated_at: Timestamp,
}

/// A row of `users`.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct UserRow {
    /// Identity.
    pub id: Uuid,
    /// Unique username.
    pub username: String,
    /// Display name.
    pub display_name: String,
    /// Optional email.
    pub email: Option<String>,
    /// Admin flag.
    pub is_admin: bool,
    /// Active flag.
    pub active: bool,
    /// Creation time.
    pub created_at: Timestamp,
}

/// A row of `repos`.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = repos)]
pub struct RepoRow {
    /// Identity.
    pub id: Uuid,
    /// Owning project.
    pub project_id: Uuid,
    /// Repo slug, unique within the project.
    pub slug: String,
    /// All registered git URL forms.
    pub git_urls: Json<Vec<String>>,
    /// Normalized URL forms used for matching.
    pub normalized_urls: Json<Vec<String>>,
    /// Creation time.
    pub created_at: Timestamp,
}

/// A row of `exit_criteria`.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = exit_criteria)]
pub struct ExitCriterionRow {
    /// Identity.
    pub id: Uuid,
    /// Owning work item.
    pub item_id: Uuid,
    /// Display order.
    pub ordinal: i32,
    /// Criterion text.
    pub text: String,
    /// Whether met.
    pub met: bool,
    /// Who marked it met.
    pub met_by: Option<Uuid>,
    /// When marked met.
    pub met_at: Option<Timestamp>,
}

// ----- DTOs ------------------------------------------------------------------

/// A single exit criterion, plain types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExitCriterion {
    /// Display order.
    pub ordinal: i32,
    /// Criterion text.
    pub text: String,
    /// Whether met.
    pub met: bool,
}

/// A relationship between two items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    /// Relationship kind: `blocks`, `relates`, or `supersedes`.
    pub kind: String,
    /// The short code of the target item.
    pub target: String,
}

/// Compact item view for lists/boards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemSummary {
    /// Short code.
    pub short_code: String,
    /// Type name.
    pub item_type: String,
    /// Current phase.
    pub phase: String,
    /// Title.
    pub title: String,
    /// Assignee user id, if any.
    pub assignee: Option<uuid::Uuid>,
    /// Whether archived.
    pub archived: bool,
}

/// Full item view, including body and relations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemDetail {
    /// Identity.
    pub id: uuid::Uuid,
    /// Short code.
    pub short_code: String,
    /// Owning project id.
    pub project_id: uuid::Uuid,
    /// Type name.
    pub item_type: String,
    /// Current phase.
    pub phase: String,
    /// Title.
    pub title: String,
    /// Markdown body (resolved from the object store).
    pub body: String,
    /// Object-store key of the current body, if any (used as an
    /// optimistic-concurrency token by the REST layer).
    pub content_key: Option<String>,
    /// Parent item short code, if any.
    pub parent: Option<String>,
    /// Assignee user id, if any.
    pub assignee: Option<uuid::Uuid>,
    /// Creator user id.
    pub created_by: uuid::Uuid,
    /// Whether archived.
    pub archived: bool,
    /// Tags.
    pub tags: Vec<String>,
    /// Referenced repo slugs.
    pub repos: Vec<String>,
    /// Outgoing links.
    pub links: Vec<Link>,
    /// Exit criteria checklist.
    pub exit_criteria: Vec<ExitCriterion>,
}

impl From<ExitCriterionRow> for ExitCriterion {
    fn from(r: ExitCriterionRow) -> Self {
        Self {
            ordinal: r.ordinal,
            text: r.text,
            met: r.met,
        }
    }
}

impl From<&WorkItemRow> for ItemSummary {
    fn from(r: &WorkItemRow) -> Self {
        Self {
            short_code: r.short_code.clone(),
            item_type: r.item_type.clone(),
            phase: r.phase.clone(),
            title: r.title.clone(),
            assignee: r.assignee.map(|u| u.0),
            archived: r.archived,
        }
    }
}
