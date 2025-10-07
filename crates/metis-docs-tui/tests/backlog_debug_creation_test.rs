use anyhow::Result;
use metis_docs_tui::models::{AppState, BoardType};
use tui_input::backend::crossterm::EventHandler;

mod common;
use common::TestHelper;

#[tokio::test]
async fn test_debug_backlog_creation() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Initial State ===");
    println!("Current board: {:?}", app.ui_state.current_board);

    // Jump to backlog board
    app.jump_to_backlog_board();
    println!(
        "After jump - Current board: {:?}",
        app.ui_state.current_board
    );
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    // Check initial backlog board state
    let backlog_board = &app.ui_state.backlog_board;
    let initial_total: usize = backlog_board.columns.iter().map(|c| c.items.len()).sum();
    println!("Initial backlog items: {}", initial_total);

    // Start document creation
    app.start_document_creation();
    println!(
        "App state after start_document_creation: {:?}",
        app.app_state()
    );
    assert_eq!(*app.app_state(), AppState::CreatingDocument);

    // Simulate user typing a title
    let title = "Test backlog task";
    for ch in title.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }

    println!("Title entered: '{}'", app.ui_state.input_title.value());

    // Create the document
    println!("=== Creating Document ===");
    match app.create_new_document().await {
        Ok(_) => println!("Document creation returned Ok"),
        Err(e) => {
            println!("Document creation failed: {:?}", e);
            return Err(e);
        }
    }

    println!("App state after create_new_document: {:?}", app.app_state());

    // Check if document service exists
    println!(
        "Document service exists: {}",
        app.document_service.is_some()
    );

    // Check what files were created
    println!("=== File System Check ===");
    let metis_dir = &helper.metis_dir();
    if let Ok(entries) = std::fs::read_dir(metis_dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() && entry.path().extension().is_some_and(|ext| ext == "md") {
                println!("Found .md file: {:?}", entry.path());
            }
        }
    }

    // Check backlog directory and file content
    let backlog_dir = metis_dir.join("backlog");
    if backlog_dir.exists() {
        println!("Backlog directory exists");
        if let Ok(entries) = std::fs::read_dir(&backlog_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                println!("Backlog file: {:?}", path);
                if let Ok(content) = std::fs::read_to_string(&path) {
                    println!("File content:\n{}", content);
                }
            }
        }
    } else {
        println!("Backlog directory does not exist");
    }

    // Debug filesystem scanning
    println!("=== Filesystem Scanning Debug ===");
    match metis_core::application::services::FilesystemService::find_markdown_files(
        helper.metis_dir(),
    ) {
        Ok(files) => {
            println!("FilesystemService found {} files:", files.len());
            for file in &files {
                println!("  {}", file);
            }
        }
        Err(e) => {
            println!("FilesystemService error: {:?}", e);
        }
    }

    // Force sync and debug with detailed results
    println!("=== Explicit Sync Check ===");
    let db_path = helper.metis_dir().join("metis.db");
    let db = metis_core::dal::Database::new(&db_path.to_string_lossy()).unwrap();
    let app_core = metis_core::application::Application::new(db);

    match app_core.sync_directory(&helper.metis_dir()).await {
        Ok(results) => {
            println!("Sync completed with {} results:", results.len());
            for result in &results {
                println!("  {:?}", result);
            }
        }
        Err(e) => println!("Sync failed: {:?}", e),
    }

    // Check database after explicit sync
    println!("=== Database Check ===");
    if let Some(ref doc_service) = app.document_service {
        match doc_service.load_documents_from_database().await {
            Ok(docs) => {
                println!("Documents in database: {}", docs.len());
                for doc in docs {
                    println!(
                        "  - '{}' type: {:?} path: {}",
                        doc.title, doc.document_type, doc.filepath
                    );
                }
            }
            Err(e) => println!("Error loading from database: {:?}", e),
        }
    }

    // Check backlog board after creation
    println!("=== Backlog Board Check ===");
    let backlog_board = &app.ui_state.backlog_board;
    let final_total: usize = backlog_board.columns.iter().map(|c| c.items.len()).sum();
    println!("Final backlog items: {}", final_total);

    for (i, column) in backlog_board.columns.iter().enumerate() {
        println!(
            "Column {}: '{}' has {} items",
            i,
            column.title,
            column.items.len()
        );
        for (j, item) in column.items.iter().enumerate() {
            println!("  Item {}: '{}'", j, item.title());
        }
    }

    Ok(())
}
