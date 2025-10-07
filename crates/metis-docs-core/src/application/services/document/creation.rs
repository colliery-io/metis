use crate::dal::database::configuration_repository::ConfigurationRepository;
use crate::domain::configuration::FlightLevelConfig;
use crate::domain::documents::initiative::Complexity;
use crate::domain::documents::strategy::RiskLevel;
use crate::domain::documents::traits::Document;
use crate::domain::documents::types::{DocumentId, DocumentType, ParentReference, Phase, Tag};
use crate::Result;
use crate::{Adr, Database, Initiative, MetisError, Strategy, Task, Vision};
use diesel::{sqlite::SqliteConnection, Connection};
use std::fs;
use std::path::{Path, PathBuf};

/// Service for creating new documents with proper defaults and validation
pub struct DocumentCreationService {
    workspace_dir: PathBuf,
    db_path: PathBuf,
}

/// Configuration for creating a new document
#[derive(Debug, Clone)]
pub struct DocumentCreationConfig {
    pub title: String,
    pub description: Option<String>,
    pub parent_id: Option<DocumentId>,
    pub tags: Vec<Tag>,
    pub phase: Option<Phase>,
    pub complexity: Option<Complexity>,
    pub risk_level: Option<RiskLevel>,
}

/// Result of document creation
#[derive(Debug)]
pub struct CreationResult {
    pub document_id: DocumentId,
    pub document_type: DocumentType,
    pub file_path: PathBuf,
    pub short_code: String,
}

impl DocumentCreationService {
    /// Create a new document creation service for a workspace
    pub fn new<P: AsRef<Path>>(workspace_dir: P) -> Self {
        let workspace_path = workspace_dir.as_ref().to_path_buf();
        let db_path = workspace_path.join("metis.db");
        Self {
            workspace_dir: workspace_path,
            db_path,
        }
    }

    /// Generate a short code for a document type
    fn generate_short_code(&self, doc_type: &str) -> Result<String> {
        let mut config_repo = ConfigurationRepository::new(
            SqliteConnection::establish(&self.db_path.to_string_lossy()).map_err(|e| {
                MetisError::ConfigurationError(
                    crate::domain::configuration::ConfigurationError::InvalidValue(e.to_string()),
                )
            })?,
        );

        config_repo.generate_short_code(doc_type)
    }

    /// Create a new vision document
    pub async fn create_vision(&self, config: DocumentCreationConfig) -> Result<CreationResult> {
        // Vision documents go directly in the workspace root
        let file_path = self.workspace_dir.join("vision.md");

        // Check if vision already exists
        if file_path.exists() {
            return Err(MetisError::ValidationFailed {
                message: "Vision document already exists".to_string(),
            });
        }

        // Generate short code for vision
        let short_code = self.generate_short_code("vision")?;

        // Create vision with defaults
        let mut tags = vec![
            Tag::Label("vision".to_string()),
            Tag::Phase(config.phase.unwrap_or(Phase::Draft)),
        ];
        tags.extend(config.tags);

        let vision = Vision::new(
            config.title.clone(),
            tags,
            false, // not archived
            short_code.clone(),
        )
        .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        // Create parent directory if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| MetisError::FileSystem(e.to_string()))?;
        }

        // Write to file
        vision
            .to_file(&file_path)
            .await
            .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        Ok(CreationResult {
            document_id: vision.id(),
            document_type: DocumentType::Vision,
            file_path,
            short_code,
        })
    }

    /// Create a new strategy document
    pub async fn create_strategy(&self, config: DocumentCreationConfig) -> Result<CreationResult> {
        // Generate short code for strategy (used for both ID and file path)
        let short_code = self.generate_short_code("strategy")?;
        let strategy_dir = self.workspace_dir.join("strategies").join(&short_code);
        let file_path = strategy_dir.join("strategy.md");

        // Check if strategy already exists
        if file_path.exists() {
            return Err(MetisError::ValidationFailed {
                message: format!("Strategy with short code '{}' already exists", short_code),
            });
        }

        // Create strategy with defaults
        let mut tags = vec![
            Tag::Label("strategy".to_string()),
            Tag::Phase(config.phase.unwrap_or(Phase::Shaping)),
        ];
        tags.extend(config.tags);

        let strategy = Strategy::new(
            config.title.clone(),
            config.parent_id,
            Vec::new(), // blocked_by
            tags,
            false,                                          // not archived
            config.risk_level.unwrap_or(RiskLevel::Medium), // use config risk_level or default to Medium
            Vec::new(),                                     // stakeholders - empty by default
            short_code.clone(),
        )
        .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        // Create parent directory
        fs::create_dir_all(&strategy_dir).map_err(|e| MetisError::FileSystem(e.to_string()))?;

        // Write to file
        strategy
            .to_file(&file_path)
            .await
            .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        Ok(CreationResult {
            document_id: strategy.id(),
            document_type: DocumentType::Strategy,
            file_path,
            short_code,
        })
    }

    /// Create a new initiative document (legacy method)
    pub async fn create_initiative(
        &self,
        config: DocumentCreationConfig,
        strategy_id: &str,
    ) -> Result<CreationResult> {
        // Use full configuration for backward compatibility
        self.create_initiative_with_config(config, strategy_id, &FlightLevelConfig::full())
            .await
    }

    /// Create a new initiative document with flight level configuration
    pub async fn create_initiative_with_config(
        &self,
        config: DocumentCreationConfig,
        strategy_id: &str,
        flight_config: &FlightLevelConfig,
    ) -> Result<CreationResult> {
        // Validate that initiatives are enabled in this configuration
        if !flight_config.initiatives_enabled {
            let enabled_types: Vec<String> = flight_config
                .enabled_document_types()
                .iter()
                .map(|t| t.to_string())
                .collect();
            return Err(MetisError::ValidationFailed {
                message: format!(
                    "Initiative creation is disabled in current configuration ({} mode). Available document types: {}. To enable initiatives, use 'metis config set --preset full' or 'metis config set --initiatives true'",
                    flight_config.preset_name(),
                    enabled_types.join(", ")
                ),
            });
        }

        // Generate short code for initiative (used for both ID and file path)
        let short_code = self.generate_short_code("initiative")?;

        // Determine the strategy short code first (outside conditionals to avoid lifetime issues)
        let strategy_short_code = if flight_config.strategies_enabled && strategy_id != "NULL" {
            // Validate parent strategy exists by looking up its short code in database
            let db_path = self.workspace_dir.join("metis.db");
            let db = Database::new(db_path.to_str().unwrap())
                .map_err(|e| MetisError::FileSystem(format!("Database error: {}", e)))?;
            let mut repo = db
                .repository()
                .map_err(|e| MetisError::FileSystem(format!("Repository error: {}", e)))?;

            // Find the strategy by short code
            let strategy = repo
                .find_by_short_code(strategy_id)
                .map_err(|e| MetisError::FileSystem(format!("Database lookup error: {}", e)))?
                .ok_or_else(|| {
                    MetisError::NotFound(format!("Parent strategy '{}' not found", strategy_id))
                })?;

            // Use the short code to build the file path
            let strategy_file = self
                .workspace_dir
                .join("strategies")
                .join(&strategy.short_code)
                .join("strategy.md");
            if !strategy_file.exists() {
                return Err(MetisError::NotFound(format!(
                    "Parent strategy '{}' not found at expected path",
                    strategy_id
                )));
            }

            strategy.short_code
        } else {
            "NULL".to_string()
        };

        // Determine directory structure using consistent NULL-based paths
        let (parent_ref, effective_strategy_id) = if flight_config.strategies_enabled {
            // Full configuration: use actual strategy_id
            if strategy_id == "NULL" {
                return Err(MetisError::ValidationFailed {
                    message: format!(
                        "Cannot create initiative with NULL strategy when strategies are enabled in {} configuration. Provide a valid strategy_id",
                        flight_config.preset_name()
                    ),
                });
            }

            (
                ParentReference::Some(DocumentId::from(strategy_id)),
                strategy_short_code.as_str(),
            )
        } else {
            // Streamlined configuration: use NULL as strategy placeholder
            (ParentReference::Null, "NULL")
        };

        // Consistent directory structure: strategies/{strategy_short_code}/initiatives/{initiative_short_code}
        let initiative_dir = self
            .workspace_dir
            .join("strategies")
            .join(effective_strategy_id)
            .join("initiatives")
            .join(&short_code);

        let file_path = initiative_dir.join("initiative.md");

        // Check if initiative already exists
        if file_path.exists() {
            return Err(MetisError::ValidationFailed {
                message: format!("Initiative with short code '{}' already exists", short_code),
            });
        }

        // Create initiative with defaults
        let mut tags = vec![
            Tag::Label("initiative".to_string()),
            Tag::Phase(config.phase.unwrap_or(Phase::Discovery)),
        ];
        tags.extend(config.tags);

        // Use the parent reference from configuration, or explicit parent_id from config
        let parent_id = config
            .parent_id
            .map(ParentReference::Some)
            .unwrap_or(parent_ref);

        let initiative = Initiative::new(
            config.title.clone(),
            parent_id.parent_id().cloned(), // Extract actual parent ID for document creation
            Some(DocumentId::from(effective_strategy_id)), // strategy_id from configuration
            Vec::new(),                     // blocked_by
            tags,
            false,                                      // not archived
            config.complexity.unwrap_or(Complexity::M), // use config complexity or default to M
            short_code.clone(),
        )
        .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        // Create parent directory
        fs::create_dir_all(&initiative_dir).map_err(|e| MetisError::FileSystem(e.to_string()))?;

        // Write to file
        initiative
            .to_file(&file_path)
            .await
            .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        Ok(CreationResult {
            document_id: initiative.id(),
            document_type: DocumentType::Initiative,
            file_path,
            short_code,
        })
    }

    /// Create a new task document (legacy method)
    pub async fn create_task(
        &self,
        config: DocumentCreationConfig,
        strategy_id: &str,
        initiative_id: &str,
    ) -> Result<CreationResult> {
        // Use full configuration for backward compatibility
        self.create_task_with_config(
            config,
            strategy_id,
            initiative_id,
            &FlightLevelConfig::full(),
        )
        .await
    }

    /// Create a new task document with flight level configuration
    pub async fn create_task_with_config(
        &self,
        config: DocumentCreationConfig,
        strategy_id: &str,
        initiative_id: &str,
        flight_config: &FlightLevelConfig,
    ) -> Result<CreationResult> {
        // Generate short code for task (used for both ID and file path)
        let short_code = self.generate_short_code("task")?;

        // Resolve short codes first (outside conditionals to avoid lifetime issues)
        let (strategy_short_code, initiative_short_code) = if flight_config.initiatives_enabled
            && initiative_id != "NULL"
        {
            // Validate parent initiative exists by looking up its short codes in database
            let db_path = self.workspace_dir.join("metis.db");
            let db = Database::new(db_path.to_str().unwrap())
                .map_err(|e| MetisError::FileSystem(format!("Database error: {}", e)))?;
            let mut repo = db
                .repository()
                .map_err(|e| MetisError::FileSystem(format!("Repository error: {}", e)))?;

            // Find the initiative by short code
            let initiative = repo
                .find_by_short_code(initiative_id)
                .map_err(|e| MetisError::FileSystem(format!("Database lookup error: {}", e)))?
                .ok_or_else(|| {
                    MetisError::NotFound(format!("Parent initiative '{}' not found", initiative_id))
                })?;

            // If strategies are enabled, also get the strategy short code
            let strategy_short_code = if flight_config.strategies_enabled && strategy_id != "NULL" {
                let strategy = repo
                    .find_by_short_code(strategy_id)
                    .map_err(|e| MetisError::FileSystem(format!("Database lookup error: {}", e)))?
                    .ok_or_else(|| {
                        MetisError::NotFound(format!("Parent strategy '{}' not found", strategy_id))
                    })?;
                strategy.short_code
            } else {
                "NULL".to_string()
            };

            // Use the short codes to build the file path
            let initiative_file = self
                .workspace_dir
                .join("strategies")
                .join(&strategy_short_code)
                .join("initiatives")
                .join(&initiative.short_code)
                .join("initiative.md");

            if !initiative_file.exists() {
                return Err(MetisError::NotFound(format!(
                    "Parent initiative '{}' not found at expected path",
                    initiative_id
                )));
            }

            (strategy_short_code, initiative.short_code)
        } else {
            ("NULL".to_string(), "NULL".to_string())
        };

        // Determine directory structure using consistent NULL-based paths
        let (parent_ref, parent_title, effective_strategy_id, effective_initiative_id) =
            if flight_config.initiatives_enabled {
                // Initiatives are enabled, tasks go under initiatives
                if initiative_id == "NULL" {
                    return Err(MetisError::ValidationFailed {
                    message: format!(
                        "Cannot create task with NULL initiative when initiatives are enabled in {} configuration. Provide a valid initiative_id or create the task as a backlog item",
                        flight_config.preset_name()
                    ),
                });
                }

                // Validation was done earlier, use the pre-computed short codes
                if flight_config.strategies_enabled && strategy_id == "NULL" {
                    return Err(MetisError::ValidationFailed {
                    message: format!(
                        "Cannot create task with NULL strategy when strategies are enabled in {} configuration. Provide a valid strategy_id or create the task as a backlog item",
                        flight_config.preset_name()
                    ),
                });
                }

                (
                    ParentReference::Some(DocumentId::from(initiative_id)),
                    Some(initiative_id.to_string()),
                    strategy_short_code.as_str(),
                    initiative_short_code.as_str(),
                )
            } else {
                // Direct configuration: use NULL placeholders for both strategy and initiative
                (ParentReference::Null, None, "NULL", "NULL")
            };

        // Consistent directory structure: strategies/{strategy_short_code}/initiatives/{initiative_short_code}/tasks/{task_short_code}
        let task_dir = self
            .workspace_dir
            .join("strategies")
            .join(effective_strategy_id)
            .join("initiatives")
            .join(effective_initiative_id)
            .join("tasks");

        let file_path = task_dir.join(format!("{}.md", short_code));

        // Check if task already exists
        if file_path.exists() {
            return Err(MetisError::ValidationFailed {
                message: format!("Task with short code '{}' already exists", short_code),
            });
        }

        // Create task with defaults
        let mut tags = vec![
            Tag::Label("task".to_string()),
            Tag::Phase(config.phase.unwrap_or(Phase::Todo)),
        ];
        tags.extend(config.tags);

        // Use the parent reference from configuration, or explicit parent_id from config
        let parent_id = config
            .parent_id
            .map(ParentReference::Some)
            .unwrap_or(parent_ref);

        let task = Task::new(
            config.title.clone(),
            parent_id.parent_id().cloned(), // Extract actual parent ID for document creation
            parent_title,                   // parent title for template
            if effective_strategy_id == "NULL" {
                None
            } else {
                Some(DocumentId::from(effective_strategy_id))
            },
            if effective_initiative_id == "NULL" {
                None
            } else {
                Some(DocumentId::from(effective_initiative_id))
            },
            Vec::new(), // blocked_by
            tags,
            false, // not archived
            short_code.clone(),
        )
        .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        // Create parent directory if needed
        if !task_dir.exists() {
            fs::create_dir_all(&task_dir).map_err(|e| MetisError::FileSystem(e.to_string()))?;
        }

        // Write to file
        task.to_file(&file_path)
            .await
            .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        Ok(CreationResult {
            document_id: task.id(),
            document_type: DocumentType::Task,
            file_path,
            short_code,
        })
    }

    /// Create a new backlog item (task without parent)
    pub async fn create_backlog_item(
        &self,
        config: DocumentCreationConfig,
    ) -> Result<CreationResult> {
        // Generate task ID from title
        let task_id = self.generate_id_from_title(&config.title);

        // Create backlog directory structure based on tags
        let backlog_dir = self.determine_backlog_directory(&config.tags);
        let file_path = backlog_dir.join(format!("{}.md", task_id));

        // Check if backlog item already exists
        if file_path.exists() {
            return Err(MetisError::ValidationFailed {
                message: format!("Backlog item with ID '{}' already exists", task_id),
            });
        }

        // Create backlog item with defaults - no parent, Backlog phase
        let mut tags = vec![
            Tag::Label("task".to_string()),
            Tag::Phase(config.phase.unwrap_or(Phase::Backlog)),
        ];
        tags.extend(config.tags);

        // Generate short code for task
        let short_code = self.generate_short_code("task")?;

        let task = Task::new(
            config.title.clone(),
            None,       // No parent for backlog items
            None,       // No parent title for template
            None,       // No strategy for backlog items
            None,       // No initiative for backlog items
            Vec::new(), // blocked_by
            tags,
            false, // not archived
            short_code.clone(),
        )
        .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        // Create parent directory if needed
        if !backlog_dir.exists() {
            fs::create_dir_all(&backlog_dir).map_err(|e| MetisError::FileSystem(e.to_string()))?;
        }

        // Write to file
        task.to_file(&file_path)
            .await
            .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        Ok(CreationResult {
            document_id: task.id(),
            document_type: DocumentType::Task,
            file_path,
            short_code,
        })
    }

    /// Determine the backlog directory based on tags
    fn determine_backlog_directory(&self, tags: &[Tag]) -> PathBuf {
        let base_backlog_dir = self.workspace_dir.join("backlog");

        // Check for type tags to determine subdirectory
        for tag in tags {
            if let Tag::Label(label) = tag {
                match label.as_str() {
                    "bug" => return base_backlog_dir.join("bugs"),
                    "feature" => return base_backlog_dir.join("features"),
                    "tech-debt" => return base_backlog_dir.join("tech-debt"),
                    _ => {}
                }
            }
        }

        // Default to general backlog if no specific type found
        base_backlog_dir
    }

    /// Create a new ADR document
    pub async fn create_adr(&self, config: DocumentCreationConfig) -> Result<CreationResult> {
        // Generate short code for ADR (used for both ID and file path)
        let short_code = self.generate_short_code("adr")?;
        let adr_filename = format!("{}.md", short_code);
        let adrs_dir = self.workspace_dir.join("adrs");
        let file_path = adrs_dir.join(&adr_filename);

        // Check if ADR already exists
        if file_path.exists() {
            return Err(MetisError::ValidationFailed {
                message: format!("ADR with short code '{}' already exists", short_code),
            });
        }

        // Find the next ADR number for the document content (still needed for ADR numbering)
        let adr_number = self.get_next_adr_number()?;

        // Create ADR with defaults
        let mut tags = vec![
            Tag::Label("adr".to_string()),
            Tag::Phase(config.phase.unwrap_or(Phase::Draft)),
        ];
        tags.extend(config.tags);

        let adr = Adr::new(
            adr_number,
            config.title.clone(),
            String::new(), // decision_maker - will be set when transitioning to decided
            None,          // decision_date - will be set when transitioning to decided
            config.parent_id,
            tags,
            false, // not archived
            short_code.clone(),
        )
        .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        // Create parent directory
        fs::create_dir_all(&adrs_dir).map_err(|e| MetisError::FileSystem(e.to_string()))?;

        // Write to file
        adr.to_file(&file_path)
            .await
            .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        Ok(CreationResult {
            document_id: adr.id(),
            document_type: DocumentType::Adr,
            file_path,
            short_code,
        })
    }

    /// Generate a slugified ID from a title (same as DocumentId::title_to_slug)
    fn generate_id_from_title(&self, title: &str) -> String {
        use crate::domain::documents::types::DocumentId;
        DocumentId::title_to_slug(title)
    }

    /// Get the next ADR number by examining existing ADRs
    fn get_next_adr_number(&self) -> Result<u32> {
        let adrs_dir = self.workspace_dir.join("adrs");

        if !adrs_dir.exists() {
            return Ok(1);
        }

        let mut max_number = 0;
        for entry in fs::read_dir(&adrs_dir).map_err(|e| MetisError::FileSystem(e.to_string()))? {
            let entry = entry.map_err(|e| MetisError::FileSystem(e.to_string()))?;
            let filename = entry.file_name().to_string_lossy().to_string();

            if filename.ends_with(".md") {
                // Parse number from filename like "001-title.md"
                if let Some(number_str) = filename.split('-').next() {
                    if let Ok(number) = number_str.parse::<u32>() {
                        max_number = max_number.max(number);
                    }
                }
            }
        }

        Ok(max_number + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_vision_document() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");
        fs::create_dir_all(&workspace_dir).unwrap();

        // Create and initialize database with proper schema
        let db_path = workspace_dir.join("metis.db");
        let _db = crate::Database::new(&db_path.to_string_lossy()).unwrap();

        // Set up project prefix in configuration
        let mut config_repo = ConfigurationRepository::new(
            SqliteConnection::establish(&db_path.to_string_lossy()).unwrap(),
        );
        config_repo.set_project_prefix("TEST").unwrap();

        let service = DocumentCreationService::new(&workspace_dir);
        let config = DocumentCreationConfig {
            title: "Test Vision".to_string(),
            description: Some("A test vision document".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };

        let result = service.create_vision(config).await.unwrap();

        assert_eq!(result.document_type, DocumentType::Vision);
        assert!(result.file_path.exists());

        // Verify we can read it back
        let vision = Vision::from_file(&result.file_path).await.unwrap();
        assert_eq!(vision.title(), "Test Vision");
    }

    #[tokio::test]
    async fn test_create_strategy_document() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");
        fs::create_dir_all(&workspace_dir).unwrap();

        // Create and initialize database with proper schema
        let db_path = workspace_dir.join("metis.db");
        let _db = crate::Database::new(&db_path.to_string_lossy()).unwrap();

        // Set up project prefix in configuration
        let mut config_repo = ConfigurationRepository::new(
            SqliteConnection::establish(&db_path.to_string_lossy()).unwrap(),
        );
        config_repo.set_project_prefix("TEST").unwrap();

        let service = DocumentCreationService::new(&workspace_dir);
        let config = DocumentCreationConfig {
            title: "Test Strategy".to_string(),
            description: Some("A test strategy document".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };

        let result = service.create_strategy(config).await.unwrap();

        assert_eq!(result.document_type, DocumentType::Strategy);
        assert!(result.file_path.exists());

        // Verify we can read it back
        let strategy = Strategy::from_file(&result.file_path).await.unwrap();
        assert_eq!(strategy.title(), "Test Strategy");
    }

    #[tokio::test]
    async fn test_create_initiative_document() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");
        fs::create_dir_all(&workspace_dir).unwrap();

        // Create and initialize database with proper schema
        let db_path = workspace_dir.join("metis.db");
        let _db = crate::Database::new(&db_path.to_string_lossy()).unwrap();

        // Set up project prefix in configuration
        let mut config_repo = ConfigurationRepository::new(
            SqliteConnection::establish(&db_path.to_string_lossy()).unwrap(),
        );
        config_repo.set_project_prefix("TEST").unwrap();

        let service = DocumentCreationService::new(&workspace_dir);

        // First create a parent strategy
        let strategy_config = DocumentCreationConfig {
            title: "Parent Strategy".to_string(),
            description: Some("A parent strategy".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };
        let strategy_result = service.create_strategy(strategy_config).await.unwrap();
        let strategy_id = strategy_result.short_code.clone();

        // Sync the strategy to database so it can be found by the initiative creation
        let db = crate::Database::new(&db_path.to_string_lossy()).unwrap();
        let mut db_service =
            crate::application::services::DatabaseService::new(db.repository().unwrap());
        let mut sync_service = crate::application::services::SyncService::new(&mut db_service);
        sync_service
            .import_from_file(&strategy_result.file_path)
            .await
            .unwrap();

        // Now create an initiative
        let initiative_config = DocumentCreationConfig {
            title: "Test Initiative".to_string(),
            description: Some("A test initiative document".to_string()),
            parent_id: Some(strategy_result.document_id),
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };

        let result = service
            .create_initiative(initiative_config, &strategy_id)
            .await
            .unwrap();

        assert_eq!(result.document_type, DocumentType::Initiative);
        assert!(result.file_path.exists());

        // Verify we can read it back
        let initiative = Initiative::from_file(&result.file_path).await.unwrap();
        assert_eq!(initiative.title(), "Test Initiative");
    }

    #[tokio::test]
    async fn test_generate_id_from_title() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");

        let service = DocumentCreationService::new(&workspace_dir);

        assert_eq!(
            service.generate_id_from_title("Test Strategy"),
            "test-strategy"
        );
        assert_eq!(
            service.generate_id_from_title("My Complex Title!"),
            "my-complex-title"
        );
        assert_eq!(
            service.generate_id_from_title("Multiple   Spaces"),
            "multiple-spaces"
        );
    }

    #[tokio::test]
    async fn test_get_next_adr_number() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");
        let adrs_dir = workspace_dir.join("adrs");
        fs::create_dir_all(&adrs_dir).unwrap();

        let service = DocumentCreationService::new(&workspace_dir);

        // Should start at 1 when no ADRs exist
        assert_eq!(service.get_next_adr_number().unwrap(), 1);

        // Create some ADR files
        fs::write(adrs_dir.join("001-first-adr.md"), "content").unwrap();
        fs::write(adrs_dir.join("002-second-adr.md"), "content").unwrap();

        // Should return 3 as next number
        assert_eq!(service.get_next_adr_number().unwrap(), 3);
    }

    // Flexible flight levels tests

    fn setup_test_service_temp() -> (DocumentCreationService, tempfile::TempDir) {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let workspace_dir = temp_dir.path().join(".metis");
        fs::create_dir_all(&workspace_dir).unwrap();

        // Create and initialize database with proper schema
        let db_path = workspace_dir.join("metis.db");
        let _db = crate::Database::new(&db_path.to_string_lossy()).unwrap();

        // Set up project prefix in configuration
        let mut config_repo = ConfigurationRepository::new(
            SqliteConnection::establish(&db_path.to_string_lossy()).unwrap(),
        );
        config_repo.set_project_prefix("TEST").unwrap();

        let service = DocumentCreationService::new(&workspace_dir);
        (service, temp_dir)
    }

    #[tokio::test]
    async fn test_create_initiative_full_configuration() {
        let (service, temp) = setup_test_service_temp();
        let flight_config = FlightLevelConfig::full();

        // First create a strategy
        let strategy_config = DocumentCreationConfig {
            title: "Test Strategy".to_string(),
            description: None,
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };

        let strategy_result = service.create_strategy(strategy_config).await.unwrap();

        // Sync the strategy to database so it can be found by the initiative creation
        let db_path = temp.path().join(".metis/metis.db");
        let db = crate::Database::new(&db_path.to_string_lossy()).unwrap();
        let mut db_service =
            crate::application::services::DatabaseService::new(db.repository().unwrap());
        let mut sync_service = crate::application::services::SyncService::new(&mut db_service);
        sync_service
            .import_from_file(&strategy_result.file_path)
            .await
            .unwrap();

        // Now create an initiative under the strategy
        let initiative_config = DocumentCreationConfig {
            title: "Test Initiative".to_string(),
            description: None,
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };

        let result = service
            .create_initiative_with_config(
                initiative_config,
                &strategy_result.short_code,
                &flight_config,
            )
            .await
            .unwrap();

        assert_eq!(result.document_type, DocumentType::Initiative);
        assert!(result.file_path.exists());

        // Verify the path structure for full configuration
        assert!(result.file_path.to_string_lossy().contains("strategies"));
        assert!(result.file_path.to_string_lossy().contains("initiatives"));
    }

    #[tokio::test]
    async fn test_create_initiative_streamlined_configuration() {
        let (service, _temp) = setup_test_service_temp();
        let flight_config = FlightLevelConfig::streamlined();

        let initiative_config = DocumentCreationConfig {
            title: "Test Initiative".to_string(),
            description: None,
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };

        // In streamlined config, we pass "NULL" as strategy_id
        let result = service
            .create_initiative_with_config(initiative_config, "NULL", &flight_config)
            .await
            .unwrap();

        assert_eq!(result.document_type, DocumentType::Initiative);
        assert!(result.file_path.exists());

        // Verify the NULL-based path structure for streamlined configuration
        assert!(result.file_path.to_string_lossy().contains("initiatives"));
        assert!(result
            .file_path
            .to_string_lossy()
            .contains("strategies/NULL"));
    }

    #[tokio::test]
    async fn test_create_initiative_disabled_in_direct_configuration() {
        let (service, _temp) = setup_test_service_temp();
        let flight_config = FlightLevelConfig::direct();

        let initiative_config = DocumentCreationConfig {
            title: "Test Initiative".to_string(),
            description: None,
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };

        // In direct config, initiatives are disabled
        let result = service
            .create_initiative_with_config(initiative_config, "NULL", &flight_config)
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Initiative creation is disabled"));
    }

    #[tokio::test]
    async fn test_create_task_direct_configuration() {
        let (service, _temp) = setup_test_service_temp();
        let flight_config = FlightLevelConfig::direct();

        let task_config = DocumentCreationConfig {
            title: "Test Task".to_string(),
            description: None,
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };

        // In direct config, tasks go directly under workspace
        let result = service
            .create_task_with_config(task_config, "NULL", "NULL", &flight_config)
            .await
            .unwrap();

        assert_eq!(result.document_type, DocumentType::Task);
        assert!(result.file_path.exists());

        // Verify the NULL-based path structure for direct configuration
        assert!(result.file_path.to_string_lossy().contains("tasks"));
        assert!(result
            .file_path
            .to_string_lossy()
            .contains("strategies/NULL/initiatives/NULL"));
    }
}
