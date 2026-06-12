//! The `/api/v1` REST surface for projects and work items.
//!
//! Handlers are thin: deserialize → `spawn_blocking(service call)` → serialize.
//! The error envelope and `ServiceError` mapping live in [`crate::error`]. The
//! authenticated [`Actor`] is injected by the auth middleware and read from
//! request extensions.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use metis_core::actor::Actor;
use metis_core::models::{ExitCriterion, ItemDetail, ItemSummary};
use metis_core::service::item::{self, ItemEdit, NewItem};
use metis_core::service::project;
use metis_core::service::query::{self, ItemFilter};
use metis_core::service::repo;
use metis_core::service::view;
use metis_core::workflow::config::ProjectConfig;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

// ----- projects --------------------------------------------------------------

/// Project response view.
#[derive(Debug, Serialize)]
pub struct ProjectView {
    /// Slug.
    pub slug: String,
    /// Display name.
    pub name: String,
    /// Type/workflow configuration.
    pub config: ProjectConfig,
}

impl From<metis_core::models::ProjectRow> for ProjectView {
    fn from(r: metis_core::models::ProjectRow) -> Self {
        Self {
            slug: r.slug,
            name: r.name,
            config: r.config.0,
        }
    }
}

/// `POST /projects` body.
#[derive(Debug, Deserialize)]
pub struct CreateProject {
    /// Slug.
    pub slug: String,
    /// Display name.
    pub name: String,
    /// Optional config (defaults to Flight Levels).
    pub config: Option<ProjectConfig>,
}

/// `POST /api/v1/projects`
pub async fn create_project(
    State(state): State<AppState>,
    Json(body): Json<CreateProject>,
) -> Result<(StatusCode, Json<ProjectView>), ApiError> {
    let view = state
        .blocking(move |conn| {
            project::create(conn, &body.slug, &body.name, body.config)
                .map(ProjectView::from)
                .map_err(ApiError::from)
        })
        .await?;
    Ok((StatusCode::CREATED, Json(view)))
}

/// `GET /api/v1/projects/:slug`
pub async fn get_project(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ProjectView>, ApiError> {
    let view = state
        .blocking(move |conn| {
            project::get(conn, &slug)
                .map(ProjectView::from)
                .map_err(ApiError::from)
        })
        .await?;
    Ok(Json(view))
}

/// `PATCH /api/v1/projects/:slug` — replace the config (re-validated).
pub async fn update_project_config(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(config): Json<ProjectConfig>,
) -> Result<Json<ProjectView>, ApiError> {
    let view = state
        .blocking(move |conn| {
            project::update_config(conn, &slug, config)
                .map(ProjectView::from)
                .map_err(ApiError::from)
        })
        .await?;
    Ok(Json(view))
}

// ----- items -----------------------------------------------------------------

/// Query params for `GET /items`.
#[derive(Debug, Deserialize)]
pub struct ListParams {
    /// Project slug.
    pub project: Option<String>,
    /// Type name.
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    /// Phase.
    pub phase: Option<String>,
    /// Assignee user id (uuid string).
    pub assignee: Option<String>,
    /// Tag.
    pub tag: Option<String>,
    /// Repo slug.
    pub repo: Option<String>,
    /// Free-text query (full-text search; results ordered by relevance).
    pub q: Option<String>,
    /// Include archived (default false).
    #[serde(default)]
    pub include_archived: bool,
    /// Max rows.
    pub limit: Option<i64>,
    /// Offset.
    pub offset: Option<i64>,
}

/// `GET /api/v1/items`
pub async fn list_items(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<ItemSummary>>, ApiError> {
    let assignee = match params.assignee {
        Some(s) => Some(
            s.parse::<uuid::Uuid>()
                .map_err(|_| ApiError::bad_request("assignee must be a uuid"))?,
        ),
        None => None,
    };
    let filter = ItemFilter {
        project: params.project,
        item_type: params.item_type,
        phase: params.phase,
        assignee,
        tag: params.tag,
        repo: params.repo,
        q: params.q,
        include_archived: params.include_archived,
        limit: params.limit,
        offset: params.offset,
    };
    let items = state
        .blocking(move |conn| query::list(conn, &filter).map_err(ApiError::from))
        .await?;
    Ok(Json(items))
}

/// `POST /items` body.
#[derive(Debug, Deserialize)]
pub struct CreateItem {
    /// Project slug.
    pub project: String,
    /// Type name.
    #[serde(rename = "type")]
    pub item_type: String,
    /// Title.
    pub title: String,
    /// Markdown body.
    #[serde(default)]
    pub body: String,
    /// Parent short code.
    pub parent: Option<String>,
    /// Assignee user id.
    pub assignee: Option<uuid::Uuid>,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Repo slugs.
    #[serde(default)]
    pub repos: Vec<String>,
    /// Exit criteria texts.
    #[serde(default)]
    pub exit_criteria: Vec<String>,
}

/// `POST /api/v1/items`
pub async fn create_item(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(body): Json<CreateItem>,
) -> Result<(StatusCode, Json<ItemDetail>), ApiError> {
    let store = state.store.clone();
    let project = body.project.clone();
    let new = NewItem {
        item_type: body.item_type,
        title: body.title,
        body: body.body,
        parent: body.parent,
        assignee: body.assignee,
        tags: body.tags,
        repos: body.repos,
        exit_criteria: body.exit_criteria,
    };
    let detail = state
        .blocking(move |conn| {
            item::create(conn, store.as_ref(), &actor, &project, &new).map_err(ApiError::from)
        })
        .await?;
    Ok((StatusCode::CREATED, Json(detail)))
}

/// `GET /api/v1/items/:short_code`
pub async fn get_item(
    State(state): State<AppState>,
    Path(short_code): Path<String>,
) -> Result<Json<ItemDetail>, ApiError> {
    let store = state.store.clone();
    let detail = state
        .blocking(move |conn| item::get(conn, store.as_ref(), &short_code).map_err(ApiError::from))
        .await?;
    Ok(Json(detail))
}

/// `PATCH /items/:short_code` body.
#[derive(Debug, Deserialize)]
pub struct PatchItem {
    /// New title.
    pub title: Option<String>,
    /// New body.
    pub body: Option<String>,
    /// New assignee (`null` JSON clears; absent leaves unchanged).
    #[serde(default, deserialize_with = "double_option")]
    pub assignee: Option<Option<uuid::Uuid>>,
    /// Replacement tags.
    pub tags: Option<Vec<String>>,
    /// Replacement exit criteria.
    pub exit_criteria: Option<Vec<ExitCriterion>>,
}

/// `PATCH /api/v1/items/:short_code`. An `If-Match` header carrying the current
/// `content_key` is enforced as an optimistic-concurrency precondition.
pub async fn patch_item(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(short_code): Path<String>,
    headers: HeaderMap,
    Json(body): Json<PatchItem>,
) -> Result<Json<ItemDetail>, ApiError> {
    let store = state.store.clone();
    let expected = headers
        .get(axum::http::header::IF_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|s| Some(s.trim_matches('"').to_string()));
    let edit = ItemEdit {
        title: body.title,
        body: body.body,
        assignee: body.assignee,
        tags: body.tags,
        exit_criteria: body.exit_criteria,
        expected_content_key: expected,
    };
    let detail = state
        .blocking(move |conn| {
            item::edit(conn, store.as_ref(), &actor, &short_code, &edit).map_err(ApiError::from)
        })
        .await?;
    Ok(Json(detail))
}

/// `POST /items/:short_code/transition` body.
#[derive(Debug, Deserialize)]
pub struct TransitionBody {
    /// Target phase (omit to advance to the next).
    pub phase: Option<String>,
    /// Force past the gate.
    #[serde(default)]
    pub force: bool,
}

/// `POST /api/v1/items/:short_code/transition`
pub async fn transition_item(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(short_code): Path<String>,
    Json(body): Json<TransitionBody>,
) -> Result<Json<ItemDetail>, ApiError> {
    let store = state.store.clone();
    let detail = state
        .blocking(move |conn| {
            item::transition(
                conn,
                store.as_ref(),
                &actor,
                &short_code,
                body.phase.as_deref(),
                body.force,
            )
            .map_err(ApiError::from)
        })
        .await?;
    Ok(Json(detail))
}

/// `POST /api/v1/items/:short_code/archive`
pub async fn archive_item(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(short_code): Path<String>,
) -> Result<Json<ItemDetail>, ApiError> {
    let store = state.store.clone();
    let detail = state
        .blocking(move |conn| {
            item::archive(conn, store.as_ref(), &actor, &short_code).map_err(ApiError::from)
        })
        .await?;
    Ok(Json(detail))
}

/// `POST /items/:short_code/links` body.
#[derive(Debug, Deserialize)]
pub struct LinkBody {
    /// `blocks`, `relates`, or `supersedes`.
    pub kind: String,
    /// Target item short code.
    pub target: String,
}

/// `POST /api/v1/items/:short_code/links`
pub async fn add_link(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(short_code): Path<String>,
    Json(body): Json<LinkBody>,
) -> Result<StatusCode, ApiError> {
    state
        .blocking(move |conn| {
            item::add_link(conn, &actor, &short_code, &body.kind, &body.target)
                .map_err(ApiError::from)
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /items/:short_code/links` body (same shape as add).
pub async fn remove_link(
    State(state): State<AppState>,
    Path(short_code): Path<String>,
    Json(body): Json<LinkBody>,
) -> Result<StatusCode, ApiError> {
    state
        .blocking(move |conn| {
            item::remove_link(conn, &short_code, &body.kind, &body.target).map_err(ApiError::from)
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ----- repos + briefing ------------------------------------------------------

/// Repo response view.
#[derive(Debug, Serialize)]
pub struct RepoView {
    /// Repo slug.
    pub slug: String,
    /// Registered git URLs.
    pub git_urls: Vec<String>,
}

impl From<metis_core::models::RepoRow> for RepoView {
    fn from(r: metis_core::models::RepoRow) -> Self {
        Self {
            slug: r.slug,
            git_urls: r.git_urls.0,
        }
    }
}

/// `POST /projects/:slug/repos` body.
#[derive(Debug, Deserialize)]
pub struct RegisterRepo {
    /// Repo slug.
    pub slug: String,
    /// Git URLs in any spelling (ssh/https/scp); normalized keys are derived.
    pub git_urls: Vec<String>,
}

/// `POST /api/v1/projects/:slug/repos`
pub async fn register_repo(
    State(state): State<AppState>,
    Path(project): Path<String>,
    Json(body): Json<RegisterRepo>,
) -> Result<(StatusCode, Json<RepoView>), ApiError> {
    let view = state
        .blocking(move |conn| {
            repo::register(conn, &project, &body.slug, body.git_urls)
                .map(RepoView::from)
                .map_err(ApiError::from)
        })
        .await?;
    Ok((StatusCode::CREATED, Json(view)))
}

/// `GET /api/v1/projects/:slug/repos`
pub async fn list_repos(
    State(state): State<AppState>,
    Path(project): Path<String>,
) -> Result<Json<Vec<RepoView>>, ApiError> {
    let views = state
        .blocking(move |conn| {
            repo::list(conn, &project)
                .map(|rows| rows.into_iter().map(RepoView::from).collect::<Vec<_>>())
                .map_err(ApiError::from)
        })
        .await?;
    Ok(Json(views))
}

/// Query for `GET /repos/resolve`.
#[derive(Debug, Deserialize)]
pub struct ResolveParams {
    /// A git remote URL (any spelling).
    pub remote: String,
}

/// `GET /api/v1/repos/resolve?remote=…` → the project/repo context, or 404.
pub async fn resolve_repo(
    State(state): State<AppState>,
    Query(params): Query<ResolveParams>,
) -> Result<Json<repo::RepoContext>, ApiError> {
    let ctx = state
        .blocking(move |conn| repo::resolve(conn, &params.remote).map_err(ApiError::from))
        .await?;
    match ctx {
        Some(ctx) => Ok(Json(ctx)),
        None => Err(ApiError {
            status: StatusCode::NOT_FOUND,
            code: "unregistered_repo",
            message: "no registered repo matches that remote".into(),
            details: None,
        }),
    }
}

/// Query for `GET /briefing`.
#[derive(Debug, Deserialize)]
pub struct BriefingParams {
    /// Repo slug to scope the briefing to.
    pub repo: String,
}

/// `GET /api/v1/briefing?repo=…` → the repo-scoped session briefing.
pub async fn briefing(
    State(state): State<AppState>,
    Query(params): Query<BriefingParams>,
) -> Result<Json<repo::Briefing>, ApiError> {
    let b = state
        .blocking(move |conn| repo::briefing(conn, &params.repo).map_err(ApiError::from))
        .await?;
    Ok(Json(b))
}

// ----- saved views -----------------------------------------------------------

/// `POST /projects/:slug/views` body.
#[derive(Debug, Deserialize)]
pub struct CreateView {
    /// Display name.
    pub name: String,
    /// The query to save (same shape as the `GET /items` filters).
    #[serde(default)]
    pub query: ItemFilter,
    /// Whether the view is visible to the whole team.
    #[serde(default)]
    pub shared: bool,
}

/// `POST /api/v1/projects/:slug/views`
pub async fn create_view(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(project): Path<String>,
    Json(body): Json<CreateView>,
) -> Result<(StatusCode, Json<view::View>), ApiError> {
    let v = state
        .blocking(move |conn| {
            view::create(conn, &actor, &project, &body.name, body.query, body.shared)
                .map_err(ApiError::from)
        })
        .await?;
    Ok((StatusCode::CREATED, Json(v)))
}

/// `GET /api/v1/projects/:slug/views`
pub async fn list_views(
    State(state): State<AppState>,
    Path(project): Path<String>,
) -> Result<Json<Vec<view::View>>, ApiError> {
    let views = state
        .blocking(move |conn| view::list(conn, &project).map_err(ApiError::from))
        .await?;
    Ok(Json(views))
}

/// `GET /api/v1/views/:id`
pub async fn get_view(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<view::View>, ApiError> {
    let v = state
        .blocking(move |conn| view::get(conn, id).map_err(ApiError::from))
        .await?;
    Ok(Json(v))
}

/// `DELETE /api/v1/views/:id`
pub async fn delete_view(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .blocking(move |conn| view::delete(conn, id).map_err(ApiError::from))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/v1/views/:id/items` — run the view.
pub async fn run_view(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<Vec<ItemSummary>>, ApiError> {
    let items = state
        .blocking(move |conn| view::run(conn, id).map_err(ApiError::from))
        .await?;
    Ok(Json(items))
}

/// serde helper: distinguish an absent field from an explicit `null`.
fn double_option<'de, D, T>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(de)?))
}
