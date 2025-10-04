use crate::domain::documents::initiative::Complexity;
use crate::domain::documents::strategy::RiskLevel;
use crate::domain::documents::traits::Document;
use crate::domain::documents::types::{DocumentId, DocumentType, Phase, Tag, ParentReference};
use crate::domain::configuration::FlightLevelConfig;
use crate::Result;
use crate::{Adr, Initiative, MetisError, Strategy, Task, Vision};
use std::fs;
use std::path::{Path, PathBuf};

/// Service for creating new documents with proper defaults and validation
pub struct DocumentCreationService {
    workspace_dir: PathBuf,
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
}

impl DocumentCreationService {
    /// Create a new document creation service for a workspace
    pub fn new<P: AsRef<Path>>(workspace_dir: P) -> Self {
        Self {
            workspace_dir: workspace_dir.as_ref().to_path_buf(),
        }
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
        })
    }

    /// Create a new strategy document
    pub async fn create_strategy(&self, config: DocumentCreationConfig) -> Result<CreationResult> {
        // Generate strategy ID from title
        let strategy_id = self.generate_id_from_title(&config.title);
        let strategy_dir = self.workspace_dir.join("strategies").join(&strategy_id);
        let file_path = strategy_dir.join("strategy.md");

        // Check if strategy already exists
        if file_path.exists() {
            return Err(MetisError::ValidationFailed {
                message: format!("Strategy with ID '{}' already exists", strategy_id),
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
        })
    }

    /// Create a new initiative document (legacy method)
    pub async fn create_initiative(
        &self,
        config: DocumentCreationConfig,
        strategy_id: &str,
    ) -> Result<CreationResult> {
        // Use full configuration for backward compatibility
        self.create_initiative_with_config(config, strategy_id, &FlightLevelConfig::full()).await
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
            return Err(MetisError::ValidationFailed {
                message: "Initiative creation is disabled in current flight level configuration".to_string(),
            });
        }

        // Generate initiative ID from title
        let initiative_id = self.generate_id_from_title(&config.title);
        
        // Determine directory structure using consistent NULL-based paths
        let (parent_ref, effective_strategy_id) = if flight_config.strategies_enabled {
            // Full configuration: use actual strategy_id
            if strategy_id == "NULL" {
                return Err(MetisError::ValidationFailed {
                    message: "Cannot create initiative with NULL strategy when strategies are enabled".to_string(),
                });
            }
            
            // Validate parent strategy exists
            let strategy_file = self
                .workspace_dir
                .join("strategies")
                .join(strategy_id)
                .join("strategy.md");
            if !strategy_file.exists() {
                return Err(MetisError::NotFound(format!(
                    "Parent strategy '{}' not found",
                    strategy_id
                )));
            }
            
            (ParentReference::Some(DocumentId::from(strategy_id)), strategy_id)
        } else {
            // Streamlined configuration: use NULL as strategy placeholder
            (ParentReference::Null, "NULL")
        };
        
        // Consistent directory structure: strategies/{strategy_id}/initiatives/{initiative_id}
        let initiative_dir = self
            .workspace_dir
            .join("strategies")
            .join(effective_strategy_id)
            .join("initiatives")
            .join(&initiative_id);

        let file_path = initiative_dir.join("initiative.md");

        // Check if initiative already exists
        if file_path.exists() {
            return Err(MetisError::ValidationFailed {
                message: format!("Initiative with ID '{}' already exists", initiative_id),
            });
        }

        // Create initiative with defaults
        let mut tags = vec![
            Tag::Label("initiative".to_string()),
            Tag::Phase(config.phase.unwrap_or(Phase::Discovery)),
        ];
        tags.extend(config.tags);

        // Use the parent reference from configuration, or explicit parent_id from config
        let parent_id = config.parent_id.map(ParentReference::Some).unwrap_or(parent_ref);

        let initiative = Initiative::new(
            config.title.clone(),
            parent_id.parent_id().cloned(), // Extract actual parent ID for document creation
            Vec::new(), // blocked_by
            tags,
            false,                                      // not archived
            config.complexity.unwrap_or(Complexity::M), // use config complexity or default to M
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
        self.create_task_with_config(config, strategy_id, initiative_id, &FlightLevelConfig::full()).await
    }

    /// Create a new task document with flight level configuration
    pub async fn create_task_with_config(
        &self,
        config: DocumentCreationConfig,
        strategy_id: &str,
        initiative_id: &str,
        flight_config: &FlightLevelConfig,
    ) -> Result<CreationResult> {
        // Generate task ID from title
        let task_id = self.generate_id_from_title(&config.title);
        
        // Determine directory structure using consistent NULL-based paths
        let (parent_ref, parent_title, effective_strategy_id, effective_initiative_id) = if flight_config.initiatives_enabled {
            // Initiatives are enabled, tasks go under initiatives
            if initiative_id == "NULL" {
                return Err(MetisError::ValidationFailed {
                    message: "Cannot create task with NULL initiative when initiatives are enabled".to_string(),
                });
            }

            let eff_strategy_id = if flight_config.strategies_enabled {
                // Full configuration: use actual strategy_id
                if strategy_id == "NULL" {
                    return Err(MetisError::ValidationFailed {
                        message: "Cannot create task with NULL strategy when strategies are enabled".to_string(),
                    });
                }
                strategy_id
            } else {
                // Streamlined configuration: use NULL as strategy placeholder
                "NULL"
            };

            // Validate parent initiative exists using the consistent path structure
            let initiative_file = self
                .workspace_dir
                .join("strategies")
                .join(eff_strategy_id)
                .join("initiatives")
                .join(initiative_id)
                .join("initiative.md");
            
            if !initiative_file.exists() {
                return Err(MetisError::NotFound(format!(
                    "Parent initiative '{}' not found",
                    initiative_id
                )));
            }

            (ParentReference::Some(DocumentId::from(initiative_id)), Some(initiative_id.to_string()), eff_strategy_id, initiative_id)
        } else {
            // Direct configuration: use NULL placeholders for both strategy and initiative
            (ParentReference::Null, None, "NULL", "NULL")
        };

        // Consistent directory structure: strategies/{strategy_id}/initiatives/{initiative_id}/tasks/{task_id}
        let task_dir = self
            .workspace_dir
            .join("strategies")
            .join(effective_strategy_id)
            .join("initiatives")
            .join(effective_initiative_id)
            .join("tasks");

        let file_path = task_dir.join(format!("{}.md", task_id));

        // Check if task already exists
        if file_path.exists() {
            return Err(MetisError::ValidationFailed {
                message: format!("Task with ID '{}' already exists", task_id),
            });
        }

        // Create task with defaults
        let mut tags = vec![
            Tag::Label("task".to_string()),
            Tag::Phase(config.phase.unwrap_or(Phase::Todo)),
        ];
        tags.extend(config.tags);

        // Use the parent reference from configuration, or explicit parent_id from config
        let parent_id = config.parent_id.map(ParentReference::Some).unwrap_or(parent_ref);

        let task = Task::new(
            config.title.clone(),
            parent_id.parent_id().cloned(), // Extract actual parent ID for document creation
            parent_title,                   // parent title for template
            Vec::new(),                     // blocked_by
            tags,
            false, // not archived
        )
        .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        // Create parent directory if needed
        if !task_dir.exists() {
            fs::create_dir_all(&task_dir)
                .map_err(|e| MetisError::FileSystem(e.to_string()))?;
        }

        // Write to file
        task.to_file(&file_path)
            .await
            .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        Ok(CreationResult {
            document_id: task.id(),
            document_type: DocumentType::Task,
            file_path,
        })
    }

    /// Create a new backlog item (task without parent)
    pub async fn create_backlog_item(&self, config: DocumentCreationConfig) -> Result<CreationResult> {
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

        let task = Task::new(
            config.title.clone(),
            None,                            // No parent for backlog items
            None,                            // No parent title for template
            Vec::new(),                      // blocked_by
            tags,
            false, // not archived
        )
        .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        // Create parent directory if needed
        if !backlog_dir.exists() {
            fs::create_dir_all(&backlog_dir)
                .map_err(|e| MetisError::FileSystem(e.to_string()))?;
        }

        // Write to file
        task.to_file(&file_path)
            .await
            .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;

        Ok(CreationResult {
            document_id: task.id(),
            document_type: DocumentType::Task,
            file_path,
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
        // Find the next ADR number
        let adr_number = self.get_next_adr_number()?;
        let adr_slug = self.generate_id_from_title(&config.title);
        let adr_filename = format!("{:03}-{}.md", adr_number, adr_slug);
        let adrs_dir = self.workspace_dir.join("adrs");
        let file_path = adrs_dir.join(&adr_filename);

        // Check if ADR already exists
        if file_path.exists() {
            return Err(MetisError::ValidationFailed {
                message: format!("ADR with filename '{}' already exists", adr_filename),
            });
        }

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
        let strategy_id = strategy_result.document_id.to_string();

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
        let service = DocumentCreationService::new(temp_dir.path());
        (service, temp_dir)
    }

    #[tokio::test]
    async fn test_create_initiative_full_configuration() {
        let (service, _temp) = setup_test_service_temp();
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
                &strategy_result.document_id.to_string(),
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
        assert!(result.file_path.to_string_lossy().contains("strategies/NULL"));
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
        assert!(result.unwrap_err().to_string().contains("Initiative creation is disabled"));
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
        assert!(result.file_path.to_string_lossy().contains("strategies/NULL/initiatives/NULL"));
    }
}
