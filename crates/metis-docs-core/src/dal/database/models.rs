use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Queryable,
    Selectable,
    Insertable,
    AsChangeset,
    QueryableByName,
    Debug,
    Clone,
    Serialize,
    Deserialize,
)]
#[diesel(table_name = crate::dal::database::schema::documents)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Document {
    pub filepath: String,
    pub id: String,
    pub title: String,
    pub document_type: String,
    pub created_at: f64,
    pub updated_at: f64,
    pub archived: bool,
    pub exit_criteria_met: bool,
    pub file_hash: String,
    pub frontmatter_json: String,
    pub content: Option<String>,
    pub phase: String,
    pub strategy_id: Option<String>,
    pub initiative_id: Option<String>,
    pub short_code: String,
}

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::dal::database::schema::document_relationships)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DocumentRelationship {
    pub child_id: String,
    pub parent_id: String,
    pub child_filepath: String,
    pub parent_filepath: String,
}

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::dal::database::schema::document_tags)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DocumentTag {
    pub document_filepath: String,
    pub tag: String,
}

// Insertable version for creating new documents
#[derive(Insertable)]
#[diesel(table_name = crate::dal::database::schema::documents)]
pub struct NewDocument {
    pub filepath: String,
    pub id: String,
    pub title: String,
    pub document_type: String,
    pub created_at: f64,
    pub updated_at: f64,
    pub archived: bool,
    pub exit_criteria_met: bool,
    pub file_hash: String,
    pub frontmatter_json: String,
    pub content: Option<String>,
    pub phase: String,
    pub strategy_id: Option<String>,
    pub initiative_id: Option<String>,
    pub short_code: String,
}

// Document search needs separate structs because rowid is auto-generated
#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::dal::database::schema::document_search)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DocumentSearch {
    pub rowid: i32,
    pub document_filepath: String,
    pub content: Option<String>,
    pub title: Option<String>,
    pub document_type: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::dal::database::schema::document_search)]
pub struct NewDocumentSearch {
    pub document_filepath: String,
    pub content: Option<String>,
    pub title: Option<String>,
    pub document_type: Option<String>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::dal::database::schema::configuration)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Configuration {
    pub key: String,
    pub value: String,
    pub updated_at: f64,
}
