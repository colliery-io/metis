//! MCP tool definitions, backed by the `metis-core` service layer.
//!
//! Each tool is a `#[mcp_tool]` struct (its fields are the arguments, with a
//! JSON schema derived for the client). The work happens in `run`, which holds
//! a [`McpState`] (pool + object store + actor) and calls the service layer.
//! Tools are the 2.x verbs renamed `document`→`item` so agent skills port with
//! renames only.
//!
// The `#[mcp_tool]` macro and `run` dispatch methods generate/define public
// items that don't warrant doc comments; the tool `description` strings are the
// real documentation surfaced to clients.
#![allow(missing_docs)]

use std::sync::Arc;

use metis_core::actor::Actor;
use metis_core::models::ExitCriterion;
use metis_core::objects::ObjectStore;
// (ExitCriterion is mapped from the MCP-side `ExitCriterionArg` below so
// metis-core need not depend on schemars.)
use metis_core::service::item::{self, ItemEdit, NewItem};
use metis_core::service::query::{self, ItemFilter};
use metis_core::service::{project, repo, ServiceError};
use metis_core::{DualConnection, Pool};
use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};
use rust_mcp_sdk::tool_box;
use serde::{Deserialize, Serialize};

/// Shared state for MCP tool execution: the connection pool, the object store,
/// and the actor this stdio session acts as.
#[derive(Clone)]
pub struct McpState {
    /// Connection pool.
    pub pool: Pool,
    /// Object store backend for bodies.
    pub store: Arc<dyn ObjectStore + Send + Sync>,
    /// The principal recorded on every mutation.
    pub actor: Actor,
}

impl McpState {
    /// Check out a connection and run a (sync) service closure on the blocking
    /// pool, returning a JSON tool result or a mapped tool error.
    async fn run_blocking<T, F>(&self, f: F) -> Result<CallToolResult, CallToolError>
    where
        T: Serialize + Send + 'static,
        F: FnOnce(&mut DualConnection, &dyn ObjectStore, &Actor) -> Result<T, ServiceError>
            + Send
            + 'static,
    {
        let pool = self.pool.clone();
        let store = self.store.clone();
        let actor = self.actor.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(|e| ServiceError::Validation(format!("pool: {e}")))?;
            f(&mut conn, store.as_ref(), &actor)
        })
        .await
        .map_err(|e| tool_err(format!("join error: {e}")))?;

        match result {
            Ok(value) => Ok(ok_json(&value)),
            Err(e) => Err(service_err(e)),
        }
    }
}

/// Serialize a value as pretty JSON into a text tool result.
fn ok_json<T: Serialize>(value: &T) -> CallToolResult {
    let text = serde_json::to_string_pretty(value)
        .unwrap_or_else(|e| format!("{{\"serialization_error\":\"{e}\"}}"));
    CallToolResult {
        content: vec![TextContent::new(text, None, None).into()],
        is_error: None,
        meta: None,
        structured_content: None,
    }
}

/// A generic tool error carrying a message.
fn tool_err(msg: String) -> CallToolError {
    CallToolError::new(std::io::Error::other(msg))
}

/// Map a service error to a tool error, preserving its message (which already
/// distinguishes not-found / validation / phase-gate / conflict).
fn service_err(e: ServiceError) -> CallToolError {
    tool_err(e.to_string())
}

// ----- projects --------------------------------------------------------------

/// Create a project (defaults to the Flight Levels configuration).
#[mcp_tool(
    name = "create_project",
    description = "Create a Metis project. The slug is used as the short-code prefix (uppercased). Without a config the project gets the default Flight Levels types (vision, initiative, task, adr, specification, plus bug/feature/chore).",
    destructive_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateProjectTool {
    /// Project slug (URL-safe; becomes the short-code prefix).
    pub slug: String,
    /// Display name.
    pub name: String,
}

impl CreateProjectTool {
    pub async fn run(&self, state: &McpState) -> Result<CallToolResult, CallToolError> {
        let (slug, name) = (self.slug.clone(), self.name.clone());
        state
            .run_blocking(move |conn, _store, _actor| {
                project::create(conn, &slug, &name, None).map(ProjectOut::from)
            })
            .await
    }
}

/// Read a project's configuration.
#[mcp_tool(
    name = "read_project",
    description = "Read a Metis project by slug, including its work-item type and workflow configuration.",
    idempotent_hint = true,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadProjectTool {
    /// Project slug.
    pub slug: String,
}

impl ReadProjectTool {
    pub async fn run(&self, state: &McpState) -> Result<CallToolResult, CallToolError> {
        let slug = self.slug.clone();
        state
            .run_blocking(move |conn, _store, _actor| {
                project::get(conn, &slug).map(ProjectOut::from)
            })
            .await
    }
}

// ----- items -----------------------------------------------------------------

/// List work items with optional filters.
#[mcp_tool(
    name = "list_items",
    description = "List/search work items. With filters (project slug, type, phase, assignee uuid, tag, repo slug) results are newest-first; with `q` (free-text search) they come back by relevance. Excludes archived unless include_archived is true.",
    idempotent_hint = true,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListItemsTool {
    /// Restrict to a project slug.
    pub project: Option<String>,
    /// Restrict to a work item type (e.g. "task").
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    /// Restrict to a phase.
    pub phase: Option<String>,
    /// Restrict to an assignee user id.
    pub assignee: Option<String>,
    /// Restrict to a tag.
    pub tag: Option<String>,
    /// Restrict to a repo slug.
    pub repo: Option<String>,
    /// Free-text search query; when set, results come back by relevance.
    pub q: Option<String>,
    /// Include archived items.
    #[serde(default)]
    pub include_archived: bool,
    /// Max rows (default 100).
    pub limit: Option<i64>,
    /// Offset for paging.
    pub offset: Option<i64>,
}

impl ListItemsTool {
    pub async fn run(&self, state: &McpState) -> Result<CallToolResult, CallToolError> {
        let assignee = match &self.assignee {
            Some(s) => Some(
                s.parse::<uuid::Uuid>()
                    .map_err(|_| tool_err("assignee must be a uuid".into()))?,
            ),
            None => None,
        };
        let filter = ItemFilter {
            project: self.project.clone(),
            item_type: self.item_type.clone(),
            phase: self.phase.clone(),
            assignee,
            tag: self.tag.clone(),
            repo: self.repo.clone(),
            q: self.q.clone(),
            include_archived: self.include_archived,
            limit: self.limit,
            offset: self.offset,
        };
        state
            .run_blocking(move |conn, _store, _actor| query::list(conn, &filter))
            .await
    }
}

/// Read one work item.
#[mcp_tool(
    name = "read_item",
    description = "Read a work item by short code (e.g. METIS-T-0001), including its markdown body, tags, repos, links, and exit criteria.",
    idempotent_hint = true,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadItemTool {
    /// Short code, e.g. METIS-T-0001.
    pub short_code: String,
}

impl ReadItemTool {
    pub async fn run(&self, state: &McpState) -> Result<CallToolResult, CallToolError> {
        let short_code = self.short_code.clone();
        state
            .run_blocking(move |conn, store, _actor| item::get(conn, store, &short_code))
            .await
    }
}

/// Create a work item.
#[mcp_tool(
    name = "create_item",
    description = "Create a work item in a project. Type must exist in the project config (e.g. task, initiative, bug). Parent is a short code; parent/root rules come from the type config. Returns the created item with its allocated short code.",
    destructive_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateItemTool {
    /// Project slug.
    pub project: String,
    /// Work item type name.
    #[serde(rename = "type")]
    pub item_type: String,
    /// Title.
    pub title: String,
    /// Markdown body (optional).
    #[serde(default)]
    pub body: String,
    /// Parent item short code (if the type requires/allows one).
    pub parent: Option<String>,
    /// Assignee user id.
    pub assignee: Option<String>,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Referenced repo slugs.
    #[serde(default)]
    pub repos: Vec<String>,
    /// Exit-criteria texts (created unmet).
    #[serde(default)]
    pub exit_criteria: Vec<String>,
}

impl CreateItemTool {
    pub async fn run(&self, state: &McpState) -> Result<CallToolResult, CallToolError> {
        let assignee = match &self.assignee {
            Some(s) => Some(
                s.parse::<uuid::Uuid>()
                    .map_err(|_| tool_err("assignee must be a uuid".into()))?,
            ),
            None => None,
        };
        let project = self.project.clone();
        let new = NewItem {
            item_type: self.item_type.clone(),
            title: self.title.clone(),
            body: self.body.clone(),
            parent: self.parent.clone(),
            assignee,
            tags: self.tags.clone(),
            repos: self.repos.clone(),
            exit_criteria: self.exit_criteria.clone(),
        };
        state
            .run_blocking(move |conn, store, actor| {
                item::create(conn, store, actor, &project, &new)
            })
            .await
    }
}

/// Edit a work item's mutable fields.
#[mcp_tool(
    name = "edit_item",
    description = "Edit a work item by short code. Any provided field is updated; a new body writes a new content object and records history. Replaces tags / exit_criteria wholesale when provided.",
    destructive_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EditItemTool {
    /// Short code.
    pub short_code: String,
    /// New title.
    pub title: Option<String>,
    /// New markdown body.
    pub body: Option<String>,
    /// Replacement tag set.
    pub tags: Option<Vec<String>>,
    /// Replacement exit criteria (ordinal/text/met).
    pub exit_criteria: Option<Vec<ExitCriterionArg>>,
}

/// MCP-side exit criterion argument (kept here so metis-core needn't depend on
/// schemars). Maps to [`metis_core::models::ExitCriterion`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExitCriterionArg {
    /// Display order.
    pub ordinal: i32,
    /// Criterion text.
    pub text: String,
    /// Whether met.
    #[serde(default)]
    pub met: bool,
}

impl From<ExitCriterionArg> for ExitCriterion {
    fn from(a: ExitCriterionArg) -> Self {
        ExitCriterion {
            ordinal: a.ordinal,
            text: a.text,
            met: a.met,
        }
    }
}

impl EditItemTool {
    pub async fn run(&self, state: &McpState) -> Result<CallToolResult, CallToolError> {
        let short_code = self.short_code.clone();
        let exit_criteria = self
            .exit_criteria
            .clone()
            .map(|v| v.into_iter().map(ExitCriterion::from).collect());
        let edit = ItemEdit {
            title: self.title.clone(),
            body: self.body.clone(),
            assignee: None,
            tags: self.tags.clone(),
            exit_criteria,
            expected_content_key: None,
        };
        state
            .run_blocking(move |conn, store, actor| {
                item::edit(conn, store, actor, &short_code, &edit)
            })
            .await
    }
}

/// Transition a work item to a new phase.
#[mcp_tool(
    name = "transition_phase",
    description = "Transition a work item to a new phase. Omit phase to advance to the next. The type's workflow is enforced (forward-adjacent + exit-criteria gate) unless force is true.",
    destructive_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransitionPhaseTool {
    /// Short code.
    pub short_code: String,
    /// Target phase (omit to advance to the next phase).
    pub phase: Option<String>,
    /// Force past the gate (non-adjacent / unmet criteria).
    #[serde(default)]
    pub force: bool,
}

impl TransitionPhaseTool {
    pub async fn run(&self, state: &McpState) -> Result<CallToolResult, CallToolError> {
        let short_code = self.short_code.clone();
        let phase = self.phase.clone();
        let force = self.force;
        state
            .run_blocking(move |conn, store, actor| {
                item::transition(conn, store, actor, &short_code, phase.as_deref(), force)
            })
            .await
    }
}

/// Link two work items.
#[mcp_tool(
    name = "link_item",
    description = "Add a link from one work item to another. Kind is one of blocks, relates, supersedes. Both items must be in the same project.",
    destructive_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LinkItemTool {
    /// Source item short code.
    pub from: String,
    /// Link kind: blocks | relates | supersedes.
    pub kind: String,
    /// Target item short code.
    pub to: String,
}

impl LinkItemTool {
    pub async fn run(&self, state: &McpState) -> Result<CallToolResult, CallToolError> {
        let (from, kind, to) = (self.from.clone(), self.kind.clone(), self.to.clone());
        state
            .run_blocking(move |conn, _store, actor| {
                item::add_link(conn, actor, &from, &kind, &to).map(|()| Ack { ok: true })
            })
            .await
    }
}

/// Archive a work item.
#[mcp_tool(
    name = "archive_item",
    description = "Archive a work item by short code.",
    destructive_hint = true,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ArchiveItemTool {
    /// Short code.
    pub short_code: String,
}

impl ArchiveItemTool {
    pub async fn run(&self, state: &McpState) -> Result<CallToolResult, CallToolError> {
        let short_code = self.short_code.clone();
        state
            .run_blocking(move |conn, store, actor| item::archive(conn, store, actor, &short_code))
            .await
    }
}

// ----- repos + briefing ------------------------------------------------------

/// Resolve a git remote URL to its project/repo context.
#[mcp_tool(
    name = "resolve_repo",
    description = "Resolve a git remote URL (any spelling: ssh/https/scp) to the Metis project and repo it is registered under, or report that it is unregistered.",
    idempotent_hint = true,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResolveRepoTool {
    /// A git remote URL (e.g. from `git remote get-url origin`).
    pub remote: String,
}

impl ResolveRepoTool {
    pub async fn run(&self, state: &McpState) -> Result<CallToolResult, CallToolError> {
        let remote = self.remote.clone();
        state
            .run_blocking(move |conn, _store, _actor| {
                // Map "unmatched" to a structured payload rather than an error,
                // so the caller can branch without parsing an error string.
                repo::resolve(conn, &remote).map(|ctx| Resolved {
                    matched: ctx.is_some(),
                    context: ctx,
                })
            })
            .await
    }
}

/// Repo-scoped session briefing.
#[mcp_tool(
    name = "briefing",
    description = "Return the work in flight for a repo (by slug): items touching it that are active or ready to pick up. Use after resolve_repo to brief a session.",
    idempotent_hint = true,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BriefingTool {
    /// Repo slug (from resolve_repo).
    pub repo: String,
}

impl BriefingTool {
    pub async fn run(&self, state: &McpState) -> Result<CallToolResult, CallToolError> {
        let repo_slug = self.repo.clone();
        state
            .run_blocking(move |conn, _store, _actor| repo::briefing(conn, &repo_slug))
            .await
    }
}

// ----- result helpers --------------------------------------------------------

/// Minimal acknowledgement payload for void operations.
#[derive(Serialize)]
struct Ack {
    ok: bool,
}

/// Resolve-repo result: a flag plus the context when matched.
#[derive(Serialize)]
struct Resolved {
    matched: bool,
    context: Option<metis_core::service::repo::RepoContext>,
}

/// Serializable projection of a project row (config + identity).
#[derive(Serialize)]
struct ProjectOut {
    slug: String,
    name: String,
    config: metis_core::workflow::config::ProjectConfig,
}

impl From<metis_core::models::ProjectRow> for ProjectOut {
    fn from(r: metis_core::models::ProjectRow) -> Self {
        Self {
            slug: r.slug,
            name: r.name,
            config: r.config.0,
        }
    }
}

// The combined tool enum (schema/registration) — names must match the dispatch.
tool_box!(
    MetisItemTools,
    [
        CreateProjectTool,
        ReadProjectTool,
        ListItemsTool,
        ReadItemTool,
        CreateItemTool,
        EditItemTool,
        TransitionPhaseTool,
        LinkItemTool,
        ArchiveItemTool,
        ResolveRepoTool,
        BriefingTool
    ]
);
