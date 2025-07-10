use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Document metadata containing timestamps and other document properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub exit_criteria_met: bool,
}

impl DocumentMetadata {
    /// Create new metadata with current timestamps
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            exit_criteria_met: false,
        }
    }

    /// Create metadata from parsed frontmatter data
    pub fn from_frontmatter(
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        exit_criteria_met: bool,
    ) -> Self {
        Self {
            created_at,
            updated_at,
            exit_criteria_met,
        }
    }

    /// Update the updated_at timestamp to now
    pub fn update(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Mark exit criteria as met and update timestamp
    pub fn mark_exit_criteria_met(&mut self) {
        self.exit_criteria_met = true;
        self.update();
    }
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self::new()
    }
}
