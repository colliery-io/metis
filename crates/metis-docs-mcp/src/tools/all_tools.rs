use super::{
    archive_document::ArchiveDocumentTool, create_document::CreateDocumentTool,
    initialize_project::InitializeProjectTool, list_documents::ListDocumentsTool,
    search_documents::SearchDocumentsTool, transition_phase::TransitionPhaseTool,
    update_blocked_by::UpdateBlockedByTool, update_document_content::UpdateDocumentContentTool,
    update_exit_criterion::UpdateExitCriterionTool, validate_document::ValidateDocumentTool,
    validate_exit_criteria::ValidateExitCriteriaTool,
};
use rust_mcp_sdk::tool_box;

// Generate the combined MetisTools enum
tool_box!(
    MetisTools,
    [
        InitializeProjectTool,
        ListDocumentsTool,
        SearchDocumentsTool,
        CreateDocumentTool,
        ValidateDocumentTool,
        TransitionPhaseTool,
        ValidateExitCriteriaTool,
        UpdateDocumentContentTool,
        UpdateExitCriterionTool,
        UpdateBlockedByTool,
        ArchiveDocumentTool
    ]
);
