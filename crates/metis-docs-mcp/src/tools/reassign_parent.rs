use crate::formatting::{error_result, ToolOutput};
use metis_core::application::services::workspace::{
    BacklogCategory, ReassignmentService, WorkspaceDetectionService,
};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "reassign_parent",
    description = "Move a task to a different parent initiative, or move it to/from the backlog. Only tasks can be reassigned. Use this to assign backlog items to initiatives or move tasks between initiatives.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReassignParentTool {
    /// Path to the .metis folder (e.g., "/Users/me/my-project/.metis"). Must end with .metis
    pub project_path: String,
    /// Short code of the task to reassign (e.g., PROJ-T-0001)
    pub short_code: String,
    /// Short code of the new parent initiative (e.g., PROJ-I-0001). Omit or set to null to move to backlog.
    pub new_parent_id: Option<String>,
    /// Required when moving to backlog: bug, feature, or tech-debt
    pub backlog_category: Option<String>,
}

impl ReassignParentTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let metis_dir = Path::new(&self.project_path);

        // Prepare workspace
        let detection_service = WorkspaceDetectionService::new();
        let db = detection_service
            .prepare_workspace(metis_dir)
            .await
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        let mut db_service =
            metis_core::application::services::DatabaseService::new(db.into_repository());
        let reassignment_service = ReassignmentService::new(metis_dir);

        // Determine operation: to initiative or to backlog
        let result = if let Some(parent_id) = &self.new_parent_id {
            // Reassign to initiative
            reassignment_service
                .reassign_to_initiative(&self.short_code, parent_id, &mut db_service)
                .await
        } else {
            // Reassign to backlog - require category
            let category_str = self.backlog_category.as_ref().ok_or_else(|| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "backlog_category is required when moving to backlog. Valid options: bug, feature, tech-debt",
                ))
            })?;

            let category = BacklogCategory::from_str(category_str).ok_or_else(|| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Invalid backlog category '{}'. Valid options: bug, feature, tech-debt",
                        category_str
                    ),
                ))
            })?;

            reassignment_service
                .reassign_to_backlog(&self.short_code, category, &mut db_service)
                .await
        };

        match result {
            Ok(reassignment_result) => {
                let dest_description = if let Some(parent_id) = &reassignment_result.new_parent {
                    format!("initiative {}", parent_id)
                } else {
                    format!(
                        "backlog ({})",
                        self.backlog_category.as_deref().unwrap_or("unknown")
                    )
                };

                let relative_path = reassignment_result
                    .new_path
                    .strip_prefix(metis_dir)
                    .unwrap_or(&reassignment_result.new_path)
                    .to_string_lossy();

                let output = ToolOutput::new()
                    .text(&format!(
                        "✓ {} reassigned to {}",
                        self.short_code, dest_description
                    ))
                    .text(&format!("New path: `{}`", relative_path))
                    .build_result();

                Ok(output)
            }
            Err(e) => {
                // Convert MetisError to user-friendly error result
                let error_msg = e.to_string();
                if error_msg.contains("not found") {
                    Ok(error_result(
                        "Document not found",
                        &error_msg,
                        Some("Use `list_documents` to see available documents."),
                    ))
                } else if error_msg.contains("Only tasks") {
                    Ok(error_result(
                        "Invalid document type",
                        &error_msg,
                        Some("Only task documents can be reassigned."),
                    ))
                } else if error_msg.contains("must be an initiative") {
                    Ok(error_result(
                        "Invalid parent type",
                        &error_msg,
                        Some("Provide an initiative short code as the new_parent_id."),
                    ))
                } else if error_msg.contains("phase") {
                    Ok(error_result(
                        "Invalid parent phase",
                        &error_msg,
                        Some("Transition the initiative to 'decompose' or 'active' first."),
                    ))
                } else if error_msg.contains("already at") {
                    Ok(error_result("Already assigned", &error_msg, None))
                } else if error_msg.contains("already exists") {
                    Ok(error_result(
                        "Destination exists",
                        &error_msg,
                        Some("Remove the conflicting file or choose a different parent."),
                    ))
                } else {
                    Err(CallToolError::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        error_msg,
                    )))
                }
            }
        }
    }
}
