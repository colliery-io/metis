//! Integration tests for the Specification document type

use metis_core::application::services::{
    document::creation::{DocumentCreationConfig, DocumentCreationService},
    workspace::{PhaseTransitionService, WorkspaceDetectionService, WorkspaceInitializationService},
};
use metis_core::domain::documents::types::{DocumentType, Phase};
use metis_core::{Document, Specification};
use tempfile::tempdir;

/// Helper to set up workspace and return (temp_dir, metis_dir)
async fn setup_workspace() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = tempdir().unwrap();
    let result =
        WorkspaceInitializationService::initialize_workspace(temp_dir.path(), "Test Project")
            .await
            .expect("Failed to initialize workspace");
    (temp_dir, result.metis_dir)
}

#[tokio::test]
async fn test_create_specification_with_parent() {
    let (_temp_dir, metis_dir) = setup_workspace().await;
    let creation_service = DocumentCreationService::new(&metis_dir);

    let config = DocumentCreationConfig {
        title: "System Design Specification".to_string(),
        description: None,
        parent_id: Some("TEST-V-0001".into()),
        tags: vec![],
        phase: None,
        complexity: None,
    };

    let result = creation_service.create_specification(config).await;
    assert!(result.is_ok(), "Create spec should succeed: {:?}", result.err());

    let result = result.unwrap();
    assert_eq!(result.document_type, DocumentType::Specification);
    assert!(result.file_path.exists(), "Spec file should exist");
    assert!(
        result.short_code.contains("-S-"),
        "Short code should contain -S-: {}",
        result.short_code
    );

    // Verify roundtrip
    let spec = Specification::from_file(&result.file_path).await.unwrap();
    assert_eq!(spec.title(), "System Design Specification");
    assert_eq!(spec.document_type(), DocumentType::Specification);
}

#[tokio::test]
async fn test_create_specification_without_parent_fails() {
    let (_temp_dir, metis_dir) = setup_workspace().await;
    let creation_service = DocumentCreationService::new(&metis_dir);

    let config = DocumentCreationConfig {
        title: "No Parent Spec".to_string(),
        description: None,
        parent_id: None,
        tags: vec![],
        phase: None,
        complexity: None,
    };

    let result = creation_service.create_specification(config).await;
    assert!(result.is_err(), "Create spec without parent should fail");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("parent"),
        "Error should mention parent: {}",
        err
    );
}

#[tokio::test]
async fn test_specification_phase_transitions() {
    let (_temp_dir, metis_dir) = setup_workspace().await;

    // Create specification
    let creation_service = DocumentCreationService::new(&metis_dir);
    let config = DocumentCreationConfig {
        title: "Phase Test Spec".to_string(),
        description: None,
        parent_id: Some("TEST-V-0001".into()),
        tags: vec![],
        phase: None,
        complexity: None,
    };
    let result = creation_service.create_specification(config).await.unwrap();
    let short_code = result.short_code;

    // Sync so DB knows about the spec
    let detection_service = WorkspaceDetectionService::new();
    let _db = detection_service.prepare_workspace(&metis_dir).await.unwrap();

    // Transition through all phases: Discovery -> Drafting -> Review -> Published
    let transition_service = PhaseTransitionService::new(&metis_dir);

    let result = transition_service.transition_to_next_phase(&short_code).await;
    assert!(result.is_ok(), "Discovery->Drafting should succeed: {:?}", result.err());
    assert_eq!(result.unwrap().to_phase, Phase::Drafting);

    let result = transition_service.transition_to_next_phase(&short_code).await;
    assert!(result.is_ok(), "Drafting->Review should succeed: {:?}", result.err());
    assert_eq!(result.unwrap().to_phase, Phase::Review);

    let result = transition_service.transition_to_next_phase(&short_code).await;
    assert!(result.is_ok(), "Review->Published should succeed: {:?}", result.err());
    assert_eq!(result.unwrap().to_phase, Phase::Published);
}

#[tokio::test]
async fn test_specification_sync_and_discovery() {
    let (_temp_dir, metis_dir) = setup_workspace().await;

    // Create specification
    let creation_service = DocumentCreationService::new(&metis_dir);
    let config = DocumentCreationConfig {
        title: "Sync Test Spec".to_string(),
        description: None,
        parent_id: Some("TEST-V-0001".into()),
        tags: vec![],
        phase: None,
        complexity: None,
    };
    let result = creation_service.create_specification(config).await.unwrap();
    let short_code = result.short_code.clone();

    // Sync workspace — specification should be imported into database
    let detection_service = WorkspaceDetectionService::new();
    let db = detection_service.prepare_workspace(&metis_dir).await.unwrap();

    // Verify specification is in database
    let mut repo = db.into_repository();
    let specs = repo.find_by_type("specification").unwrap();
    assert_eq!(specs.len(), 1, "Should find 1 specification in DB");
    assert_eq!(specs[0].short_code, short_code);
    assert_eq!(specs[0].title, "Sync Test Spec");
}

#[tokio::test]
async fn test_specification_archive_no_cascade() {
    let (_temp_dir, metis_dir) = setup_workspace().await;

    // Create specification
    let creation_service = DocumentCreationService::new(&metis_dir);
    let config = DocumentCreationConfig {
        title: "Archive Test Spec".to_string(),
        description: None,
        parent_id: Some("TEST-V-0001".into()),
        tags: vec![],
        phase: None,
        complexity: None,
    };
    let result = creation_service.create_specification(config).await.unwrap();

    // Sync so DB knows about the spec
    let detection_service = WorkspaceDetectionService::new();
    let db = detection_service.prepare_workspace(&metis_dir).await.unwrap();

    // Archive the specification using DatabaseService
    use metis_core::application::services::workspace::ArchiveService;
    use metis_core::application::services::DatabaseService;
    let archive_service = ArchiveService::new(&metis_dir);
    let mut db_service = DatabaseService::new(db.into_repository());
    let archive_result = archive_service
        .archive_document_by_short_code(&result.short_code, &mut db_service)
        .await;
    assert!(
        archive_result.is_ok(),
        "Archive should succeed: {:?}",
        archive_result.err()
    );

    // Verify file is moved to archived
    assert!(
        !result.file_path.exists(),
        "Original file should no longer exist"
    );
}
