use crate::application::services::DatabaseService;
use crate::dal::database::models::Document;
use crate::Result;
use crate::MetisError;
use std::fs;
use std::path::{Path, PathBuf};

/// Service for reassigning tasks to different parent initiatives or the backlog
pub struct ReassignmentService {
    workspace_dir: PathBuf,
}

/// Result of reassignment operation
#[derive(Debug)]
pub struct ReassignmentResult {
    pub short_code: String,
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub new_parent: Option<String>,
}

/// Backlog category for standalone tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BacklogCategory {
    Bug,
    Feature,
    TechDebt,
}

impl BacklogCategory {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bug" => Some(Self::Bug),
            "feature" => Some(Self::Feature),
            "tech-debt" | "techdebt" | "tech_debt" => Some(Self::TechDebt),
            _ => None,
        }
    }

    pub fn directory_name(&self) -> &'static str {
        match self {
            Self::Bug => "bugs",
            Self::Feature => "features",
            Self::TechDebt => "tech-debt",
        }
    }
}

impl ReassignmentService {
    /// Create a new reassignment service for a workspace
    pub fn new<P: AsRef<Path>>(workspace_dir: P) -> Self {
        let path = workspace_dir.as_ref();
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_default()
                .join(path)
        };

        Self {
            workspace_dir: absolute_path,
        }
    }

    /// Reassign a task to a new parent initiative
    pub async fn reassign_to_initiative(
        &self,
        short_code: &str,
        new_parent_id: &str,
        db_service: &mut DatabaseService,
    ) -> Result<ReassignmentResult> {
        // Find source document
        let source_doc = self.find_task_by_short_code(short_code, db_service)?;

        // Find and validate parent initiative
        let parent_doc = self.find_and_validate_parent(new_parent_id, db_service)?;

        // Determine paths
        let source_path = self.workspace_dir.join(&source_doc.filepath);
        let dest_path = self.compute_initiative_task_path(&parent_doc, &source_doc)?;

        // Perform the move
        self.move_file(&source_path, &dest_path)?;

        Ok(ReassignmentResult {
            short_code: short_code.to_string(),
            old_path: source_path,
            new_path: dest_path,
            new_parent: Some(new_parent_id.to_string()),
        })
    }

    /// Move a task to the backlog
    pub async fn reassign_to_backlog(
        &self,
        short_code: &str,
        category: BacklogCategory,
        db_service: &mut DatabaseService,
    ) -> Result<ReassignmentResult> {
        // Find source document
        let source_doc = self.find_task_by_short_code(short_code, db_service)?;

        // Determine paths
        let source_path = self.workspace_dir.join(&source_doc.filepath);
        let filename = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| MetisError::ValidationFailed {
                message: "Could not determine filename".to_string(),
            })?;

        let dest_path = self.workspace_dir
            .join("backlog")
            .join(category.directory_name())
            .join(filename);

        // Perform the move
        self.move_file(&source_path, &dest_path)?;

        Ok(ReassignmentResult {
            short_code: short_code.to_string(),
            old_path: source_path,
            new_path: dest_path,
            new_parent: None,
        })
    }

    /// Find a task by short code and validate it's a task
    fn find_task_by_short_code(
        &self,
        short_code: &str,
        db_service: &mut DatabaseService,
    ) -> Result<Document> {
        let doc = db_service
            .find_by_short_code(short_code)?
            .ok_or_else(|| MetisError::NotFound(format!(
                "Document '{}' not found",
                short_code
            )))?;

        if doc.document_type != "task" {
            return Err(MetisError::ValidationFailed {
                message: format!(
                    "Only tasks can be reassigned. '{}' is a {}.",
                    short_code, doc.document_type
                ),
            });
        }

        Ok(doc)
    }

    /// Find and validate a parent initiative
    fn find_and_validate_parent(
        &self,
        parent_id: &str,
        db_service: &mut DatabaseService,
    ) -> Result<Document> {
        let parent = db_service
            .find_by_short_code(parent_id)?
            .ok_or_else(|| MetisError::NotFound(format!(
                "Parent initiative '{}' not found",
                parent_id
            )))?;

        if parent.document_type != "initiative" {
            return Err(MetisError::ValidationFailed {
                message: format!(
                    "Parent must be an initiative. '{}' is a {}.",
                    parent_id, parent.document_type
                ),
            });
        }

        // Validate phase
        let phase = parent.phase.to_lowercase();
        if phase != "decompose" && phase != "active" {
            return Err(MetisError::ValidationFailed {
                message: format!(
                    "Initiative '{}' is in '{}' phase. Tasks can only be assigned to initiatives in 'decompose' or 'active' phase.",
                    parent_id, parent.phase
                ),
            });
        }

        Ok(parent)
    }

    /// Compute the destination path for a task under an initiative
    fn compute_initiative_task_path(
        &self,
        parent_doc: &Document,
        source_doc: &Document,
    ) -> Result<PathBuf> {
        let source_path = self.workspace_dir.join(&source_doc.filepath);
        let filename = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| MetisError::ValidationFailed {
                message: "Could not determine source filename".to_string(),
            })?;

        // Get initiative directory from its filepath
        let parent_path = Path::new(&parent_doc.filepath);
        let initiative_dir = parent_path.parent().ok_or_else(|| {
            MetisError::ValidationFailed {
                message: "Could not determine initiative directory".to_string(),
            }
        })?;

        // Tasks go in {initiative_dir}/tasks/
        Ok(self.workspace_dir.join(initiative_dir).join("tasks").join(filename))
    }

    /// Move a file from source to destination
    fn move_file(&self, source: &Path, dest: &Path) -> Result<()> {
        // Validate source exists
        if !source.exists() {
            return Err(MetisError::NotFound(format!(
                "Source file not found: {}",
                source.display()
            )));
        }

        // Check destination doesn't exist
        if dest.exists() {
            return Err(MetisError::ValidationFailed {
                message: format!(
                    "Destination already exists: {}",
                    dest.display()
                ),
            });
        }

        // Same location check
        if source == dest {
            return Err(MetisError::ValidationFailed {
                message: "Task is already at the target location".to_string(),
            });
        }

        // Create destination directory if needed
        if let Some(parent_dir) = dest.parent() {
            if !parent_dir.exists() {
                fs::create_dir_all(parent_dir).map_err(|e| {
                    MetisError::FileSystem(format!(
                        "Failed to create destination directory: {}",
                        e
                    ))
                })?;
            }
        }

        // Move the file
        fs::rename(source, dest).map_err(|e| {
            MetisError::FileSystem(format!("Failed to move file: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backlog_category_parsing() {
        assert_eq!(BacklogCategory::from_str("bug"), Some(BacklogCategory::Bug));
        assert_eq!(BacklogCategory::from_str("feature"), Some(BacklogCategory::Feature));
        assert_eq!(BacklogCategory::from_str("tech-debt"), Some(BacklogCategory::TechDebt));
        assert_eq!(BacklogCategory::from_str("techdebt"), Some(BacklogCategory::TechDebt));
        assert_eq!(BacklogCategory::from_str("tech_debt"), Some(BacklogCategory::TechDebt));
        assert_eq!(BacklogCategory::from_str("invalid"), None);
    }

    #[test]
    fn test_backlog_category_directory() {
        assert_eq!(BacklogCategory::Bug.directory_name(), "bugs");
        assert_eq!(BacklogCategory::Feature.directory_name(), "features");
        assert_eq!(BacklogCategory::TechDebt.directory_name(), "tech-debt");
    }
}
