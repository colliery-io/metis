//! Project operations: create, read, update config. (Repo registry lives in
//! [`super::repo`].)

use diesel::prelude::*;
use diesel_dualdb::types::{Json, Timestamp, Uuid};
use diesel_dualdb::DualConnection;

use super::{Result, ServiceError};
use crate::models::ProjectRow;
use crate::schema::projects;
use crate::workflow::config::ProjectConfig;

/// Create a project. `config` defaults to the Flight Levels configuration and
/// is validated before insert.
pub fn create(
    conn: &mut DualConnection,
    slug: &str,
    name: &str,
    config: Option<ProjectConfig>,
) -> Result<ProjectRow> {
    if slug.trim().is_empty() {
        return Err(ServiceError::Validation("project slug is empty".into()));
    }
    let config = config.unwrap_or_else(ProjectConfig::default_flight_levels);
    config.validate()?;

    let now = chrono::Utc::now();
    let id = uuid::Uuid::new_v4();
    diesel::insert_into(projects::table)
        .values((
            projects::id.eq(Uuid(id)),
            projects::slug.eq(slug),
            projects::name.eq(name),
            projects::config.eq(Json(config)),
            projects::created_at.eq(Timestamp(now)),
            projects::updated_at.eq(Timestamp(now)),
        ))
        .execute(conn)?;

    get(conn, slug)
}

/// Fetch a project by slug.
pub fn get(conn: &mut DualConnection, slug: &str) -> Result<ProjectRow> {
    projects::table
        .filter(projects::slug.eq(slug))
        .select(ProjectRow::as_select())
        .first(conn)
        .optional()?
        .ok_or_else(|| ServiceError::NotFound(format!("project {slug:?}")))
}

/// Replace a project's type/workflow configuration (validated first).
pub fn update_config(
    conn: &mut DualConnection,
    slug: &str,
    config: ProjectConfig,
) -> Result<ProjectRow> {
    config.validate()?;
    let n = diesel::update(projects::table.filter(projects::slug.eq(slug)))
        .set((
            projects::config.eq(Json(config)),
            projects::updated_at.eq(Timestamp(chrono::Utc::now())),
        ))
        .execute(conn)?;
    if n == 0 {
        return Err(ServiceError::NotFound(format!("project {slug:?}")));
    }
    get(conn, slug)
}
