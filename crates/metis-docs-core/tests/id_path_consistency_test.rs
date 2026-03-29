use metis_core::application::services::document::creation::{
    DocumentCreationConfig, DocumentCreationService,
};
use metis_core::application::services::document::discovery::DocumentDiscoveryService;
use metis_core::application::services::workspace::initialization::WorkspaceInitializationService;
use metis_core::domain::documents::types::{DocumentType, Tag};
use metis_core::{Complexity, Database};
use std::path::PathBuf;
use tempfile::tempdir;

// Helper function to setup workspace with configuration
async fn setup_test_workspace(project_name: &str) -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempdir().unwrap();
    let workspace_dir = temp_dir.path().join(".metis");

    // Initialize workspace
    WorkspaceInitializationService::initialize_workspace(&workspace_dir, project_name)
        .await
        .unwrap();

    // Setup database configuration
    let db_path = workspace_dir.join("metis.db");
    let db = Database::new(&db_path.to_string_lossy()).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();
    config_repo.set_project_prefix("TEST").unwrap();

    (temp_dir, workspace_dir)
}

#[tokio::test]
async fn test_initiative_short_code_matches_path() {
    let (_temp_dir, workspace_dir) = setup_test_workspace("test-project").await;
    let creation_service = DocumentCreationService::new(&workspace_dir);
    let discovery_service = DocumentDiscoveryService::new(&workspace_dir);

    // Test Initiative
    let initiative_config = DocumentCreationConfig {
        title: "Semantic Search RAG Initiative".to_string(),
        description: Some("An initiative for semantic search".to_string()),
        parent_id: None,
        tags: vec![],
        phase: None,
        complexity: None,
    };

    let initiative_result = creation_service
        .create_initiative(initiative_config)
        .await
        .unwrap();

    // Verify the ID still uses kebab-case for compatibility
    assert_eq!(
        initiative_result.document_id.to_string(),
        "semantic-search-rag-initiative"
    );

    // Verify the directory path uses the short code (not the document ID)
    let expected_path = workspace_dir
        .join("initiatives")
        .join(&initiative_result.short_code);
    assert!(expected_path.exists());
    assert!(expected_path.join("initiative.md").exists());

    // Verify discovery service finds it with the correct ID
    let found = discovery_service
        .find_document_by_id("semantic-search-rag-initiative")
        .await
        .unwrap();
    assert_eq!(found.document_type, DocumentType::Initiative);
    // Canonicalize expected path to match discovery service's canonical workspace_dir
    let expected_file_path = expected_path.join("initiative.md").canonicalize().unwrap();
    assert_eq!(found.file_path, expected_file_path);
}

#[tokio::test]
async fn test_initiative_id_path_consistency() {
    let (_temp_dir, workspace_dir) = setup_test_workspace("test-project").await;
    let creation_service = DocumentCreationService::new(&workspace_dir);

    // Create initiative with complex title
    let initiative_config = DocumentCreationConfig {
        title: "Build AI-Powered Search & Retrieval System".to_string(),
        description: Some("Initiative for AI search".to_string()),
        parent_id: None,
        tags: vec![],
        phase: None,
        complexity: Some(Complexity::L),
    };

    let initiative_result = creation_service
        .create_initiative(initiative_config)
        .await
        .unwrap();

    // Verify ID generation still uses kebab-case (for compatibility)
    assert_eq!(
        initiative_result.document_id.to_string(),
        "build-ai-powered-search-retrieval"
    );

    // Verify path now uses short codes instead of document IDs
    let expected_path = workspace_dir
        .join("initiatives")
        .join(&initiative_result.short_code);

    assert!(expected_path.exists());
    assert!(expected_path.join("initiative.md").exists());
}

#[tokio::test]
async fn test_task_id_path_consistency() {
    let (_temp_dir, workspace_dir) = setup_test_workspace("test-project").await;
    let creation_service = DocumentCreationService::new(&workspace_dir);

    // Create parent initiative
    let initiative_config = DocumentCreationConfig {
        title: "Initiative One".to_string(),
        description: None,
        parent_id: None,
        tags: vec![],
        phase: None,
        complexity: None,
    };
    let initiative = creation_service
        .create_initiative(initiative_config)
        .await
        .unwrap();

    // Sync initiative to database for task creation
    let db_path = workspace_dir.join("metis.db");
    let db = Database::new(&db_path.to_string_lossy()).unwrap();
    let mut db_service =
        metis_core::application::services::DatabaseService::new(db.repository().unwrap());
    let mut sync_service = metis_core::application::services::SyncService::new(&mut db_service);
    sync_service
        .import_from_file(&initiative.file_path)
        .await
        .unwrap();

    // Create task with special characters
    let task_config = DocumentCreationConfig {
        title: "Setup CI/CD Pipeline & Testing Framework".to_string(),
        description: Some("Task with special chars".to_string()),
        parent_id: Some(initiative.document_id.clone()),
        tags: vec![Tag::Label("devops".to_string())],
        phase: None,
        complexity: None,
    };

    let task_result = creation_service
        .create_task(task_config, &initiative.short_code)
        .await
        .unwrap();

    // Verify ID generation still handles special chars (for compatibility)
    assert_eq!(
        task_result.document_id.to_string(),
        "setup-ci-cd-pipeline-testing"
    );

    // Verify file path now uses short codes in directory structure
    let expected_file = workspace_dir
        .join("initiatives")
        .join(&initiative.short_code)
        .join("tasks")
        .join(format!("{}.md", task_result.short_code));

    assert!(expected_file.exists());
    assert_eq!(task_result.file_path, expected_file);
}

#[tokio::test]
async fn test_adr_id_consistency() {
    let (_temp_dir, workspace_dir) = setup_test_workspace("test-project").await;
    let creation_service = DocumentCreationService::new(&workspace_dir);

    // Create ADR
    let adr_config = DocumentCreationConfig {
        title: "Use PostgreSQL for Primary Database".to_string(),
        description: Some("Database selection decision".to_string()),
        parent_id: None,
        tags: vec![
            Tag::Label("architecture".to_string()),
            Tag::Label("database".to_string()),
        ],
        phase: None,
        complexity: None,
    };

    let adr_result = creation_service.create_adr(adr_config).await.unwrap();

    // ADRs still have ID format: NNN-slug (for compatibility)
    let expected_id = "001-use-postgresql-for-primary-data";
    assert_eq!(adr_result.document_id.to_string(), expected_id);

    // Verify file name now uses short code format: TEST-A-0001.md
    let file_name = adr_result.file_path.file_name().unwrap().to_str().unwrap();
    assert_eq!(file_name, format!("{}.md", adr_result.short_code));
    assert!(adr_result.short_code.starts_with("TEST-A-"));
    assert!(file_name.ends_with(".md"));
}

#[tokio::test]
async fn test_long_title_id_path_consistency() {
    let (_temp_dir, workspace_dir) = setup_test_workspace("test-project").await;
    let creation_service = DocumentCreationService::new(&workspace_dir);

    // Create initiative with very long title
    let long_title = "This is an extremely long initiative title that should be truncated to ensure file paths don't exceed system limits while still maintaining readability and uniqueness in the generated identifier".to_string();

    let initiative_config = DocumentCreationConfig {
        title: long_title.clone(),
        description: None,
        parent_id: None,
        tags: vec![],
        phase: None,
        complexity: None,
    };

    let result = creation_service
        .create_initiative(initiative_config)
        .await
        .unwrap();

    // Verify ID is still truncated appropriately (for compatibility)
    assert!(result.document_id.to_string().len() <= 35); // MAX_ID_LENGTH

    // Verify path now uses short code instead of document ID
    let initiative_dir = workspace_dir.join("initiatives").join(&result.short_code);
    assert!(initiative_dir.exists());
    assert!(initiative_dir.join("initiative.md").exists());

    // Verify we can find it by ID
    let discovery_service = DocumentDiscoveryService::new(&workspace_dir);
    let found = discovery_service
        .find_document_by_id(&result.document_id.to_string())
        .await;
    assert!(found.is_ok());
}

#[tokio::test]
async fn test_unicode_title_id_path_consistency() {
    let (_temp_dir, workspace_dir) = setup_test_workspace("test-project").await;
    let creation_service = DocumentCreationService::new(&workspace_dir);

    // Test various Unicode titles
    let unicode_titles = [
        "Café Initiative für München",
        "日本語 テスト 計画",
        "Инициатива для России",
        "🚀 Rocket Launch Initiative 🌟",
        "Initiative with émojis and àccents",
    ];

    for (i, title) in unicode_titles.iter().enumerate() {
        let config = DocumentCreationConfig {
            title: title.to_string(),
            description: Some(format!("Unicode test {}", i)),
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
        };

        let result = creation_service.create_initiative(config).await.unwrap();

        // Verify ID contains only valid slug characters (alphanumeric includes Unicode)
        let id = result.document_id.to_string();
        assert!(
            id.chars().all(|c| c.is_alphanumeric() || c == '-'),
            "ID should only contain alphanumeric and hyphens: {}",
            id
        );
        assert!(
            !id.contains("--"),
            "ID should not contain double hyphens: {}",
            id
        );

        // Verify path exists using short code
        let initiative_dir = workspace_dir.join("initiatives").join(&result.short_code);
        assert!(
            initiative_dir.exists(),
            "Directory should exist for Unicode title: {}",
            title
        );
        assert!(initiative_dir.join("initiative.md").exists());
    }
}

/// Regression test for the ID/path mismatch bug
/// This ensures that when we create a document, the ID in the database/frontmatter
/// always matches the directory path on the filesystem
#[tokio::test]
async fn test_regression_id_path_mismatch_bug() {
    let (_temp_dir, workspace_dir) = setup_test_workspace("test-project").await;
    let creation_service = DocumentCreationService::new(&workspace_dir);
    let discovery_service = DocumentDiscoveryService::new(&workspace_dir);

    // This was the problematic case: create an initiative with a long ID
    // but the directory was created as a truncated version
    let initiative_config = DocumentCreationConfig {
        title: "Semantic Search RAG Initiative".to_string(),
        description: Some("An initiative for implementing semantic search with RAG".to_string()),
        parent_id: None,
        tags: vec![
            Tag::Label("search".to_string()),
            Tag::Label("ai".to_string()),
        ],
        phase: None,
        complexity: Some(Complexity::L),
    };

    let initiative_result = creation_service
        .create_initiative(initiative_config)
        .await
        .unwrap();
    let initiative_id = initiative_result.short_code.clone();

    // The document ID should still be the full slug (for compatibility)
    assert_eq!(
        initiative_result.document_id.to_string(),
        "semantic-search-rag-initiative"
    );

    // The directory path now uses short code instead of document ID
    let initiative_dir = workspace_dir
        .join("initiatives")
        .join(&initiative_result.short_code);
    assert!(
        initiative_dir.exists(),
        "Initiative directory should exist at path using short code"
    );
    assert!(
        initiative_dir.join("initiative.md").exists(),
        "Initiative file should exist"
    );

    // Verify the frontmatter ID matches
    let initiative_content = std::fs::read_to_string(initiative_dir.join("initiative.md")).unwrap();
    assert!(
        initiative_content.contains(&format!("id: {}", initiative_result.document_id)),
        "Frontmatter ID should match"
    );

    // Verify discovery by ID works
    let found = discovery_service
        .find_document_by_id(&initiative_result.document_id.to_string())
        .await
        .unwrap();
    // Canonicalize expected path to match discovery service's canonical workspace_dir
    let expected_file_path = initiative_dir.join("initiative.md").canonicalize().unwrap();
    assert_eq!(found.file_path, expected_file_path);

    // Sync initiative to database before creating task
    let db_path = workspace_dir.join("metis.db");
    let db = Database::new(&db_path.to_string_lossy()).unwrap();
    let mut db_service =
        metis_core::application::services::DatabaseService::new(db.repository().unwrap());
    let mut sync_service = metis_core::application::services::SyncService::new(&mut db_service);
    sync_service
        .import_from_file(&initiative_result.file_path)
        .await
        .unwrap();

    // Now create a task under this initiative to ensure child paths work correctly
    let task_config = DocumentCreationConfig {
        title: "Implement Vector Search".to_string(),
        description: Some("Task to implement vector search capabilities".to_string()),
        parent_id: Some(metis_core::domain::documents::types::DocumentId::from(
            initiative_id.clone(),
        )),
        tags: vec![],
        phase: None,
        complexity: None,
    };

    let task_result = creation_service
        .create_task(task_config, &initiative_id)
        .await
        .unwrap();

    // Verify task is created under the correct parent path using short code
    let task_path = initiative_dir
        .join("tasks")
        .join(format!("{}.md", task_result.short_code));

    assert!(
        task_path.exists(),
        "Task should be created under correct parent initiative path"
    );
}
