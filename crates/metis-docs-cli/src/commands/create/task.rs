use crate::workspace;
use anyhow::Result;
use metis_core::{
    Task, Initiative, Document,
    Tag, Phase,
    domain::documents::{
        types::DocumentId,
    },
};
use std::path::Path;

/// Create a new Task document with defaults and write to file
pub async fn create_new_task(title: &str, initiative_id: &str) -> Result<()> {
    // 1. Validate we're in a metis workspace
    let (workspace_exists, metis_dir) = workspace::has_metis_vault();
    if !workspace_exists {
        anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
    }
    let metis_dir = metis_dir.unwrap();
    
    // 2. Verify the initiative exists and get its document ID and file path
    let (initiative_doc_id, initiative_path) = find_initiative(&metis_dir, initiative_id).await?;
    
    // 3. Create Task with defaults
    let tags = vec![
        Tag::Label("task".to_string()),
        Tag::Phase(Phase::Todo),
    ];
    
    let task = Task::new(
        title.to_string(),
        Some(initiative_doc_id.clone()),
        Some(initiative_id.to_string()), // parent title for template
        Vec::new(), // blocked_by
        tags,
        false, // not archived
    ).map_err(|e| anyhow::anyhow!("Failed to create task: {}", e))?;
    
    // 4. Determine file path: /strategies/{strategy-id}/initiatives/{initiative-id}/{task-id}.md
    let doc_id = task.id();
    let initiative_dir = initiative_path.parent().unwrap();
    let file_path = initiative_dir.join(format!("{}.md", doc_id));
    
    // Check if file already exists
    if file_path.exists() {
        anyhow::bail!("Task document already exists: {}", file_path.display());
    }
    
    // 5. Write to file
    task.to_file(&file_path).await?;
    
    println!("✓ Created task: {}", file_path.display());
    println!("  ID: {}", doc_id);
    println!("  Title: {}", title);
    println!("  Parent Initiative: {}", initiative_doc_id);
    
    Ok(())
}

/// Find an initiative by ID and return its DocumentId and file path
async fn find_initiative(workspace_dir: &Path, initiative_id: &str) -> Result<(DocumentId, std::path::PathBuf)> {
    let strategies_dir = workspace_dir.join("strategies");
    
    if !strategies_dir.exists() {
        anyhow::bail!("No strategies directory found. Create a strategy and initiative first.");
    }
    
    // Search through all strategy directories for the initiative
    for strategy_entry in std::fs::read_dir(&strategies_dir)? {
        let strategy_dir = strategy_entry?.path();
        if !strategy_dir.is_dir() {
            continue;
        }
        
        let initiatives_dir = strategy_dir.join("initiatives");
        if !initiatives_dir.exists() {
            continue;
        }
        
        // Look for the initiative directory
        let initiative_dir = initiatives_dir.join(initiative_id);
        if !initiative_dir.exists() || !initiative_dir.is_dir() {
            continue;
        }
        
        // Parse the initiative document to get its actual ID
        let initiative_path = initiative_dir.join("initiative.md");
        if !initiative_path.exists() {
            continue;
        }
        
        let initiative = Initiative::from_file(&initiative_path).await
            .map_err(|e| anyhow::anyhow!("Failed to parse initiative document: {}", e))?;
        
        let initiative_doc_id = initiative.id();
        
        // Verify the directory name matches the initiative ID
        if initiative_doc_id.to_string() == initiative_id {
            return Ok((initiative_doc_id, initiative_path));
        }
    }
    
    // If we get here, initiative wasn't found
    let available = list_available_initiatives(workspace_dir)?;
    if available.is_empty() {
        anyhow::bail!("No initiatives found. Create an initiative first.");
    } else {
        anyhow::bail!(
            "Initiative '{}' not found. Available initiatives: {}",
            initiative_id,
            available.join(", ")
        );
    }
}

/// List all available initiative IDs across all strategies
fn list_available_initiatives(workspace_dir: &Path) -> Result<Vec<String>> {
    let mut initiatives = Vec::new();
    let strategies_dir = workspace_dir.join("strategies");
    
    if !strategies_dir.exists() {
        return Ok(initiatives);
    }
    
    for strategy_entry in std::fs::read_dir(&strategies_dir)? {
        let strategy_dir = strategy_entry?.path();
        if !strategy_dir.is_dir() {
            continue;
        }
        
        let initiatives_dir = strategy_dir.join("initiatives");
        if !initiatives_dir.exists() {
            continue;
        }
        
        for initiative_entry in std::fs::read_dir(&initiatives_dir)? {
            let initiative_dir = initiative_entry?.path();
            if initiative_dir.is_dir() {
                if let Some(name) = initiative_dir.file_name().and_then(|n| n.to_str()) {
                    initiatives.push(name.to_string());
                }
            }
        }
    }
    
    initiatives.sort();
    Ok(initiatives)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use crate::commands::InitCommand;

    #[tokio::test]
    async fn test_create_new_task_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        // Change to temp directory without workspace
        if std::env::set_current_dir(temp_dir.path()).is_ok() {
            let result = create_new_task("Test Task", "some-initiative").await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Not in a Metis workspace"));
            
            // Restore original directory
            let _ = std::env::set_current_dir(original_dir);
        }
    }

    #[tokio::test]
    async fn test_find_initiative_not_found() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        // Ensure we can change to temp directory
        if std::env::set_current_dir(temp_dir.path()).is_err() {
            std::env::set_current_dir(original_dir).unwrap();
            return;
        }
        
        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
        };
        if init_cmd.execute().await.is_err() {
            std::env::set_current_dir(original_dir).unwrap();
            return;
        }
        
        // Try to find non-existent initiative
        let metis_dir = temp_dir.path().join(".metis");
        
        // Create strategies directory but no actual strategies
        fs::create_dir_all(metis_dir.join("strategies")).unwrap();
        
        let result = find_initiative(&metis_dir, "non-existent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No initiatives found"));
        
        // Always restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_list_available_initiatives() {
        let temp_dir = tempdir().unwrap();
        let strategies_dir = temp_dir.path().join("strategies");
        
        // Create some strategy and initiative directories
        fs::create_dir_all(&strategies_dir).unwrap();
        
        // Strategy 1 with initiatives
        let strategy1_dir = strategies_dir.join("strategy-1");
        let initiatives1_dir = strategy1_dir.join("initiatives");
        fs::create_dir_all(&initiatives1_dir).unwrap();
        fs::create_dir(initiatives1_dir.join("initiative-1")).unwrap();
        fs::create_dir(initiatives1_dir.join("initiative-2")).unwrap();
        
        // Strategy 2 with initiatives  
        let strategy2_dir = strategies_dir.join("strategy-2");
        let initiatives2_dir = strategy2_dir.join("initiatives");
        fs::create_dir_all(&initiatives2_dir).unwrap();
        fs::create_dir(initiatives2_dir.join("another-initiative")).unwrap();
        
        let initiatives = list_available_initiatives(temp_dir.path()).unwrap();
        assert_eq!(initiatives.len(), 3);
        assert!(initiatives.contains(&"initiative-1".to_string()));
        assert!(initiatives.contains(&"initiative-2".to_string()));
        assert!(initiatives.contains(&"another-initiative".to_string()));
    }

}