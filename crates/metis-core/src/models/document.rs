//! Document model definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Default)]
#[sqlx(rename_all = "lowercase")]
pub enum DocumentType {
    #[default]
    Task,
    Vision,
    Strategy,
    Initiative,
    Adr,
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DocumentType::Vision => write!(f, "vision"),
            DocumentType::Strategy => write!(f, "strategy"),
            DocumentType::Initiative => write!(f, "initiative"),
            DocumentType::Task => write!(f, "task"),
            DocumentType::Adr => write!(f, "adr"),
        }
    }
}

impl std::str::FromStr for DocumentType {
    type Err = crate::MetisError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vision" => Ok(DocumentType::Vision),
            "strategy" => Ok(DocumentType::Strategy),
            "initiative" => Ok(DocumentType::Initiative),
            "task" => Ok(DocumentType::Task),
            "adr" => Ok(DocumentType::Adr),
            _ => Err(crate::MetisError::InvalidDocumentType {
                document_type: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub filepath: String,
    pub document_type: DocumentType,
    pub level: DocumentType, // Same as document_type
    pub status: String,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub content_hash: String,
    pub frontmatter: serde_json::Value,
    pub exit_criteria_met: bool,
    pub content: Option<String>, // Body without frontmatter
    pub file_size: Option<i64>,
    pub file_modified_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum Complexity {
    S,  // Small
    M,  // Medium
    L,  // Large
    XL, // Extra Large
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum RelationshipType {
    Parent,
    Blocks,
    Supersedes,
    Related,
}

#[derive(Debug, Clone)]
pub struct DocumentRelationship {
    pub from_id: String,
    pub to_id: String,
    pub relationship_type: RelationshipType,
    pub created_at: DateTime<Utc>,
}
