use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Document metadata containing timestamps and other document properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub exit_criteria_met: bool,
    pub short_code: String,
}

impl DocumentMetadata {
    /// Create new metadata with current timestamps and short code
    pub fn new(short_code: String) -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            exit_criteria_met: false,
            short_code,
        }
    }

    /// Create metadata from parsed frontmatter data
    pub fn from_frontmatter(
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        exit_criteria_met: bool,
        short_code: String,
    ) -> Self {
        Self {
            created_at,
            updated_at,
            exit_criteria_met,
            short_code,
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

// Note: No Default implementation as short_code is required
