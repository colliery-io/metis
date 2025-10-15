use crate::AppState;
use metis_core::{
    application::services::workspace::transition::PhaseTransitionService,
    domain::documents::types::Phase, Application, Database,
};
use tauri::State;

fn parse_phase(phase_str: &str) -> Result<Phase, String> {
    match phase_str.to_lowercase().as_str() {
        "draft" => Ok(Phase::Draft),
        "review" => Ok(Phase::Review),
        "published" => Ok(Phase::Published),
        "discussion" => Ok(Phase::Discussion),
        "decided" => Ok(Phase::Decided),
        "superseded" => Ok(Phase::Superseded),
        "backlog" => Ok(Phase::Backlog),
        "todo" => Ok(Phase::Todo),
        "active" => Ok(Phase::Active),
        "blocked" => Ok(Phase::Blocked),
        "completed" => Ok(Phase::Completed),
        "shaping" => Ok(Phase::Shaping),
        "design" => Ok(Phase::Design),
        "ready" => Ok(Phase::Ready),
        "decompose" => Ok(Phase::Decompose),
        "discovery" => Ok(Phase::Discovery),
        "doing" => Ok(Phase::Active), // Map "doing" to Active
        _ => Err(format!("Unknown phase: {}", phase_str)),
    }
}

#[tauri::command]
pub async fn transition_phase(
    state: State<'_, std::sync::Mutex<AppState>>,
    short_code: String,
    new_phase: Option<String>,
) -> Result<String, String> {
    let project_path = {
        let app_state = state
            .lock()
            .map_err(|e| format!("Failed to lock state: {}", e))?;
        app_state
            .current_project
            .as_ref()
            .ok_or("No project loaded")?
            .clone()
    };

    let metis_dir = project_path.join(".metis");

    let transition_service = PhaseTransitionService::new(&metis_dir);

    // Perform the transition using short code directly
    let result = if let Some(phase_str) = new_phase {
        // Transition to specific phase
        let target_phase = parse_phase(&phase_str)?;
        transition_service
            .transition_document(&short_code, target_phase)
            .await
            .map_err(|e| format!("Failed to transition phase: {}", e))?
    } else {
        // Auto-transition to next phase
        transition_service
            .transition_to_next_phase(&short_code)
            .await
            .map_err(|e| format!("Failed to transition phase: {}", e))?
    };

    // Auto-sync after transition to update database
    let db_path = metis_dir.join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database for sync: {}", e))?;
    let app = Application::new(database);

    let sync_results = app
        .sync_directory(&metis_dir)
        .await
        .map_err(|e| format!("Failed to sync workspace: {}", e))?;

    tracing::info!("Sync completed with {} results", sync_results.len());
    for sync_result in &sync_results {
        tracing::info!("Sync result: {:?}", sync_result);
    }

    // Verify the document was actually updated in the database
    let db_path = metis_dir.join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database for verification: {}", e))?;
    let mut app_verify = Application::new(database);

    // Check if the document in the database has the correct phase
    let updated_doc = app_verify
        .with_database(|service| service.find_by_short_code(&short_code))
        .map_err(|e| format!("Failed to verify document update: {}", e))?;

    if let Some(doc) = updated_doc {
        tracing::info!("Verified document phase in database: {}", doc.phase);
        if doc.phase != result.to_phase.to_string() {
            tracing::warn!(
                "Database phase {} doesn't match expected phase {}",
                doc.phase,
                result.to_phase
            );
        }
    } else {
        tracing::warn!("Document not found in database after sync");
    }

    Ok(result.to_phase.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;


    #[tokio::test]
    async fn test_parse_phase_valid() {
        assert!(parse_phase("draft").is_ok());
        assert!(parse_phase("review").is_ok());
        assert!(parse_phase("doing").is_ok()); // Should map to Active
    }

    #[tokio::test]
    async fn test_parse_phase_invalid() {
        let result = parse_phase("invalid_phase");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown phase"));
    }
}
