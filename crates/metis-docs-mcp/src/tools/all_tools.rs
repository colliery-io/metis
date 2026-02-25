use super::{
    archive_document::ArchiveDocumentTool, create_document::CreateDocumentTool,
    edit_document::EditDocumentTool, index_code::IndexCodeTool,
    initialize_project::InitializeProjectTool, list_documents::ListDocumentsTool,
    read_document::ReadDocumentTool, reassign_parent::ReassignParentTool,
    search_documents::SearchDocumentsTool, transition_phase::TransitionPhaseTool,
};
use rust_mcp_sdk::tool_box;

// Generate the combined MetisTools enum
tool_box!(
    MetisTools,
    [
        InitializeProjectTool,
        ListDocumentsTool,
        SearchDocumentsTool,
        ReadDocumentTool,
        CreateDocumentTool,
        EditDocumentTool,
        TransitionPhaseTool,
        ArchiveDocumentTool,
        ReassignParentTool,
        IndexCodeTool
    ]
);
