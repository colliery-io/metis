use super::{
    hello::HelloWorldTool,
    initialize_project::InitializeProjectTool,
    list_documents::ListDocumentsTool,
    search_documents::SearchDocumentsTool,
    create_document::CreateDocumentTool,
    validate_document::ValidateDocumentTool,
    transition_phase::TransitionPhaseTool,
    check_phase_transition::CheckPhaseTransitionTool,
    validate_exit_criteria::ValidateExitCriteriaTool,
    update_document_content::UpdateDocumentContentTool,
    update_exit_criterion::UpdateExitCriterionTool,
    update_blocked_by::UpdateBlockedByTool,
};
use rust_mcp_sdk::tool_box;

// Generate the combined MetisTools enum
tool_box!(
    MetisTools,
    [
        HelloWorldTool,
        InitializeProjectTool,
        ListDocumentsTool,
        SearchDocumentsTool,
        CreateDocumentTool,
        ValidateDocumentTool,
        TransitionPhaseTool,
        CheckPhaseTransitionTool,
        ValidateExitCriteriaTool,
        UpdateDocumentContentTool,
        UpdateExitCriterionTool,
        UpdateBlockedByTool
    ]
);