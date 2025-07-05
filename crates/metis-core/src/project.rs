//! Project initialization functionality

use crate::{DocumentContext, DocumentStore, DocumentType, MetisError, Result, TemplateEngine};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for initializing a new Metis project
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub description: Option<String>,
    pub root_path: PathBuf,
}

/// Metadata returned after successful project initialization
#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    pub project_path: PathBuf,
    pub database_path: PathBuf,
}

/// Initialize a new Metis project (idempotent and non-destructive)
pub async fn initialize_project(config: ProjectConfig) -> Result<ProjectMetadata> {
    let project_path = config.root_path.join("metis");
    let database_path = project_path.join(".metis.db");

    // 1. Validate project name (filesystem safety)
    if !is_valid_project_name(&config.name) {
        return Err(MetisError::ValidationFailed {
            message: format!("Invalid project name '{}'. Use only alphanumeric characters, hyphens, and underscores.", config.name),
        });
    }

    // 2. Validate parent directory exists and is writable
    if !config.root_path.exists() {
        return Err(MetisError::ValidationFailed {
            message: format!(
                "Parent directory does not exist: {}",
                config.root_path.display()
            ),
        });
    }

    if !config.root_path.is_dir() {
        return Err(MetisError::ValidationFailed {
            message: format!("Path is not a directory: {}", config.root_path.display()),
        });
    }

    // 3. Create metis directory if it doesn't exist
    if !project_path.exists() {
        fs::create_dir_all(&project_path).map_err(|e| MetisError::ValidationFailed {
            message: format!("Failed to create metis directory: {}", e),
        })?;
    }

    // 4. Test write permissions by creating and removing a temporary file
    let temp_file = project_path.join(".metis_temp_test");
    if let Err(e) = fs::write(&temp_file, "") {
        return Err(MetisError::ValidationFailed {
            message: format!("Directory is not writable: {}", e),
        });
    }
    let _ = fs::remove_file(temp_file); // Ignore errors on cleanup

    // 5. Create directory structure if it doesn't exist
    create_directory_structure(&project_path)?;

    // 6. Initialize database if it doesn't exist
    if !database_path.exists() {
        let database_url = format!("sqlite:{}", database_path.display());
        let _store = DocumentStore::new(&database_url).await?;
    }

    // 7. Create initial vision document if it doesn't exist
    create_initial_vision(&project_path, &config.name, config.description.as_deref())?;

    Ok(ProjectMetadata {
        project_path,
        database_path,
    })
}

/// Create the standard Metis directory structure (idempotent)
fn create_directory_structure(project_path: &Path) -> Result<()> {
    let strategies_dir = project_path.join("strategies");
    let decisions_dir = project_path.join("decisions");

    // create_dir_all is already idempotent - it won't fail if directories exist
    fs::create_dir_all(&strategies_dir).map_err(|e| MetisError::ValidationFailed {
        message: format!("Failed to create strategies directory: {}", e),
    })?;

    fs::create_dir_all(&decisions_dir).map_err(|e| MetisError::ValidationFailed {
        message: format!("Failed to create decisions directory: {}", e),
    })?;

    Ok(())
}

/// Create the initial vision document using the template system (non-destructive)
fn create_initial_vision(
    project_path: &Path,
    project_name: &str,
    description: Option<&str>,
) -> Result<()> {
    let vision_path = project_path.join("vision.md");

    // Only create vision document if it doesn't already exist
    if vision_path.exists() {
        return Ok(());
    }

    let template_engine = TemplateEngine::new()?;

    // Create context for vision document
    let vision_context = DocumentContext::new(format!("{} Vision", project_name));

    // Render the vision document
    let vision_content = template_engine.render_document(&DocumentType::Vision, &vision_context)?;

    // Customize content if description is provided
    let final_content = if let Some(desc) = description {
        // Replace the placeholder purpose section with the provided description
        vision_content.replace("{Why this vision exists and what it aims to achieve}", desc)
    } else {
        vision_content
    };

    // Write vision document to project root
    fs::write(&vision_path, final_content).map_err(|e| MetisError::ValidationFailed {
        message: format!("Failed to create vision document: {}", e),
    })?;

    Ok(())
}

/// Validate that a project name is safe for filesystem use
fn is_valid_project_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 255 {
        return false;
    }

    // Only allow alphanumeric characters, hyphens, underscores, and spaces
    name.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_is_valid_project_name() {
        assert!(is_valid_project_name("my-project"));
        assert!(is_valid_project_name("my_project"));
        assert!(is_valid_project_name("MyProject123"));
        assert!(is_valid_project_name("My Project"));

        assert!(!is_valid_project_name(""));
        assert!(!is_valid_project_name("my/project"));
        assert!(!is_valid_project_name("my\\project"));
        assert!(!is_valid_project_name("my<project>"));
        assert!(!is_valid_project_name("my|project"));

        // Test very long name
        let long_name = "a".repeat(256);
        assert!(!is_valid_project_name(&long_name));
    }

    #[tokio::test]
    async fn test_initialize_project_success() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let config = ProjectConfig {
            name: "Test Project".to_string(),
            description: Some("A test project for validation".to_string()),
            root_path: project_path.clone(),
        };

        let result = initialize_project(config).await;
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.project_path, project_path.join("metis"));
        assert_eq!(
            metadata.database_path,
            project_path.join("metis").join(".metis.db")
        );

        // Verify directory structure was created
        assert!(project_path.join("metis").join("strategies").exists());
        assert!(project_path.join("metis").join("decisions").exists());

        // Verify database was created
        assert!(project_path.join("metis").join(".metis.db").exists());

        // Verify vision document was created
        let vision_path = project_path.join("metis").join("vision.md");
        assert!(vision_path.exists());

        let vision_content = fs::read_to_string(vision_path).unwrap();
        assert!(vision_content.contains("Test Project Vision"));
        assert!(vision_content.contains("A test project for validation"));
    }

    #[tokio::test]
    async fn test_initialize_project_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create existing metis directory and .metis.db file
        fs::create_dir_all(project_path.join("metis")).unwrap();
        fs::write(project_path.join("metis").join(".metis.db"), "").unwrap();

        let config = ProjectConfig {
            name: "Test Project".to_string(),
            description: None,
            root_path: project_path.clone(),
        };

        // Should succeed because initialization is now idempotent
        let result = initialize_project(config).await;
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.project_path, project_path.join("metis"));
        assert_eq!(
            metadata.database_path,
            project_path.join("metis").join(".metis.db")
        );
    }

    #[tokio::test]
    async fn test_initialize_project_twice() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let config = ProjectConfig {
            name: "Test Project".to_string(),
            description: Some("A test project for double initialization".to_string()),
            root_path: project_path.clone(),
        };

        // First initialization should succeed
        let result1 = initialize_project(config.clone()).await;
        assert!(result1.is_ok());

        // Verify it was created properly
        assert!(project_path.join("metis").join(".metis.db").exists());
        assert!(project_path.join("metis").join("vision.md").exists());

        // Second initialization should succeed (idempotent)
        let result2 = initialize_project(config).await;
        assert!(result2.is_ok());

        let metadata2 = result2.unwrap();
        assert_eq!(metadata2.project_path, project_path.join("metis"));
        assert_eq!(
            metadata2.database_path,
            project_path.join("metis").join(".metis.db")
        );
    }

    #[tokio::test]
    async fn test_initialize_project_invalid_name() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let config = ProjectConfig {
            name: "invalid/name".to_string(), // Contains slash
            description: None,
            root_path: project_path.clone(),
        };

        let result = initialize_project(config).await;
        assert!(result.is_err());

        if let Err(MetisError::ValidationFailed { message }) = result {
            assert!(message.contains("Invalid project name"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[tokio::test]
    async fn test_initialize_project_nonexistent_directory() {
        let nonexistent_path = PathBuf::from("/nonexistent/directory");

        let config = ProjectConfig {
            name: "Test Project".to_string(),
            description: None,
            root_path: nonexistent_path,
        };

        let result = initialize_project(config).await;
        assert!(result.is_err());

        if let Err(MetisError::ValidationFailed { message }) = result {
            assert!(message.contains("Parent directory does not exist"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[tokio::test]
    async fn test_initialize_project_without_description() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let config = ProjectConfig {
            name: "Simple Project".to_string(),
            description: None,
            root_path: project_path.clone(),
        };

        let result = initialize_project(config).await;
        assert!(result.is_ok());

        // Verify vision document was created with default content
        let vision_path = project_path.join("metis").join("vision.md");
        assert!(vision_path.exists());

        let vision_content = fs::read_to_string(vision_path).unwrap();
        assert!(vision_content.contains("Simple Project Vision"));
        // Should contain the default placeholder text
        assert!(vision_content.contains("{Why this vision exists and what it aims to achieve}"));
    }

    #[test]
    fn test_create_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let result = create_directory_structure(project_path);
        assert!(result.is_ok());

        assert!(project_path.join("strategies").exists());
        assert!(project_path.join("decisions").exists());
        assert!(project_path.join("strategies").is_dir());
        assert!(project_path.join("decisions").is_dir());
    }
}
