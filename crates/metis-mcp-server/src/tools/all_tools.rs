use super::document::{CreateDocumentTool, ValidateDocumentTool};
use super::obsidian::OpenVaultInObsidianTool;
use super::phase::{CheckPhaseTransitionTool, TransitionPhaseTool, ValidateExitCriteriaTool};
use super::project::InitializeProjectTool;
use super::query::{ListDocumentsTool, SearchDocumentsTool};
use super::update::{UpdateBlockedByTool, UpdateDocumentContentTool, UpdateExitCriterionTool};
use rust_mcp_sdk::tool_box;

// Generate the combined MetisTools enum
tool_box!(
    MetisTools,
    [
        InitializeProjectTool,
        CreateDocumentTool,
        ValidateDocumentTool,
        UpdateDocumentContentTool,
        UpdateExitCriterionTool,
        UpdateBlockedByTool,
        TransitionPhaseTool,
        CheckPhaseTransitionTool,
        ValidateExitCriteriaTool,
        ListDocumentsTool,
        SearchDocumentsTool,
        OpenVaultInObsidianTool
    ]
);
