//! Full-text search over work items — the one sanctioned per-backend divergence.
//!
//! Structure is searched the same way everywhere through this API; the storage
//! and query differ by backend (SQLite FTS5 `MATCH` vs Postgres `tsvector @@`)
//! behind it. The index (`work_item_search`, keyed by short code) is fed at
//! write time by [`crate::service::item`]. Raw SQL via [`diesel::sql_query`] is
//! the dispatch escape hatch; placeholder syntax (`?` vs `$n`) and the match
//! operator are chosen by the live connection's backend.

use diesel::prelude::*;
use diesel::sql_types::{BigInt, Text};
use diesel_dualdb::pool::Backend;
use diesel_dualdb::DualConnection;

use super::Result;

#[derive(QueryableByName)]
struct CodeRow {
    #[diesel(sql_type = Text)]
    short_code: String,
}

fn backend(conn: &DualConnection) -> Backend {
    match conn {
        DualConnection::Pg(_) => Backend::Postgres,
        DualConnection::Sqlite(_) => Backend::Sqlite,
    }
}

/// Upsert an item's searchable content (title + body). Delete-then-insert so it
/// works uniformly for the FTS5 virtual table and the Postgres table.
pub fn index(conn: &mut DualConnection, short_code: &str, title: &str, body: &str) -> Result<()> {
    let content = format!("{title}\n{body}");
    let (del, ins) = match backend(conn) {
        Backend::Sqlite => (
            "DELETE FROM work_item_search WHERE short_code = ?",
            "INSERT INTO work_item_search(short_code, content) VALUES (?, ?)",
        ),
        Backend::Postgres => (
            "DELETE FROM work_item_search WHERE short_code = $1",
            "INSERT INTO work_item_search(short_code, content) VALUES ($1, $2)",
        ),
    };
    diesel::sql_query(del)
        .bind::<Text, _>(short_code)
        .execute(conn)?;
    diesel::sql_query(ins)
        .bind::<Text, _>(short_code)
        .bind::<Text, _>(&content)
        .execute(conn)?;
    Ok(())
}

/// Remove an item from the index (e.g. on hard delete).
pub fn remove(conn: &mut DualConnection, short_code: &str) -> Result<()> {
    let sql = match backend(conn) {
        Backend::Sqlite => "DELETE FROM work_item_search WHERE short_code = ?",
        Backend::Postgres => "DELETE FROM work_item_search WHERE short_code = $1",
    };
    diesel::sql_query(sql)
        .bind::<Text, _>(short_code)
        .execute(conn)?;
    Ok(())
}

/// Short codes matching `q`, best first, up to `limit`. Empty (or all-punctuation)
/// queries return no matches rather than erroring.
pub fn search(conn: &mut DualConnection, q: &str, limit: i64) -> Result<Vec<String>> {
    let rows: Vec<CodeRow> = match backend(conn) {
        Backend::Sqlite => {
            // FTS5's MATCH grammar treats punctuation specially; reduce the
            // user query to bare alphanumeric terms (implicit AND) so arbitrary
            // input can't be a syntax error.
            let safe: String = q
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { ' ' })
                .collect();
            if safe.split_whitespace().next().is_none() {
                return Ok(vec![]);
            }
            diesel::sql_query(
                "SELECT short_code FROM work_item_search \
                 WHERE work_item_search MATCH ? ORDER BY rank LIMIT ?",
            )
            .bind::<Text, _>(safe.trim())
            .bind::<BigInt, _>(limit)
            .load(conn)?
        }
        Backend::Postgres => {
            if q.trim().is_empty() {
                return Ok(vec![]);
            }
            diesel::sql_query(
                "SELECT short_code FROM work_item_search \
                 WHERE tsv @@ plainto_tsquery('english', $1) \
                 ORDER BY ts_rank(tsv, plainto_tsquery('english', $1)) DESC LIMIT $2",
            )
            .bind::<Text, _>(q)
            .bind::<BigInt, _>(limit)
            .load(conn)?
        }
    };
    Ok(rows.into_iter().map(|r| r.short_code).collect())
}
