//! Work item operations — the heart of the service layer.
//!
//! Covers creation (short-code allocation + body storage + audit), reads
//! (resolving the body from the object store plus relations), edits (new
//! content object + history event, with an optional optimistic-concurrency
//! precondition), workflow transitions (via the [`workflow`](crate::workflow)
//! engine), archival, and links. Every mutation runs in a transaction and
//! writes an `events` row carrying the [`Actor`].

use diesel::prelude::*;
use diesel_dualdb::types::{Json, Timestamp, Uuid};
use diesel_dualdb::DualConnection;

use super::{Result, ServiceError};
use crate::actor::Actor;
use crate::models::{ExitCriterion, ItemDetail, Link, WorkItemRow};
use crate::objects::ObjectStore;
use crate::schema::{
    events, exit_criteria, projects, work_item_links, work_item_repos, work_item_tags, work_items,
};
use crate::workflow::config::ProjectConfig;
use crate::workflow::short_code::format_short_code;

/// Fields for creating an item.
#[derive(Debug, Clone, Default)]
pub struct NewItem {
    /// Work item type name (must exist in the project config).
    pub item_type: String,
    /// Title.
    pub title: String,
    /// Markdown body (stored in the object store; empty means no body).
    pub body: String,
    /// Parent item short code, if any.
    pub parent: Option<String>,
    /// Assignee user id, if any.
    pub assignee: Option<uuid::Uuid>,
    /// Tags.
    pub tags: Vec<String>,
    /// Referenced repo slugs.
    pub repos: Vec<String>,
    /// Exit criteria texts (created unmet).
    pub exit_criteria: Vec<String>,
}

/// A partial edit. `None` fields are left unchanged. `assignee: Some(None)`
/// clears the assignee.
#[derive(Debug, Clone, Default)]
pub struct ItemEdit {
    /// New title.
    pub title: Option<String>,
    /// New body (writes a new content object, recording history).
    pub body: Option<String>,
    /// New assignee (`Some(None)` clears).
    pub assignee: Option<Option<uuid::Uuid>>,
    /// Replacement tag set.
    pub tags: Option<Vec<String>>,
    /// Replacement exit-criteria checklist.
    pub exit_criteria: Option<Vec<ExitCriterion>>,
    /// Optimistic-concurrency precondition: if set, the item's current
    /// `content_key` must equal this, else [`ServiceError::Conflict`].
    pub expected_content_key: Option<Option<String>>,
}

const MAX_SHORTCODE_RETRIES: usize = 5;

/// Create a work item. Allocates a short code from the per-(project,type)
/// sequence inside the transaction and retries on the (rare) collision, so
/// concurrent creates never produce a duplicate code.
pub fn create(
    conn: &mut DualConnection,
    store: &dyn ObjectStore,
    actor: &Actor,
    project_slug: &str,
    new: &NewItem,
) -> Result<ItemDetail> {
    let project = projects::table
        .filter(projects::slug.eq(project_slug))
        .select((projects::id, projects::slug, projects::config))
        .first::<(Uuid, String, Json<ProjectConfig>)>(conn)
        .optional()?
        .ok_or_else(|| ServiceError::NotFound(format!("project {project_slug:?}")))?;
    let (project_id, slug, Json(config)) = project;

    let ty = config.item_type(&new.item_type).ok_or_else(|| {
        ServiceError::Validation(format!("unknown work item type {:?}", new.item_type))
    })?;
    let initial_phase = ty.initial_phase().to_string();
    let letter = ty.letter.clone();
    let prefix = slug.to_uppercase();

    // Validate parent/root rules and resolve parent id.
    let parent_id = resolve_parent(
        conn,
        project_id.0,
        &config,
        &new.item_type,
        new.parent.as_deref(),
    )?;

    let mut attempt = 0;
    loop {
        attempt += 1;
        let result = conn.transaction::<uuid::Uuid, ServiceError, _>(|conn| {
            // Next sequence for this (project, type).
            let max_seq: Option<i32> = work_items::table
                .filter(work_items::project_id.eq(project_id))
                .filter(work_items::item_type.eq(&new.item_type))
                .select(diesel::dsl::max(work_items::seq))
                .first(conn)?;
            let seq = max_seq.unwrap_or(0) + 1;
            let short_code = format_short_code(&prefix, &letter, seq as u32);

            // Body object first (write-before-commit; db-blob shares this tx).
            let content_key = if new.body.is_empty() {
                None
            } else {
                Some(store.put(conn, new.body.as_bytes())?)
            };

            let id = uuid::Uuid::new_v4();
            let now = chrono::Utc::now();
            diesel::insert_into(work_items::table)
                .values((
                    work_items::id.eq(Uuid(id)),
                    work_items::project_id.eq(project_id),
                    work_items::short_code.eq(&short_code),
                    work_items::seq.eq(seq),
                    work_items::item_type.eq(&new.item_type),
                    work_items::phase.eq(&initial_phase),
                    work_items::title.eq(&new.title),
                    work_items::content_key.eq(content_key.as_ref()),
                    work_items::parent_id.eq(parent_id.map(Uuid)),
                    work_items::assignee.eq(new.assignee.map(Uuid)),
                    work_items::created_by.eq(Uuid(actor.user)),
                    work_items::archived.eq(false),
                    work_items::created_at.eq(Timestamp(now)),
                    work_items::updated_at.eq(Timestamp(now)),
                ))
                .execute(conn)?;

            insert_tags(conn, id, &new.tags)?;
            attach_repos(conn, project_id.0, id, &new.repos)?;
            insert_exit_criteria(conn, id, &new.exit_criteria)?;
            record_event(
                conn,
                id,
                actor,
                "created",
                serde_json::json!({ "short_code": short_code, "content_key": content_key }),
            )?;

            Ok(id)
        });

        match result {
            Ok(id) => return get_by_id(conn, store, id),
            Err(ServiceError::Db(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ))) if attempt < MAX_SHORTCODE_RETRIES => {
                // Lost a short-code race; recompute in a fresh transaction.
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

/// Fetch an item by short code, resolving its body and relations.
pub fn get(
    conn: &mut DualConnection,
    store: &dyn ObjectStore,
    short_code: &str,
) -> Result<ItemDetail> {
    let row = work_items::table
        .filter(work_items::short_code.eq(short_code))
        .select(WorkItemRow::as_select())
        .first(conn)
        .optional()?
        .ok_or_else(|| ServiceError::NotFound(format!("item {short_code:?}")))?;
    detail_from_row(conn, store, row)
}

fn get_by_id(
    conn: &mut DualConnection,
    store: &dyn ObjectStore,
    id: uuid::Uuid,
) -> Result<ItemDetail> {
    let row = work_items::table
        .filter(work_items::id.eq(Uuid(id)))
        .select(WorkItemRow::as_select())
        .first(conn)?;
    detail_from_row(conn, store, row)
}

/// Edit mutable fields. Writes a new body object (recording the prior key in an
/// event) when `body` changes.
pub fn edit(
    conn: &mut DualConnection,
    store: &dyn ObjectStore,
    actor: &Actor,
    short_code: &str,
    edit: &ItemEdit,
) -> Result<ItemDetail> {
    let id = conn.transaction::<uuid::Uuid, ServiceError, _>(|conn| {
        let row = work_items::table
            .filter(work_items::short_code.eq(short_code))
            .select(WorkItemRow::as_select())
            .first(conn)
            .optional()?
            .ok_or_else(|| ServiceError::NotFound(format!("item {short_code:?}")))?;

        // Optimistic concurrency.
        if let Some(expected) = &edit.expected_content_key {
            if &row.content_key != expected {
                return Err(ServiceError::Conflict(format!(
                    "item {short_code:?} was modified (content_key mismatch)"
                )));
            }
        }

        let now = chrono::Utc::now();

        if let Some(title) = &edit.title {
            diesel::update(work_items::table.filter(work_items::id.eq(row.id)))
                .set((
                    work_items::title.eq(title),
                    work_items::updated_at.eq(Timestamp(now)),
                ))
                .execute(conn)?;
        }
        if let Some(assignee) = &edit.assignee {
            diesel::update(work_items::table.filter(work_items::id.eq(row.id)))
                .set((
                    work_items::assignee.eq(assignee.map(Uuid)),
                    work_items::updated_at.eq(Timestamp(now)),
                ))
                .execute(conn)?;
        }
        if let Some(tags) = &edit.tags {
            diesel::delete(work_item_tags::table.filter(work_item_tags::item_id.eq(row.id)))
                .execute(conn)?;
            insert_tags(conn, row.id.0, tags)?;
        }
        if let Some(criteria) = &edit.exit_criteria {
            diesel::delete(exit_criteria::table.filter(exit_criteria::item_id.eq(row.id)))
                .execute(conn)?;
            insert_exit_criteria_full(conn, row.id.0, criteria)?;
        }
        if let Some(body) = &edit.body {
            let new_key = if body.is_empty() {
                None
            } else {
                Some(store.put(conn, body.as_bytes())?)
            };
            diesel::update(work_items::table.filter(work_items::id.eq(row.id)))
                .set((
                    work_items::content_key.eq(new_key.as_ref()),
                    work_items::updated_at.eq(Timestamp(now)),
                ))
                .execute(conn)?;
            // Record history: prior + new content keys (GC treats both as live).
            record_event(
                conn,
                row.id.0,
                actor,
                "edited_body",
                serde_json::json!({
                    "prev_content_key": row.content_key,
                    "new_content_key": new_key,
                }),
            )?;
        } else {
            record_event(conn, row.id.0, actor, "edited", serde_json::json!({}))?;
        }

        Ok(row.id.0)
    })?;

    get_by_id(conn, store, id)
}

/// Transition an item to a new phase. `to == None` advances to the next phase.
/// Enforces the type's workflow (adjacency + exit-criteria gate) unless `force`.
pub fn transition(
    conn: &mut DualConnection,
    store: &dyn ObjectStore,
    actor: &Actor,
    short_code: &str,
    to: Option<&str>,
    force: bool,
) -> Result<ItemDetail> {
    let id = conn.transaction::<uuid::Uuid, ServiceError, _>(|conn| {
        let row = work_items::table
            .filter(work_items::short_code.eq(short_code))
            .select(WorkItemRow::as_select())
            .first(conn)
            .optional()?
            .ok_or_else(|| ServiceError::NotFound(format!("item {short_code:?}")))?;

        let config = project_config(conn, row.project_id.0)?;
        let ty = config.item_type(&row.item_type).ok_or_else(|| {
            ServiceError::Validation(format!("unknown work item type {:?}", row.item_type))
        })?;

        let met = exit_criteria_all_met(conn, row.id.0)?;
        let outcome = ty.resolve_transition(&row.phase, to, force, met)?;

        diesel::update(work_items::table.filter(work_items::id.eq(row.id)))
            .set((
                work_items::phase.eq(&outcome.to),
                work_items::updated_at.eq(Timestamp(chrono::Utc::now())),
            ))
            .execute(conn)?;
        record_event(
            conn,
            row.id.0,
            actor,
            "transitioned",
            serde_json::json!({ "from": outcome.from, "to": outcome.to, "force": force }),
        )?;
        Ok(row.id.0)
    })?;

    get_by_id(conn, store, id)
}

/// Archive an item.
pub fn archive(
    conn: &mut DualConnection,
    store: &dyn ObjectStore,
    actor: &Actor,
    short_code: &str,
) -> Result<ItemDetail> {
    let id = conn.transaction::<uuid::Uuid, ServiceError, _>(|conn| {
        let row = work_items::table
            .filter(work_items::short_code.eq(short_code))
            .select(WorkItemRow::as_select())
            .first(conn)
            .optional()?
            .ok_or_else(|| ServiceError::NotFound(format!("item {short_code:?}")))?;
        diesel::update(work_items::table.filter(work_items::id.eq(row.id)))
            .set((
                work_items::archived.eq(true),
                work_items::updated_at.eq(Timestamp(chrono::Utc::now())),
            ))
            .execute(conn)?;
        record_event(conn, row.id.0, actor, "archived", serde_json::json!({}))?;
        Ok(row.id.0)
    })?;
    get_by_id(conn, store, id)
}

/// Add a link from one item to another (`blocks`/`relates`/`supersedes`).
pub fn add_link(
    conn: &mut DualConnection,
    actor: &Actor,
    from_short_code: &str,
    kind: &str,
    to_short_code: &str,
) -> Result<()> {
    let valid = ["blocks", "relates", "supersedes"];
    if !valid.contains(&kind) {
        return Err(ServiceError::Validation(format!(
            "unknown link kind {kind:?}"
        )));
    }
    conn.transaction::<(), ServiceError, _>(|conn| {
        let from = item_ref(conn, from_short_code)?;
        let to = item_ref(conn, to_short_code)?;
        if from.1 != to.1 {
            return Err(ServiceError::Validation(
                "cross-project links are not supported".into(),
            ));
        }
        diesel::insert_into(work_item_links::table)
            .values((
                work_item_links::from_id.eq(Uuid(from.0)),
                work_item_links::to_id.eq(Uuid(to.0)),
                work_item_links::kind.eq(kind),
            ))
            .execute(conn)?;
        record_event(
            conn,
            from.0,
            actor,
            "linked",
            serde_json::json!({ "kind": kind, "to": to_short_code }),
        )?;
        Ok(())
    })
}

/// Remove a link.
pub fn remove_link(
    conn: &mut DualConnection,
    from_short_code: &str,
    kind: &str,
    to_short_code: &str,
) -> Result<()> {
    conn.transaction::<(), ServiceError, _>(|conn| {
        let from = item_ref(conn, from_short_code)?;
        let to = item_ref(conn, to_short_code)?;
        diesel::delete(
            work_item_links::table
                .filter(work_item_links::from_id.eq(Uuid(from.0)))
                .filter(work_item_links::to_id.eq(Uuid(to.0)))
                .filter(work_item_links::kind.eq(kind)),
        )
        .execute(conn)?;
        Ok(())
    })
}

// ----- helpers ---------------------------------------------------------------

/// `(id, project_id)` for a short code.
fn item_ref(conn: &mut DualConnection, short_code: &str) -> Result<(uuid::Uuid, uuid::Uuid)> {
    work_items::table
        .filter(work_items::short_code.eq(short_code))
        .select((work_items::id, work_items::project_id))
        .first::<(Uuid, Uuid)>(conn)
        .optional()?
        .map(|(a, b)| (a.0, b.0))
        .ok_or_else(|| ServiceError::NotFound(format!("item {short_code:?}")))
}

fn project_config(conn: &mut DualConnection, project_id: uuid::Uuid) -> Result<ProjectConfig> {
    let Json(config) = projects::table
        .filter(projects::id.eq(Uuid(project_id)))
        .select(projects::config)
        .first::<Json<ProjectConfig>>(conn)?;
    Ok(config)
}

/// Validate parent/root constraints and return the resolved parent id.
fn resolve_parent(
    conn: &mut DualConnection,
    project_id: uuid::Uuid,
    config: &ProjectConfig,
    item_type: &str,
    parent: Option<&str>,
) -> Result<Option<uuid::Uuid>> {
    let ty = config
        .item_type(item_type)
        .ok_or_else(|| ServiceError::Validation(format!("unknown work item type {item_type:?}")))?;
    match parent {
        None => {
            if !ty.allow_root {
                return Err(ServiceError::Validation(format!(
                    "type {item_type:?} requires a parent"
                )));
            }
            Ok(None)
        }
        Some(code) => {
            let (pid, ppid, ptype) = work_items::table
                .filter(work_items::short_code.eq(code))
                .select((
                    work_items::id,
                    work_items::project_id,
                    work_items::item_type,
                ))
                .first::<(Uuid, Uuid, String)>(conn)
                .optional()?
                .ok_or_else(|| ServiceError::NotFound(format!("parent {code:?}")))?;
            if ppid.0 != project_id {
                return Err(ServiceError::Validation(
                    "parent must be in the same project".into(),
                ));
            }
            if !ty.allowed_parents.contains(&ptype) {
                return Err(ServiceError::Validation(format!(
                    "type {item_type:?} may not have a {ptype:?} parent"
                )));
            }
            Ok(Some(pid.0))
        }
    }
}

fn insert_tags(conn: &mut DualConnection, item_id: uuid::Uuid, tags: &[String]) -> Result<()> {
    for tag in tags {
        diesel::insert_into(work_item_tags::table)
            .values((
                work_item_tags::item_id.eq(Uuid(item_id)),
                work_item_tags::tag.eq(tag),
            ))
            .execute(conn)?;
    }
    Ok(())
}

fn attach_repos(
    conn: &mut DualConnection,
    project_id: uuid::Uuid,
    item_id: uuid::Uuid,
    repo_slugs: &[String],
) -> Result<()> {
    use crate::schema::repos;
    for slug in repo_slugs {
        let repo_id = repos::table
            .filter(repos::project_id.eq(Uuid(project_id)))
            .filter(repos::slug.eq(slug))
            .select(repos::id)
            .first::<Uuid>(conn)
            .optional()?
            .ok_or_else(|| ServiceError::NotFound(format!("repo {slug:?}")))?;
        diesel::insert_into(work_item_repos::table)
            .values((
                work_item_repos::item_id.eq(Uuid(item_id)),
                work_item_repos::repo_id.eq(repo_id),
            ))
            .execute(conn)?;
    }
    Ok(())
}

fn insert_exit_criteria(
    conn: &mut DualConnection,
    item_id: uuid::Uuid,
    texts: &[String],
) -> Result<()> {
    for (i, text) in texts.iter().enumerate() {
        diesel::insert_into(exit_criteria::table)
            .values((
                exit_criteria::id.eq(Uuid(uuid::Uuid::new_v4())),
                exit_criteria::item_id.eq(Uuid(item_id)),
                exit_criteria::ordinal.eq(i as i32),
                exit_criteria::text.eq(text),
                exit_criteria::met.eq(false),
            ))
            .execute(conn)?;
    }
    Ok(())
}

fn insert_exit_criteria_full(
    conn: &mut DualConnection,
    item_id: uuid::Uuid,
    criteria: &[ExitCriterion],
) -> Result<()> {
    for c in criteria {
        diesel::insert_into(exit_criteria::table)
            .values((
                exit_criteria::id.eq(Uuid(uuid::Uuid::new_v4())),
                exit_criteria::item_id.eq(Uuid(item_id)),
                exit_criteria::ordinal.eq(c.ordinal),
                exit_criteria::text.eq(&c.text),
                exit_criteria::met.eq(c.met),
            ))
            .execute(conn)?;
    }
    Ok(())
}

fn exit_criteria_all_met(conn: &mut DualConnection, item_id: uuid::Uuid) -> Result<bool> {
    let unmet: i64 = exit_criteria::table
        .filter(exit_criteria::item_id.eq(Uuid(item_id)))
        .filter(exit_criteria::met.eq(false))
        .count()
        .get_result(conn)?;
    Ok(unmet == 0)
}

fn record_event(
    conn: &mut DualConnection,
    item_id: uuid::Uuid,
    actor: &Actor,
    kind: &str,
    payload: serde_json::Value,
) -> Result<()> {
    diesel::insert_into(events::table)
        .values((
            events::id.eq(Uuid(uuid::Uuid::new_v4())),
            events::item_id.eq(Uuid(item_id)),
            events::actor_user.eq(Uuid(actor.user)),
            events::actor_agent.eq(actor.agent.as_deref()),
            events::kind.eq(kind),
            events::payload.eq(Json(payload)),
            events::created_at.eq(Timestamp(chrono::Utc::now())),
        ))
        .execute(conn)?;
    Ok(())
}

fn detail_from_row(
    conn: &mut DualConnection,
    store: &dyn ObjectStore,
    row: WorkItemRow,
) -> Result<ItemDetail> {
    let body = match &row.content_key {
        Some(key) => store
            .get(conn, key)?
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default(),
        None => String::new(),
    };

    let tags: Vec<String> = work_item_tags::table
        .filter(work_item_tags::item_id.eq(row.id))
        .select(work_item_tags::tag)
        .load(conn)?;

    let repo_slugs: Vec<String> = {
        use crate::schema::repos;
        work_item_repos::table
            .inner_join(repos::table.on(repos::id.eq(work_item_repos::repo_id)))
            .filter(work_item_repos::item_id.eq(row.id))
            .select(repos::slug)
            .load(conn)?
    };

    let raw_links: Vec<(Uuid, String)> = work_item_links::table
        .filter(work_item_links::from_id.eq(row.id))
        .select((work_item_links::to_id, work_item_links::kind))
        .load::<(Uuid, String)>(conn)?;
    let mut links = Vec::with_capacity(raw_links.len());
    for (to_id, kind) in raw_links {
        let target: Option<String> = work_items::table
            .filter(work_items::id.eq(to_id))
            .select(work_items::short_code)
            .first(conn)
            .optional()?;
        if let Some(target) = target {
            links.push(Link { kind, target });
        }
    }

    let criteria: Vec<ExitCriterion> = exit_criteria::table
        .filter(exit_criteria::item_id.eq(row.id))
        .order(exit_criteria::ordinal.asc())
        .select((
            exit_criteria::ordinal,
            exit_criteria::text,
            exit_criteria::met,
        ))
        .load::<(i32, String, bool)>(conn)?
        .into_iter()
        .map(|(ordinal, text, met)| ExitCriterion { ordinal, text, met })
        .collect();

    let parent = match row.parent_id {
        Some(pid) => work_items::table
            .filter(work_items::id.eq(pid))
            .select(work_items::short_code)
            .first(conn)
            .optional()?,
        None => None,
    };

    Ok(ItemDetail {
        id: row.id.0,
        short_code: row.short_code,
        project_id: row.project_id.0,
        item_type: row.item_type,
        phase: row.phase,
        title: row.title,
        body,
        content_key: row.content_key,
        parent,
        assignee: row.assignee.map(|u| u.0),
        created_by: row.created_by.0,
        archived: row.archived,
        tags,
        repos: repo_slugs,
        links,
        exit_criteria: criteria,
    })
}
