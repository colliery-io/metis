use crate::application::services::document::DocumentDiscoveryService;
use crate::domain::documents::traits::Document;
use crate::domain::documents::types::{DocumentType, Phase};
use crate::Result;
use crate::{Adr, Initiative, MetisError, Strategy, Task, Vision};
use std::path::{Path, PathBuf};

/// Service for managing document phase transitions
pub struct PhaseTransitionService {
    discovery_service: DocumentDiscoveryService,
}

/// Result of a phase transition
#[derive(Debug)]
pub struct TransitionResult {
    pub document_id: String,
    pub document_type: DocumentType,
    pub from_phase: Phase,
    pub to_phase: Phase,
    pub file_path: PathBuf,
}

impl PhaseTransitionService {
    /// Create a new phase transition service for a workspace
    pub fn new<P: AsRef<Path>>(workspace_dir: P) -> Self {
        let workspace_dir = workspace_dir.as_ref().to_path_buf();
        let discovery_service = DocumentDiscoveryService::new(&workspace_dir);

        Self { discovery_service }
    }

    /// Transition a document to a specific phase
    pub async fn transition_document(
        &self,
        document_id: &str,
        target_phase: Phase,
    ) -> Result<TransitionResult> {
        // Find the document - try short code first, then document ID
        let discovery_result = match self
            .discovery_service
            .find_document_by_short_code(document_id)
            .await
        {
            Ok(result) => result,
            Err(_) => {
                self.discovery_service
                    .find_document_by_id(document_id)
                    .await?
            }
        };

        // Load the document and get current phase
        let current_phase = self
            .get_current_phase(&discovery_result.file_path, discovery_result.document_type)
            .await?;

        // Validate the transition
        self.validate_transition(discovery_result.document_type, current_phase, target_phase)?;

        // Perform the transition
        self.perform_transition(
            &discovery_result.file_path,
            discovery_result.document_type,
            target_phase,
        )
        .await?;

        Ok(TransitionResult {
            document_id: document_id.to_string(),
            document_type: discovery_result.document_type,
            from_phase: current_phase,
            to_phase: target_phase,
            file_path: discovery_result.file_path,
        })
    }

    /// Transition a document to the next phase in its natural sequence
    pub async fn transition_to_next_phase(&self, document_id: &str) -> Result<TransitionResult> {
        // Find the document - try short code first, then document ID
        let discovery_result = match self
            .discovery_service
            .find_document_by_short_code(document_id)
            .await
        {
            Ok(result) => result,
            Err(_) => {
                self.discovery_service
                    .find_document_by_id(document_id)
                    .await?
            }
        };

        // Load the document and get current phase
        let current_phase = self
            .get_current_phase(&discovery_result.file_path, discovery_result.document_type)
            .await?;

        // Determine next phase
        let next_phase = self.get_next_phase(discovery_result.document_type, current_phase)?;

        // Perform the transition
        self.perform_transition(
            &discovery_result.file_path,
            discovery_result.document_type,
            next_phase,
        )
        .await?;

        Ok(TransitionResult {
            document_id: document_id.to_string(),
            document_type: discovery_result.document_type,
            from_phase: current_phase,
            to_phase: next_phase,
            file_path: discovery_result.file_path,
        })
    }

    /// Get the current phase of a document
    async fn get_current_phase(&self, file_path: &Path, doc_type: DocumentType) -> Result<Phase> {
        match doc_type {
            DocumentType::Vision => {
                let vision = Vision::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(vision.phase()?)
            }
            DocumentType::Strategy => {
                let strategy = Strategy::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(strategy.phase()?)
            }
            DocumentType::Initiative => {
                let initiative = Initiative::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(initiative.phase()?)
            }
            DocumentType::Task => {
                let task = Task::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(task.phase()?)
            }
            DocumentType::Adr => {
                let adr = Adr::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(adr.phase()?)
            }
        }
    }

    /// Perform the actual phase transition
    async fn perform_transition(
        &self,
        file_path: &Path,
        doc_type: DocumentType,
        target_phase: Phase,
    ) -> Result<()> {
        match doc_type {
            DocumentType::Vision => {
                let mut vision = Vision::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                vision.transition_phase(Some(target_phase)).map_err(|_e| {
                    MetisError::InvalidPhaseTransition {
                        from: vision.phase().unwrap_or(Phase::Draft).to_string(),
                        to: target_phase.to_string(),
                        doc_type: "vision".to_string(),
                    }
                })?;
                vision
                    .to_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
            DocumentType::Strategy => {
                let mut strategy = Strategy::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                strategy
                    .transition_phase(Some(target_phase))
                    .map_err(|_e| MetisError::InvalidPhaseTransition {
                        from: strategy.phase().unwrap_or(Phase::Shaping).to_string(),
                        to: target_phase.to_string(),
                        doc_type: "strategy".to_string(),
                    })?;
                strategy
                    .to_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
            DocumentType::Initiative => {
                let mut initiative = Initiative::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                initiative
                    .transition_phase(Some(target_phase))
                    .map_err(|_e| MetisError::InvalidPhaseTransition {
                        from: initiative.phase().unwrap_or(Phase::Discovery).to_string(),
                        to: target_phase.to_string(),
                        doc_type: "initiative".to_string(),
                    })?;
                initiative
                    .to_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
            DocumentType::Task => {
                let mut task = Task::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                task.transition_phase(Some(target_phase)).map_err(|_e| {
                    MetisError::InvalidPhaseTransition {
                        from: task.phase().unwrap_or(Phase::Todo).to_string(),
                        to: target_phase.to_string(),
                        doc_type: "task".to_string(),
                    }
                })?;
                task.to_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
            DocumentType::Adr => {
                let mut adr = Adr::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                adr.transition_phase(Some(target_phase)).map_err(|_e| {
                    MetisError::InvalidPhaseTransition {
                        from: adr.phase().unwrap_or(Phase::Draft).to_string(),
                        to: target_phase.to_string(),
                        doc_type: "adr".to_string(),
                    }
                })?;
                adr.to_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Validate that a phase transition is allowed
    fn validate_transition(
        &self,
        doc_type: DocumentType,
        from_phase: Phase,
        to_phase: Phase,
    ) -> Result<()> {
        let valid_transitions = self.get_valid_transitions(doc_type, from_phase);

        if !valid_transitions.contains(&to_phase) {
            return Err(MetisError::InvalidPhaseTransition {
                from: from_phase.to_string(),
                to: to_phase.to_string(),
                doc_type: doc_type.to_string(),
            });
        }

        Ok(())
    }

    /// Get valid transitions from a given phase for a document type
    fn get_valid_transitions(&self, doc_type: DocumentType, from_phase: Phase) -> Vec<Phase> {
        match doc_type {
            DocumentType::Vision => match from_phase {
                Phase::Draft => vec![Phase::Review],
                Phase::Review => vec![Phase::Draft, Phase::Published],
                Phase::Published => vec![Phase::Review],
                _ => vec![],
            },
            DocumentType::Strategy => match from_phase {
                Phase::Shaping => vec![Phase::Design],
                Phase::Design => vec![Phase::Shaping, Phase::Ready],
                Phase::Ready => vec![Phase::Design, Phase::Active],
                Phase::Active => vec![Phase::Ready, Phase::Completed],
                Phase::Completed => vec![],
                _ => vec![],
            },
            DocumentType::Initiative => match from_phase {
                Phase::Discovery => vec![Phase::Design],
                Phase::Design => vec![Phase::Discovery, Phase::Ready],
                Phase::Ready => vec![Phase::Design, Phase::Decompose],
                Phase::Decompose => vec![Phase::Ready, Phase::Active],
                Phase::Active => vec![Phase::Decompose, Phase::Completed],
                Phase::Completed => vec![],
                _ => vec![],
            },
            DocumentType::Task => match from_phase {
                Phase::Backlog => vec![Phase::Todo],
                Phase::Todo => vec![Phase::Active, Phase::Blocked],
                Phase::Active => vec![Phase::Todo, Phase::Completed, Phase::Blocked],
                Phase::Blocked => vec![Phase::Todo, Phase::Active],
                Phase::Completed => vec![],
                _ => vec![],
            },
            DocumentType::Adr => match from_phase {
                Phase::Draft => vec![Phase::Discussion],
                Phase::Discussion => vec![Phase::Draft, Phase::Decided],
                Phase::Decided => vec![],
                _ => vec![],
            },
        }
    }

    /// Get the next phase in the natural sequence for a document type
    fn get_next_phase(&self, doc_type: DocumentType, current_phase: Phase) -> Result<Phase> {
        match doc_type {
            DocumentType::Vision => match current_phase {
                Phase::Draft => Ok(Phase::Review),
                Phase::Review => Ok(Phase::Published),
                Phase::Published => Err(MetisError::InvalidPhaseTransition {
                    from: current_phase.to_string(),
                    to: "none".to_string(),
                    doc_type: "vision".to_string(),
                }),
                _ => Err(MetisError::InvalidPhaseTransition {
                    from: current_phase.to_string(),
                    to: "unknown".to_string(),
                    doc_type: "vision".to_string(),
                }),
            },
            DocumentType::Strategy => match current_phase {
                Phase::Shaping => Ok(Phase::Design),
                Phase::Design => Ok(Phase::Ready),
                Phase::Ready => Ok(Phase::Active),
                Phase::Active => Ok(Phase::Completed),
                Phase::Completed => Err(MetisError::InvalidPhaseTransition {
                    from: current_phase.to_string(),
                    to: "none".to_string(),
                    doc_type: "strategy".to_string(),
                }),
                _ => Err(MetisError::InvalidPhaseTransition {
                    from: current_phase.to_string(),
                    to: "unknown".to_string(),
                    doc_type: "strategy".to_string(),
                }),
            },
            DocumentType::Initiative => match current_phase {
                Phase::Discovery => Ok(Phase::Design),
                Phase::Design => Ok(Phase::Ready),
                Phase::Ready => Ok(Phase::Decompose),
                Phase::Decompose => Ok(Phase::Active),
                Phase::Active => Ok(Phase::Completed),
                Phase::Completed => Err(MetisError::InvalidPhaseTransition {
                    from: current_phase.to_string(),
                    to: "none".to_string(),
                    doc_type: "initiative".to_string(),
                }),
                _ => Err(MetisError::InvalidPhaseTransition {
                    from: current_phase.to_string(),
                    to: "unknown".to_string(),
                    doc_type: "initiative".to_string(),
                }),
            },
            DocumentType::Task => {
                match current_phase {
                    Phase::Backlog => Ok(Phase::Todo), // Transition from backlog to todo
                    Phase::Todo => Ok(Phase::Active),
                    Phase::Active => Ok(Phase::Completed),
                    Phase::Blocked => Ok(Phase::Active), // Unblock by default
                    Phase::Completed => Err(MetisError::InvalidPhaseTransition {
                        from: current_phase.to_string(),
                        to: "none".to_string(),
                        doc_type: "task".to_string(),
                    }),
                    _ => Err(MetisError::InvalidPhaseTransition {
                        from: current_phase.to_string(),
                        to: "unknown".to_string(),
                        doc_type: "task".to_string(),
                    }),
                }
            }
            DocumentType::Adr => match current_phase {
                Phase::Draft => Ok(Phase::Discussion),
                Phase::Discussion => Ok(Phase::Decided),
                Phase::Decided => Err(MetisError::InvalidPhaseTransition {
                    from: current_phase.to_string(),
                    to: "none".to_string(),
                    doc_type: "adr".to_string(),
                }),
                _ => Err(MetisError::InvalidPhaseTransition {
                    from: current_phase.to_string(),
                    to: "unknown".to_string(),
                    doc_type: "adr".to_string(),
                }),
            },
        }
    }

    /// Check if a phase transition is valid without performing it
    pub fn is_valid_transition(
        &self,
        doc_type: DocumentType,
        from_phase: Phase,
        to_phase: Phase,
    ) -> bool {
        self.validate_transition(doc_type, from_phase, to_phase)
            .is_ok()
    }

    /// Get all valid transitions for a document type and phase
    pub fn get_valid_transitions_for(
        &self,
        doc_type: DocumentType,
        from_phase: Phase,
    ) -> Vec<Phase> {
        self.get_valid_transitions(doc_type, from_phase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::services::document::creation::DocumentCreationConfig;
    use crate::application::services::document::DocumentCreationService;
    use crate::dal::Database;
    use diesel::Connection;
    
    use std::path::PathBuf;
    use tempfile::tempdir;

    async fn setup_test_workspace() -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");
        std::fs::create_dir_all(&workspace_dir).unwrap();

        // Initialize database with configuration
        let db_path = workspace_dir.join("metis.db");
        let _db = Database::new(&db_path.to_string_lossy()).unwrap();
        let mut config_repo =
            crate::dal::database::configuration_repository::ConfigurationRepository::new(
                diesel::sqlite::SqliteConnection::establish(&db_path.to_string_lossy()).unwrap(),
            );
        config_repo.set_project_prefix("TEST").unwrap();

        (temp_dir, workspace_dir)
    }

    #[tokio::test]
    async fn test_transition_vision_to_next_phase() {
        let (_temp_dir, workspace_dir) = setup_test_workspace().await;

        // Create a vision document
        let creation_service = DocumentCreationService::new(&workspace_dir);
        let config = DocumentCreationConfig {
            title: "Test Vision".to_string(),
            description: Some("A test vision".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None, // Should default to Draft
            complexity: None,
            risk_level: None,
        };
        let creation_result = creation_service.create_vision(config).await.unwrap();

        // Transition to next phase
        let transition_service = PhaseTransitionService::new(&workspace_dir);
        let transition_result = transition_service
            .transition_to_next_phase(&creation_result.document_id.to_string())
            .await
            .unwrap();

        assert_eq!(transition_result.from_phase, Phase::Draft);
        assert_eq!(transition_result.to_phase, Phase::Review);
        assert_eq!(transition_result.document_type, DocumentType::Vision);
    }

    #[tokio::test]
    async fn test_transition_strategy_through_phases() {
        let (_temp_dir, workspace_dir) = setup_test_workspace().await;

        // Create a strategy document
        let creation_service = DocumentCreationService::new(&workspace_dir);
        let config = DocumentCreationConfig {
            title: "Test Strategy".to_string(),
            description: Some("A test strategy".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None, // Should default to Shaping
            complexity: None,
            risk_level: None,
        };
        let creation_result = creation_service.create_strategy(config).await.unwrap();

        let transition_service = PhaseTransitionService::new(&workspace_dir);
        let document_id = creation_result.document_id.to_string();

        // Transition through the strategy phases
        let result1 = transition_service
            .transition_to_next_phase(&document_id)
            .await
            .unwrap();
        assert_eq!(result1.from_phase, Phase::Shaping);
        assert_eq!(result1.to_phase, Phase::Design);

        let result2 = transition_service
            .transition_to_next_phase(&document_id)
            .await
            .unwrap();
        assert_eq!(result2.from_phase, Phase::Design);
        assert_eq!(result2.to_phase, Phase::Ready);

        let result3 = transition_service
            .transition_to_next_phase(&document_id)
            .await
            .unwrap();
        assert_eq!(result3.from_phase, Phase::Ready);
        assert_eq!(result3.to_phase, Phase::Active);

        let result4 = transition_service
            .transition_to_next_phase(&document_id)
            .await
            .unwrap();
        assert_eq!(result4.from_phase, Phase::Active);
        assert_eq!(result4.to_phase, Phase::Completed);
    }

    #[tokio::test]
    async fn test_transition_to_specific_phase() {
        let (_temp_dir, workspace_dir) = setup_test_workspace().await;

        // Create a vision document
        let creation_service = DocumentCreationService::new(&workspace_dir);
        let config = DocumentCreationConfig {
            title: "Test Vision".to_string(),
            description: Some("A test vision".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None, // Should default to Draft
            complexity: None,
            risk_level: None,
        };
        let creation_result = creation_service.create_vision(config).await.unwrap();

        // Transition directly to Review phase
        let transition_service = PhaseTransitionService::new(&workspace_dir);
        let transition_result = transition_service
            .transition_document(&creation_result.document_id.to_string(), Phase::Review)
            .await
            .unwrap();

        assert_eq!(transition_result.from_phase, Phase::Draft);
        assert_eq!(transition_result.to_phase, Phase::Review);
    }

    #[tokio::test]
    async fn test_invalid_transition() {
        let (_temp_dir, workspace_dir) = setup_test_workspace().await;

        // Create a vision document
        let creation_service = DocumentCreationService::new(&workspace_dir);
        let config = DocumentCreationConfig {
            title: "Test Vision".to_string(),
            description: Some("A test vision".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None, // Should default to Draft
            complexity: None,
            risk_level: None,
        };
        let creation_result = creation_service.create_vision(config).await.unwrap();

        // Try to transition directly to Published (should fail)
        let transition_service = PhaseTransitionService::new(&workspace_dir);
        let result = transition_service
            .transition_document(&creation_result.document_id.to_string(), Phase::Published)
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MetisError::InvalidPhaseTransition { .. }
        ));
    }

    #[tokio::test]
    async fn test_get_valid_transitions() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");

        let transition_service = PhaseTransitionService::new(&workspace_dir);

        // Test vision transitions
        let vision_draft_transitions =
            transition_service.get_valid_transitions_for(DocumentType::Vision, Phase::Draft);
        assert_eq!(vision_draft_transitions, vec![Phase::Review]);

        let vision_review_transitions =
            transition_service.get_valid_transitions_for(DocumentType::Vision, Phase::Review);
        assert_eq!(
            vision_review_transitions,
            vec![Phase::Draft, Phase::Published]
        );

        // Test strategy transitions
        let strategy_shaping_transitions =
            transition_service.get_valid_transitions_for(DocumentType::Strategy, Phase::Shaping);
        assert_eq!(strategy_shaping_transitions, vec![Phase::Design]);

        // Test task transitions - specifically backlog to todo
        let task_backlog_transitions =
            transition_service.get_valid_transitions_for(DocumentType::Task, Phase::Backlog);
        assert_eq!(task_backlog_transitions, vec![Phase::Todo]);
    }

    #[tokio::test]
    async fn test_is_valid_transition() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");

        let transition_service = PhaseTransitionService::new(&workspace_dir);

        // Valid transitions
        assert!(transition_service.is_valid_transition(
            DocumentType::Vision,
            Phase::Draft,
            Phase::Review
        ));
        assert!(transition_service.is_valid_transition(
            DocumentType::Strategy,
            Phase::Shaping,
            Phase::Design
        ));

        // Invalid transitions
        assert!(!transition_service.is_valid_transition(
            DocumentType::Vision,
            Phase::Draft,
            Phase::Published
        ));
        assert!(!transition_service.is_valid_transition(
            DocumentType::Strategy,
            Phase::Shaping,
            Phase::Active
        ));
    }
}
