//! # metis-server
//!
//! The Metis 3.0 server: an axum binary exposing the [`metis_core`] domain over
//! HTTP, with bearer-token auth and an admin CLI for bootstrap. This task
//! (METIS-T-0130) delivers the skeleton — config, state, auth middleware,
//! `/healthz`, a `/api/v1/whoami` probe, and admin commands. The full `/api/v1`
//! surface lands in METIS-T-0131.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod admin;
pub mod auth;
pub mod config;
pub mod error;
pub mod hook;
pub mod mcp;
pub mod routes;
pub mod state;
pub mod user_config;

use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{middleware, Extension, Json, Router};
use metis_core::actor::Actor;
use metis_core::service::user;
use metis_core::{db, objects::ObjectStore};
use serde_json::json;

use crate::config::ServerConfig;
use crate::error::ApiError;
use crate::state::AppState;

/// Build application state: connect + migrate, construct the object store, and
/// (in local mode) ensure the implicit user exists.
pub fn build_state(config: ServerConfig) -> anyhow::Result<AppState> {
    let pool = db::connect_and_migrate(&config.database_url)?;
    let store: Arc<dyn ObjectStore + Send + Sync> = config.object_store.build()?.into();

    let local_user = if config.local {
        let mut conn = pool.get()?;
        let user = match user::get_user_by_name(&mut conn, "local") {
            Ok(u) => u,
            Err(_) => user::create_user(&mut conn, "local", "Local User", None, true)?,
        };
        Some(user.id.0)
    } else {
        None
    };

    Ok(AppState {
        pool,
        store,
        config: Arc::new(config),
        local_user,
    })
}

/// Build the HTTP router. `/healthz` is public; everything under `/api/v1`
/// requires auth.
pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/api/v1/whoami", get(whoami))
        .route("/api/v1/projects", post(routes::create_project))
        .route(
            "/api/v1/projects/:slug",
            get(routes::get_project).patch(routes::update_project_config),
        )
        .route(
            "/api/v1/items",
            get(routes::list_items).post(routes::create_item),
        )
        .route(
            "/api/v1/items/:short_code",
            get(routes::get_item).patch(routes::patch_item),
        )
        .route(
            "/api/v1/items/:short_code/transition",
            post(routes::transition_item),
        )
        .route(
            "/api/v1/items/:short_code/archive",
            post(routes::archive_item),
        )
        .route(
            "/api/v1/items/:short_code/links",
            post(routes::add_link).delete(routes::remove_link),
        )
        .route(
            "/api/v1/projects/:slug/repos",
            get(routes::list_repos).post(routes::register_repo),
        )
        .route("/api/v1/repos/resolve", get(routes::resolve_repo))
        .route("/api/v1/briefing", get(routes::briefing))
        .route(
            "/api/v1/projects/:slug/views",
            get(routes::list_views).post(routes::create_view),
        )
        .route(
            "/api/v1/views/:id",
            get(routes::get_view).delete(routes::delete_view),
        )
        .route("/api/v1/views/:id/items", get(routes::run_view))
        // Hosted MCP over Streamable HTTP (per-connection PAT auth via the
        // same middleware). POST only; GET yields 405 (no server push).
        .route("/mcp", post(mcp::http::post_mcp))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));

    Router::new()
        .route("/healthz", get(healthz))
        .merge(protected)
        .with_state(state)
}

/// Bind and serve until shutdown.
pub async fn serve(config: ServerConfig) -> anyhow::Result<()> {
    let bind = config.bind;
    let local = config.local;
    let state = build_state(config)?;
    let app = router(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(%bind, local, "metis server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Liveness probe.
async fn healthz(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

/// Echo the authenticated actor — proves the auth middleware end to end.
async fn whoami(Extension(actor): Extension<Actor>) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(json!({ "user": actor.user, "agent": actor.agent })))
}
