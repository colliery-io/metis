use crate::dal::database::configuration_repository::ConfigurationRepository;
use crate::{Database, MetisError, Phase, Result, Tag, Vision};
use diesel::{sqlite::SqliteConnection, Connection};
use std::path::{Path, PathBuf};

/// Service for initializing new Metis workspaces
pub struct WorkspaceInitializationService;

/// Result of workspace initialization
pub struct WorkspaceInitializationResult {
    pub metis_dir: PathBuf,
    pub database_path: PathBuf,
    pub vision_path: PathBuf,
}

impl WorkspaceInitializationService {
    /// Initialize a new Metis workspace at the given base path
    /// Creates a .metis directory with database, strategies directory, and default vision
    pub async fn initialize_workspace<P: AsRef<Path>>(
        base_path: P,
        project_name: &str,
    ) -> Result<WorkspaceInitializationResult> {
        Self::initialize_workspace_with_prefix(base_path, project_name, None).await
    }

    /// Initialize a new Metis workspace with an optional custom prefix
    pub async fn initialize_workspace_with_prefix<P: AsRef<Path>>(
        base_path: P,
        project_name: &str,
        prefix: Option<&str>,
    ) -> Result<WorkspaceInitializationResult> {
        let base_path = base_path.as_ref();

        // Create .metis directory
        let metis_dir = base_path.join(".metis");
        std::fs::create_dir_all(&metis_dir)?;

        // Initialize database - check if it already exists and is valid
        let db_path = metis_dir.join("metis.db");
        let db_exists = db_path.exists();

        // Try to create/open database
        let db_result = Database::new(db_path.to_str().unwrap());

        match db_result {
            Ok(_db) => {
                // Database is valid, set up project configuration
                let mut config_repo = ConfigurationRepository::new(
                    SqliteConnection::establish(db_path.to_str().unwrap()).map_err(|e| {
                        MetisError::ConfigurationError(
                            crate::domain::configuration::ConfigurationError::InvalidValue(
                                e.to_string(),
                            ),
                        )
                    })?,
                );

                // Set project prefix if not already set
                if config_repo.get_project_prefix()?.is_none() {
                    let default_prefix = {
                        let p = prefix.unwrap_or("PROJ").to_uppercase();
                        if p.len() > 6 {
                            p.chars().take(6).collect()
                        } else {
                            p
                        }
                    };
                    config_repo.set_project_prefix(&default_prefix)?;
                }
            }
            Err(e) => {
                if db_exists {
                    // Database exists but is invalid, return error
                    return Err(MetisError::FileSystem(format!(
                        "Invalid existing database at {}: {}",
                        db_path.display(),
                        e
                    )));
                } else {
                    // Failed to create new database
                    return Err(MetisError::FileSystem(format!(
                        "Failed to initialize database: {}",
                        e
                    )));
                }
            }
        }

        // Create strategies directory
        let strategies_dir = metis_dir.join("strategies");
        std::fs::create_dir_all(&strategies_dir)?;

        // Create default vision document only if it doesn't exist
        let vision_path = metis_dir.join("vision.md");
        if !vision_path.exists() {
            let vision_path = Self::create_default_vision(&metis_dir, project_name).await?;
            Ok(WorkspaceInitializationResult {
                metis_dir,
                database_path: db_path,
                vision_path,
            })
        } else {
            Ok(WorkspaceInitializationResult {
                metis_dir,
                database_path: db_path,
                vision_path,
            })
        }
    }

    /// Create a new Vision document with defaults and write to file
    async fn create_default_vision(workspace_dir: &Path, title: &str) -> Result<PathBuf> {
        // Generate short code for vision using the database
        let db_path = workspace_dir.join("metis.db");
        let mut config_repo = ConfigurationRepository::new(
            SqliteConnection::establish(&db_path.to_string_lossy()).map_err(|e| {
                MetisError::ConfigurationError(
                    crate::domain::configuration::ConfigurationError::InvalidValue(e.to_string()),
                )
            })?,
        );
        let short_code = config_repo.generate_short_code("vision")?;

        // Create Vision with defaults
        let tags = vec![Tag::Label("vision".to_string()), Tag::Phase(Phase::Draft)];

        let vision = Vision::new(
            title.to_string(),
            tags,
            false, // not archived
            short_code,
        )?;

        // Write to vision.md at workspace root
        let vision_path = workspace_dir.join("vision.md");
        vision.to_file(&vision_path).await?;

        Ok(vision_path)
    }

    /// Check if a directory contains a valid Metis workspace
    pub fn is_workspace(path: &Path) -> bool {
        let metis_dir = path.join(".metis");
        let db_path = metis_dir.join("metis.db");

        metis_dir.exists() && metis_dir.is_dir() && db_path.exists() && db_path.is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_initialize_workspace() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        let result =
            WorkspaceInitializationService::initialize_workspace(base_path, "Test Project").await;
        assert!(result.is_ok());

        let result = result.unwrap();

        // Verify .metis directory was created
        let metis_dir = base_path.join(".metis");
        assert!(metis_dir.exists());
        assert!(metis_dir.is_dir());
        assert_eq!(result.metis_dir, metis_dir);

        // Verify database was created
        let db_path = metis_dir.join("metis.db");
        assert!(db_path.exists());
        assert!(db_path.is_file());
        assert_eq!(result.database_path, db_path);

        // Verify strategies directory was created
        let strategies_dir = metis_dir.join("strategies");
        assert!(strategies_dir.exists());
        assert!(strategies_dir.is_dir());

        // Verify vision.md was created
        let vision_path = metis_dir.join("vision.md");
        assert!(vision_path.exists());
        assert!(vision_path.is_file());
        assert_eq!(result.vision_path, vision_path);

        // Verify vision.md content
        let vision_content = fs::read_to_string(&vision_path).unwrap();
        assert!(vision_content.contains("Test Project"));
        assert!(vision_content.contains("#vision"));
        assert!(vision_content.contains("#phase/draft"));
        assert!(vision_content.contains("archived: false"));
    }

    #[tokio::test]
    async fn test_initialize_workspace_already_exists() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        // First initialization
        let result1 =
            WorkspaceInitializationService::initialize_workspace(base_path, "Test Project").await;
        assert!(result1.is_ok());

        // Get the database size after first initialization
        let db_path = base_path.join(".metis").join("metis.db");
        let original_size = fs::metadata(&db_path).unwrap().len();

        // Second initialization should succeed (idempotent)
        let result2 =
            WorkspaceInitializationService::initialize_workspace(base_path, "Test Project").await;
        assert!(result2.is_ok());

        // Verify database still exists and has a valid size (should be same or similar)
        let new_size = fs::metadata(&db_path).unwrap().len();
        assert!(new_size > 0);

        // Database might be slightly different due to migrations/timestamps, but should be similar size
        assert!(new_size >= original_size / 2 && new_size <= original_size * 2);
    }

    #[test]
    fn test_is_workspace() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        // Not a workspace initially
        assert!(!WorkspaceInitializationService::is_workspace(base_path));

        // Create .metis directory but no database
        let metis_dir = base_path.join(".metis");
        fs::create_dir_all(&metis_dir).unwrap();
        assert!(!WorkspaceInitializationService::is_workspace(base_path));

        // Create database file
        let db_path = metis_dir.join("metis.db");
        fs::write(&db_path, "test").unwrap();
        assert!(WorkspaceInitializationService::is_workspace(base_path));
    }
}
