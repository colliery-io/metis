//! The phase transition engine: pure functions over [`ItemTypeConfig`].
//!
//! Rules (ported from Metis 2.x semantics, generalized to configurable phase
//! lists):
//! - A transition is **forward-adjacent** by default: the target must be the
//!   immediately following phase in the type's ordered list.
//! - Leaving a phase forward requires the item's **exit criteria to be met**.
//! - `force` overrides both: it permits any phase-to-phase move (including
//!   backward or skipping) and ignores exit criteria — for corrections and
//!   admin overrides.
//!
//! No I/O: callers compute `exit_criteria_met` from the item's checklist and
//! persist the resulting phase. See METIS-T-0127.

use crate::workflow::config::ItemTypeConfig;

/// Why a requested transition is not allowed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TransitionError {
    /// The `from` or `to` phase is not part of this type's phase list.
    #[error("phase {phase:?} is not valid for this work item type")]
    UnknownPhase {
        /// The offending phase name.
        phase: String,
    },
    /// No target was given and the item is already in its terminal phase.
    #[error("already in terminal phase {phase:?}; there is no next phase")]
    NoNextPhase {
        /// The current (terminal) phase.
        phase: String,
    },
    /// The move is not forward-adjacent and `force` was not set.
    #[error("non-adjacent transition {from:?} -> {to:?} requires force")]
    NonAdjacent {
        /// Current phase.
        from: String,
        /// Requested target phase.
        to: String,
    },
    /// Exit criteria are unmet and `force` was not set.
    #[error("cannot advance from {from:?}: exit criteria not met (use force to override)")]
    ExitCriteriaNotMet {
        /// The phase being left.
        from: String,
    },
}

/// The accepted move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionOutcome {
    /// Phase moved from.
    pub from: String,
    /// Phase moved to.
    pub to: String,
}

impl ItemTypeConfig {
    /// The initial phase for newly created items of this type.
    pub fn initial_phase(&self) -> &str {
        // Validated non-empty in `ProjectConfig::validate`; default to "" only
        // if an unvalidated config slips through (no panic).
        self.phases.first().map(String::as_str).unwrap_or("")
    }

    /// Index of `phase` in the ordered list, if present.
    pub fn phase_index(&self, phase: &str) -> Option<usize> {
        self.phases.iter().position(|p| p == phase)
    }

    /// The phase immediately after `from`, if any.
    pub fn next_phase(&self, from: &str) -> Option<&str> {
        let idx = self.phase_index(from)?;
        self.phases.get(idx + 1).map(String::as_str)
    }

    /// Resolve and validate a transition.
    ///
    /// `to == None` means "advance to the next phase". `force` permits any move
    /// and skips the exit-criteria gate.
    pub fn resolve_transition(
        &self,
        from: &str,
        to: Option<&str>,
        force: bool,
        exit_criteria_met: bool,
    ) -> Result<TransitionOutcome, TransitionError> {
        let from_idx = self
            .phase_index(from)
            .ok_or_else(|| TransitionError::UnknownPhase {
                phase: from.to_string(),
            })?;

        let target = match to {
            Some(t) => t.to_string(),
            None => self
                .next_phase(from)
                .ok_or_else(|| TransitionError::NoNextPhase {
                    phase: from.to_string(),
                })?
                .to_string(),
        };

        let to_idx = self
            .phase_index(&target)
            .ok_or_else(|| TransitionError::UnknownPhase {
                phase: target.clone(),
            })?;

        if !force {
            // forward-adjacent only
            if to_idx != from_idx + 1 {
                return Err(TransitionError::NonAdjacent {
                    from: from.to_string(),
                    to: target,
                });
            }
            // exit criteria gate the forward move
            if !exit_criteria_met {
                return Err(TransitionError::ExitCriteriaNotMet {
                    from: from.to_string(),
                });
            }
        }

        Ok(TransitionOutcome {
            from: from.to_string(),
            to: target,
        })
    }
}
