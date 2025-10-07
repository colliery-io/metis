/// Shared constants for the Metis documentation system
///
/// Directory and file names
pub const METIS_DIR_NAME: &str = ".metis";
pub const DATABASE_FILE_NAME: &str = "metis.db";
pub const BACKUP_DATABASE_FILE_NAME: &str = "metis.db.backup";
pub const LOG_FILE_NAME: &str = "metis-mcp-server.log";

/// File extensions
pub const MARKDOWN_EXT: &str = ".md";
pub const YAML_EXT: &str = ".yaml";
pub const JSON_EXT: &str = ".json";

/// Document directories
pub const VISION_DIR: &str = "vision";
pub const STRATEGY_DIR: &str = "strategy";
pub const INITIATIVE_DIR: &str = "initiative";
pub const TASK_DIR: &str = "task";
pub const ADR_DIR: &str = "adr";
pub const ARCHIVED_DIR: &str = "archived";

/// Template names
pub const VISION_TEMPLATE: &str = "vision";
pub const STRATEGY_TEMPLATE: &str = "strategy";
pub const INITIATIVE_TEMPLATE: &str = "initiative";
pub const TASK_TEMPLATE: &str = "task";
pub const ADR_TEMPLATE: &str = "adr";

/// Document phases
pub mod phases {
    pub const VISION_DRAFT: &str = "draft";
    pub const VISION_REVIEW: &str = "review";
    pub const VISION_PUBLISHED: &str = "published";

    pub const STRATEGY_SHAPING: &str = "shaping";
    pub const STRATEGY_DESIGN: &str = "design";
    pub const STRATEGY_READY: &str = "ready";
    pub const STRATEGY_ACTIVE: &str = "active";
    pub const STRATEGY_COMPLETED: &str = "completed";

    pub const INITIATIVE_DISCOVERY: &str = "discovery";
    pub const INITIATIVE_DESIGN: &str = "design";
    pub const INITIATIVE_READY: &str = "ready";
    pub const INITIATIVE_DECOMPOSE: &str = "decompose";
    pub const INITIATIVE_ACTIVE: &str = "active";
    pub const INITIATIVE_COMPLETED: &str = "completed";

    pub const TASK_TODO: &str = "todo";
    pub const TASK_DOING: &str = "doing";
    pub const TASK_COMPLETED: &str = "completed";

    pub const ADR_DRAFT: &str = "draft";
    pub const ADR_DISCUSSION: &str = "discussion";
    pub const ADR_DECIDED: &str = "decided";
    pub const ADR_SUPERSEDED: &str = "superseded";
}

/// Complexity levels for initiatives
pub mod complexity {
    pub const XS: &str = "xs";
    pub const S: &str = "s";
    pub const M: &str = "m";
    pub const L: &str = "l";
    pub const XL: &str = "xl";
}

/// Risk levels for strategies
pub mod risk {
    pub const LOW: &str = "low";
    pub const MEDIUM: &str = "medium";
    pub const HIGH: &str = "high";
}

/// Database settings
pub mod database {
    pub const CONNECTION_TIMEOUT_SECS: u64 = 30;
    pub const MAX_RETRIES: u32 = 3;
}

/// File system settings
pub mod filesystem {
    pub const MAX_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024; // 10MB
    pub const BACKUP_RETENTION_DAYS: u32 = 30;
}
