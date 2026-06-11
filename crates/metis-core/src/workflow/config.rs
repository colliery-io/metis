//! Project configuration: the set of work item types and their workflows.
//!
//! In Metis 3.0 the rigid Vision→Initiative→Task hierarchy becomes *the default
//! configuration* rather than a hard-coded constraint. A project's `config`
//! column (see `schema::projects`) stores a [`ProjectConfig`] as JSON: a map of
//! work item types, each defining its display name, short-code letter, ordered
//! phase list, body template, and which types may parent it.
//!
//! This is the deliberate stop short of JIRA's custom-field machinery: types
//! and ordered phases, plus a terminal-supersede concept expressed simply as
//! the last phase in a sequence. No conditional transitions, no per-phase
//! permissions, no screen schemes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A project's work item type definitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Work item types, keyed by lowercase type name (e.g. `"task"`).
    pub types: BTreeMap<String, ItemTypeConfig>,
}

/// One work item type and its workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemTypeConfig {
    /// Human-facing name, e.g. `"Task"`.
    pub display_name: String,
    /// Short-code letter(s): 1–2 uppercase ASCII letters, e.g. `"T"`, unique
    /// across the project. Used in `{PREFIX}-{LETTER}-{NNNN}` codes.
    pub letter: String,
    /// Ordered phase names. Non-empty; the first is the initial phase, the last
    /// is terminal. Forward transitions move one step along this list.
    pub phases: Vec<String>,
    /// Markdown body template seeded into new items of this type.
    #[serde(default)]
    pub template: String,
    /// Type names permitted as a parent of items of this type.
    #[serde(default)]
    pub allowed_parents: Vec<String>,
    /// Whether items of this type may exist with no parent (a root/standalone
    /// item, e.g. a backlog task or a top-level vision).
    #[serde(default)]
    pub allow_root: bool,
}

/// Why a [`ProjectConfig`] is invalid.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    /// No work item types were defined.
    #[error("project config defines no work item types")]
    Empty,
    /// A type key is empty or not lowercase `[a-z][a-z0-9_]*`.
    #[error("invalid type name {0:?} (must match [a-z][a-z0-9_]*)")]
    InvalidTypeName(String),
    /// A short-code letter is not 1–2 uppercase ASCII letters.
    #[error("type {ty:?}: invalid short-code letter {letter:?} (must match [A-Z]{{1,2}})")]
    InvalidLetter {
        /// The offending type name.
        ty: String,
        /// The offending letter value.
        letter: String,
    },
    /// Two types share a short-code letter.
    #[error("duplicate short-code letter {letter:?} on types {a:?} and {b:?}")]
    DuplicateLetter {
        /// The duplicated letter.
        letter: String,
        /// First type using it.
        a: String,
        /// Second type using it.
        b: String,
    },
    /// A type has an empty phase list.
    #[error("type {0:?} has an empty phase list")]
    EmptyPhases(String),
    /// A type lists an empty or duplicate phase.
    #[error("type {ty:?} has an invalid or duplicate phase {phase:?}")]
    InvalidPhase {
        /// The type name.
        ty: String,
        /// The offending phase name.
        phase: String,
    },
    /// A type references a parent type that is not defined.
    #[error("type {ty:?} lists unknown parent type {parent:?}")]
    UnknownParent {
        /// The type with the bad reference.
        ty: String,
        /// The missing parent type.
        parent: String,
    },
    /// A type can be neither a root nor a child of anything — it could never be
    /// created.
    #[error("type {0:?} is unreachable: allow_root is false and allowed_parents is empty")]
    Unreachable(String),
}

impl ProjectConfig {
    /// Look up a type by name.
    pub fn item_type(&self, name: &str) -> Option<&ItemTypeConfig> {
        self.types.get(name)
    }

    /// Validate the whole configuration. Returns the first problem found.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.types.is_empty() {
            return Err(ConfigError::Empty);
        }

        // type names + per-type internals
        let mut letters: BTreeMap<String, String> = BTreeMap::new();
        for (name, ty) in &self.types {
            if !is_ident(name) {
                return Err(ConfigError::InvalidTypeName(name.clone()));
            }
            if !is_letter(&ty.letter) {
                return Err(ConfigError::InvalidLetter {
                    ty: name.clone(),
                    letter: ty.letter.clone(),
                });
            }
            if let Some(prev) = letters.insert(ty.letter.clone(), name.clone()) {
                return Err(ConfigError::DuplicateLetter {
                    letter: ty.letter.clone(),
                    a: prev,
                    b: name.clone(),
                });
            }
            if ty.phases.is_empty() {
                return Err(ConfigError::EmptyPhases(name.clone()));
            }
            let mut seen = std::collections::BTreeSet::new();
            for phase in &ty.phases {
                if !is_ident(phase) || !seen.insert(phase.clone()) {
                    return Err(ConfigError::InvalidPhase {
                        ty: name.clone(),
                        phase: phase.clone(),
                    });
                }
            }
        }

        // cross-type references (second pass, all names known)
        for (name, ty) in &self.types {
            for parent in &ty.allowed_parents {
                if !self.types.contains_key(parent) {
                    return Err(ConfigError::UnknownParent {
                        ty: name.clone(),
                        parent: parent.clone(),
                    });
                }
            }
            if !ty.allow_root && ty.allowed_parents.is_empty() {
                return Err(ConfigError::Unreachable(name.clone()));
            }
        }

        Ok(())
    }

    /// The built-in Flight Levels configuration: the five classic Metis
    /// document types plus general-purpose `bug`/`feature`/`chore` task-like
    /// types. This is what a project gets unless it supplies its own config.
    pub fn default_flight_levels() -> Self {
        let mut types = BTreeMap::new();

        types.insert(
            "vision".to_string(),
            ItemTypeConfig {
                display_name: "Vision".to_string(),
                letter: "V".to_string(),
                phases: vec!["draft".into(), "review".into(), "published".into()],
                template: "# {title}\n\n## Purpose\n\n## Long-term Vision\n".to_string(),
                allowed_parents: vec![],
                allow_root: true,
            },
        );
        types.insert(
            "initiative".to_string(),
            ItemTypeConfig {
                display_name: "Initiative".to_string(),
                letter: "I".to_string(),
                phases: vec![
                    "discovery".into(),
                    "design".into(),
                    "ready".into(),
                    "decompose".into(),
                    "active".into(),
                    "completed".into(),
                ],
                template: "# {title}\n\n## Context\n\n## Goals & Non-Goals\n".to_string(),
                allowed_parents: vec!["vision".into()],
                allow_root: true,
            },
        );
        types.insert(
            "task".to_string(),
            ItemTypeConfig {
                display_name: "Task".to_string(),
                letter: "T".to_string(),
                phases: vec![
                    "backlog".into(),
                    "todo".into(),
                    "active".into(),
                    "completed".into(),
                ],
                template: "# {title}\n\n## Objective\n\n## Acceptance Criteria\n".to_string(),
                allowed_parents: vec!["initiative".into()],
                allow_root: true,
            },
        );
        types.insert(
            "adr".to_string(),
            ItemTypeConfig {
                display_name: "ADR".to_string(),
                letter: "A".to_string(),
                // decided -> superseded is the supersede semantics, expressed as
                // the terminal step of the linear sequence.
                phases: vec![
                    "draft".into(),
                    "discussion".into(),
                    "decided".into(),
                    "superseded".into(),
                ],
                template: "# {title}\n\n## Context\n\n## Decision\n\n## Consequences\n".to_string(),
                allowed_parents: vec![],
                allow_root: true,
            },
        );
        types.insert(
            "specification".to_string(),
            ItemTypeConfig {
                display_name: "Specification".to_string(),
                letter: "S".to_string(),
                phases: vec![
                    "discovery".into(),
                    "drafting".into(),
                    "review".into(),
                    "published".into(),
                ],
                template: "# {title}\n\n## Overview\n\n## Requirements\n".to_string(),
                allowed_parents: vec![],
                allow_root: true,
            },
        );

        // General-purpose task-like types — same lifecycle as task.
        for (name, display, letter) in [
            ("bug", "Bug", "B"),
            ("feature", "Feature", "F"),
            ("chore", "Chore", "C"),
        ] {
            types.insert(
                name.to_string(),
                ItemTypeConfig {
                    display_name: display.to_string(),
                    letter: letter.to_string(),
                    phases: vec![
                        "backlog".into(),
                        "todo".into(),
                        "active".into(),
                        "completed".into(),
                    ],
                    template: format!("# {{title}}\n\n## {display}\n"),
                    allowed_parents: vec!["initiative".into()],
                    allow_root: true,
                },
            );
        }

        ProjectConfig { types }
    }
}

/// `[a-z][a-z0-9_]*`
fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// 1–2 uppercase ASCII letters.
fn is_letter(s: &str) -> bool {
    !s.is_empty() && s.len() <= 2 && s.chars().all(|c| c.is_ascii_uppercase())
}
