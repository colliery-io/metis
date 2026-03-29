use crate::formatting::ToolOutput;
use metis_core::{
    application::services::workspace::{PhaseTransitionService, WorkspaceDetectionService},
    domain::documents::types::{DocumentType, Phase},
};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "transition_phase",
    description = "Transition a document to a new phase using its short code (e.g., PROJ-V-0001). If phase is not provided, transitions to the next valid phase automatically. IMPORTANT: You can only transition to adjacent phases - you cannot skip phases (e.g., todo->completed is invalid; must go todo->active->completed).",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransitionPhaseTool {
    /// Path to the .metis folder (e.g., "/Users/me/my-project/.metis"). Must end with .metis
    pub project_path: String,
    /// Document short code (e.g., PROJ-V-0001) to identify the document
    pub short_code: String,
    /// Phase to transition to (optional - if not provided, transitions to next phase)
    pub phase: Option<String>,
    /// Force transition even if exit criteria aren't met
    pub force: Option<bool>,
}

impl TransitionPhaseTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let metis_dir = Path::new(&self.project_path);

        // Prepare workspace (validates, creates/updates database, syncs)
        let detection_service = WorkspaceDetectionService::new();
        let _db = detection_service
            .prepare_workspace(metis_dir)
            .await
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        // Create the phase transition service
        let transition_service = PhaseTransitionService::new(metis_dir);

        // Perform the transition using short code directly
        let result = if let Some(phase_str) = &self.phase {
            // Transition to specific phase
            let target_phase = self.parse_phase(phase_str)?;
            transition_service
                .transition_document(&self.short_code, target_phase)
                .await
                .map_err(|e| CallToolError::new(e))?
        } else {
            // Auto-transition to next phase
            transition_service
                .transition_to_next_phase(&self.short_code)
                .await
                .map_err(|e| CallToolError::new(e))?
        };

        // Get phase progression for visual display
        let doc_type_str = result.document_type.to_string();
        let phases = self.get_phase_sequence(&doc_type_str);
        let current_index = phases
            .iter()
            .position(|p| *p == result.to_phase.to_string())
            .unwrap_or(0);

        let phase_strs: Vec<&str> = phases.iter().map(|s| s.as_str()).collect();

        // Get phase-specific guidance for what to do in the new phase
        let guidance = self.phase_guidance(&doc_type_str, &result.to_phase.to_string());

        let mut output = ToolOutput::new()
            .header("Phase Transition")
            .text(&format!(
                "{}: {} -> {}",
                self.short_code, result.from_phase, result.to_phase
            ))
            .blank()
            .phase_progress(&phase_strs, current_index);

        if let Some(guidance_text) = guidance {
            output = output.blank().text(guidance_text);
        }

        Ok(output.build_result())
    }

    fn parse_phase(&self, phase_str: &str) -> Result<Phase, CallToolError> {
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
            "design" => Ok(Phase::Design),
            "ready" => Ok(Phase::Ready),
            "decompose" => Ok(Phase::Decompose),
            "discovery" => Ok(Phase::Discovery),
            "drafting" => Ok(Phase::Drafting),
            _ => Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unknown phase: {}", phase_str),
            ))),
        }
    }

    fn phase_guidance(&self, document_type: &str, phase: &str) -> Option<&'static str> {
        match (document_type, phase) {
            // Vision phases
            ("vision", "draft") => Some(
                "**Draft phase**: Define the purpose, core values, long-term vision, and success criteria. This is the foundation — take time to get it right."
            ),
            ("vision", "review") => Some(
                "**Review phase**: The vision is ready for stakeholder review. Verify that purpose resonates, values are actionable, and success is measurable. Get explicit sign-off before publishing."
            ),
            ("vision", "published") => Some(
                "**Published**: This vision is now the authoritative strategic direction. Initiatives can be created under it. Changes should be rare and deliberate."
            ),

            // Initiative phases
            ("initiative", "discovery") => Some(
                "**Discovery phase**: Understand the problem space. Ask clarifying questions about scope, priorities, and constraints. Document context, goals, and non-goals. Do NOT assume you understand the full picture — ask the human."
            ),
            ("initiative", "design") => Some(
                "**Design phase**: Define the technical approach. Present multiple options with trade-offs. Document architecture, detailed design, and alternatives considered. Get human approval on the approach before proceeding."
            ),
            ("initiative", "ready") => Some(
                "**Ready phase**: The design is approved and the initiative is ready for decomposition. Review that all design decisions are documented and the implementation plan is clear. Get human sign-off before decomposing."
            ),
            ("initiative", "decompose") => Some(
                "**Decompose phase**: Break the initiative into discrete, actionable tasks. Each task should be independently completable (1-14 days). After creating tasks, open them for review with `open_document` (include_children: true) and get human approval before moving to active."
            ),
            ("initiative", "active") => Some(
                "**Active phase**: Tasks are being executed. Track progress by updating active tasks regularly. This initiative should not be transitioned to completed until ALL child tasks are done."
            ),
            ("initiative", "completed") => Some(
                "**Completed**: All work under this initiative is done. Consider archiving if no longer needed for reference."
            ),

            // Task phases
            ("task", "todo") => Some(
                "**Todo phase**: Task is defined and ready to be picked up. Read the task thoroughly before starting — understand the objective, acceptance criteria, and dependencies."
            ),
            ("task", "active") => Some(
                "**Active phase**: You are now working on this task. Update the Status Updates section regularly with progress, findings, decisions, and plan changes. This is your working memory — if context is lost, this is how you (or another agent) picks up where you left off."
            ),
            ("task", "blocked") => Some(
                "**Blocked**: This task cannot proceed. Document what is blocking it in the Status Updates section. Include what you've tried and what needs to happen to unblock."
            ),
            ("task", "completed") => Some(
                "**Completed**: Task is done. Verify all acceptance criteria are met before considering this truly finished."
            ),

            // ADR phases
            ("adr", "draft") => Some(
                "**Draft phase**: Document the decision context, the options considered, and the proposed decision. Be thorough — ADRs are the historical record of why decisions were made."
            ),
            ("adr", "discussion") => Some(
                "**Discussion phase**: The ADR is open for review and debate. Gather feedback, document concerns, and refine the decision. Do not finalize without stakeholder input."
            ),
            ("adr", "decided") => Some(
                "**Decided**: This decision is now in effect. Implementation should follow the decision as documented. If circumstances change, create a new ADR that supersedes this one."
            ),

            // Specification phases
            ("specification", "discovery") => Some(
                "**Discovery phase**: Gather requirements and understand what needs to be specified. Interview stakeholders, review existing documentation, and identify scope."
            ),
            ("specification", "drafting") => Some(
                "**Drafting phase**: Write the specification content. Be precise and complete — this document will guide implementation."
            ),
            ("specification", "review") => Some(
                "**Review phase**: The specification is ready for technical and stakeholder review. Address feedback and refine until approved."
            ),
            ("specification", "published") => Some(
                "**Published**: This specification is the authoritative reference. It remains a living document — update it as the system evolves, but changes should be reviewed."
            ),

            _ => None,
        }
    }

    fn get_phase_sequence(&self, document_type: &str) -> Vec<String> {
        // Use DocumentType::phase_sequence() - the single source of truth
        let doc_type = match document_type {
            "vision" => Some(DocumentType::Vision),
            "initiative" => Some(DocumentType::Initiative),
            "task" => Some(DocumentType::Task),
            "adr" => Some(DocumentType::Adr),
            "specification" => Some(DocumentType::Specification),
            _ => None,
        };

        match doc_type {
            Some(dt) => dt.phase_sequence().iter().map(|p| p.to_string()).collect(),
            None => vec!["unknown".to_string()],
        }
    }
}
