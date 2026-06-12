//! Saved views: named, reusable item queries (a stored [`ItemFilter`]).
//!
//! A view belongs to a project, is owned by the actor that created it, may be
//! shared, and is "run" by applying its stored query through [`query::list`].

use diesel::prelude::*;
use diesel_dualdb::types::{Json, Uuid};
use diesel_dualdb::DualConnection;
use serde::{Deserialize, Serialize};

use super::query::{self, ItemFilter};
use super::{project, Result, ServiceError};
use crate::actor::Actor;
use crate::models::ItemSummary;
use crate::schema::{projects, views};

/// A saved view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    /// Identity.
    pub id: uuid::Uuid,
    /// Owning project slug.
    pub project: String,
    /// Display name.
    pub name: String,
    /// The stored query.
    pub query: ItemFilter,
    /// Whether visible to the whole team.
    pub shared: bool,
    /// Owner user id.
    pub owner: uuid::Uuid,
}

type ViewCols = (Uuid, Uuid, String, Json<ItemFilter>, bool, Uuid);

fn select() -> (
    views::id,
    views::project_id,
    views::name,
    views::query_json,
    views::shared,
    views::owner,
) {
    (
        views::id,
        views::project_id,
        views::name,
        views::query_json,
        views::shared,
        views::owner,
    )
}

fn to_view(conn: &mut DualConnection, row: ViewCols) -> Result<View> {
    let (id, project_id, name, Json(query), shared, owner) = row;
    let project: String = projects::table
        .filter(projects::id.eq(project_id))
        .select(projects::slug)
        .first(conn)?;
    Ok(View {
        id: id.0,
        project,
        name,
        query,
        shared,
        owner: owner.0,
    })
}

/// Create a saved view owned by `actor`.
pub fn create(
    conn: &mut DualConnection,
    actor: &Actor,
    project_slug: &str,
    name: &str,
    query: ItemFilter,
    shared: bool,
) -> Result<View> {
    let proj = project::get(conn, project_slug)?;
    let id = uuid::Uuid::new_v4();
    diesel::insert_into(views::table)
        .values((
            views::id.eq(Uuid(id)),
            views::project_id.eq(Uuid(proj.id.0)),
            views::owner.eq(Uuid(actor.user)),
            views::name.eq(name),
            views::query_json.eq(Json(query)),
            views::shared.eq(shared),
        ))
        .execute(conn)?;
    get(conn, id)
}

/// Fetch a view by id.
pub fn get(conn: &mut DualConnection, id: uuid::Uuid) -> Result<View> {
    let row = views::table
        .filter(views::id.eq(Uuid(id)))
        .select(select())
        .first::<ViewCols>(conn)
        .optional()?
        .ok_or_else(|| ServiceError::NotFound(format!("view {id}")))?;
    to_view(conn, row)
}

/// List a project's views (shared and owned alike).
pub fn list(conn: &mut DualConnection, project_slug: &str) -> Result<Vec<View>> {
    let proj = project::get(conn, project_slug)?;
    let rows = views::table
        .filter(views::project_id.eq(Uuid(proj.id.0)))
        .order(views::name.asc())
        .select(select())
        .load::<ViewCols>(conn)?;
    rows.into_iter().map(|r| to_view(conn, r)).collect()
}

/// Delete a view by id.
pub fn delete(conn: &mut DualConnection, id: uuid::Uuid) -> Result<()> {
    let n = diesel::delete(views::table.filter(views::id.eq(Uuid(id)))).execute(conn)?;
    if n == 0 {
        return Err(ServiceError::NotFound(format!("view {id}")));
    }
    Ok(())
}

/// Run a view: apply its stored query, scoped to the view's project.
pub fn run(conn: &mut DualConnection, id: uuid::Uuid) -> Result<Vec<ItemSummary>> {
    let view = get(conn, id)?;
    let mut filter = view.query;
    // A view is project-scoped; force its project unless the query names one.
    if filter.project.is_none() {
        filter.project = Some(view.project);
    }
    query::list(conn, &filter)
}
