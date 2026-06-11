//! Unit-level tests for the workflow engine: default config validity, legal and
//! illegal transitions (including the ADR supersede flow), config validation
//! errors, and short-code round-trips.

use metis_core::workflow::config::{ConfigError, ItemTypeConfig, ProjectConfig};
use metis_core::workflow::short_code::{format_short_code, parse_short_code, ShortCode};
use metis_core::workflow::transition::TransitionError;
use std::collections::BTreeMap;

fn flight_levels() -> ProjectConfig {
    ProjectConfig::default_flight_levels()
}

#[test]
fn default_config_is_valid() {
    flight_levels()
        .validate()
        .expect("default config validates");
}

#[test]
fn default_config_has_expected_types_and_letters() {
    let cfg = flight_levels();
    let letters: BTreeMap<&str, &str> = cfg
        .types
        .iter()
        .map(|(k, v)| (k.as_str(), v.letter.as_str()))
        .collect();
    assert_eq!(letters["vision"], "V");
    assert_eq!(letters["initiative"], "I");
    assert_eq!(letters["task"], "T");
    assert_eq!(letters["adr"], "A");
    assert_eq!(letters["specification"], "S");
    assert_eq!(letters["bug"], "B");
    assert_eq!(letters["feature"], "F");
    assert_eq!(letters["chore"], "C");
}

#[test]
fn initiative_initial_phase_and_next() {
    let cfg = flight_levels();
    let init = cfg.item_type("initiative").unwrap();
    assert_eq!(init.initial_phase(), "discovery");
    assert_eq!(init.next_phase("discovery"), Some("design"));
    assert_eq!(init.next_phase("completed"), None);
}

#[test]
fn legal_forward_transition_with_criteria_met() {
    let cfg = flight_levels();
    let task = cfg.item_type("task").unwrap();
    let out = task
        .resolve_transition("todo", Some("active"), false, true)
        .expect("legal forward move");
    assert_eq!(out.from, "todo");
    assert_eq!(out.to, "active");
}

#[test]
fn omitted_target_advances_to_next() {
    let cfg = flight_levels();
    let task = cfg.item_type("task").unwrap();
    let out = task
        .resolve_transition("backlog", None, false, true)
        .unwrap();
    assert_eq!(out.to, "todo");
}

#[test]
fn forward_blocked_when_criteria_unmet() {
    let cfg = flight_levels();
    let task = cfg.item_type("task").unwrap();
    let err = task
        .resolve_transition("active", Some("completed"), false, false)
        .unwrap_err();
    assert_eq!(
        err,
        TransitionError::ExitCriteriaNotMet {
            from: "active".into()
        }
    );
}

#[test]
fn non_adjacent_requires_force() {
    let cfg = flight_levels();
    let task = cfg.item_type("task").unwrap();
    // skip todo+active
    let err = task
        .resolve_transition("backlog", Some("completed"), false, true)
        .unwrap_err();
    assert_eq!(
        err,
        TransitionError::NonAdjacent {
            from: "backlog".into(),
            to: "completed".into()
        }
    );
}

#[test]
fn force_allows_skip_and_ignores_criteria() {
    let cfg = flight_levels();
    let task = cfg.item_type("task").unwrap();
    let out = task
        .resolve_transition("backlog", Some("completed"), true, false)
        .expect("force overrides adjacency + criteria");
    assert_eq!(out.to, "completed");
}

#[test]
fn force_allows_backward_move() {
    let cfg = flight_levels();
    let init = cfg.item_type("initiative").unwrap();
    let out = init
        .resolve_transition("active", Some("design"), true, false)
        .expect("force allows backward");
    assert_eq!(out.to, "design");
}

#[test]
fn terminal_phase_has_no_next() {
    let cfg = flight_levels();
    let task = cfg.item_type("task").unwrap();
    let err = task
        .resolve_transition("completed", None, false, true)
        .unwrap_err();
    assert_eq!(
        err,
        TransitionError::NoNextPhase {
            phase: "completed".into()
        }
    );
}

#[test]
fn unknown_phase_is_rejected() {
    let cfg = flight_levels();
    let task = cfg.item_type("task").unwrap();
    let err = task
        .resolve_transition("nope", Some("todo"), false, true)
        .unwrap_err();
    assert_eq!(
        err,
        TransitionError::UnknownPhase {
            phase: "nope".into()
        }
    );
}

#[test]
fn adr_supersede_flow() {
    let cfg = flight_levels();
    let adr = cfg.item_type("adr").unwrap();
    assert_eq!(adr.initial_phase(), "draft");
    // walk the full linear flow with criteria met
    for (from, to) in [
        ("draft", "discussion"),
        ("discussion", "decided"),
        ("decided", "superseded"),
    ] {
        let out = adr.resolve_transition(from, None, false, true).unwrap();
        assert_eq!(out.to, to, "{from} should advance to {to}");
    }
}

#[test]
fn validate_rejects_empty() {
    let cfg = ProjectConfig {
        types: BTreeMap::new(),
    };
    assert_eq!(cfg.validate().unwrap_err(), ConfigError::Empty);
}

#[test]
fn validate_rejects_duplicate_letters() {
    let mut types = BTreeMap::new();
    let mk = |letter: &str| ItemTypeConfig {
        display_name: "X".into(),
        letter: letter.into(),
        phases: vec!["a".into(), "b".into()],
        template: String::new(),
        allowed_parents: vec![],
        allow_root: true,
    };
    types.insert("one".to_string(), mk("X"));
    types.insert("two".to_string(), mk("X"));
    let err = ProjectConfig { types }.validate().unwrap_err();
    assert!(matches!(err, ConfigError::DuplicateLetter { .. }));
}

#[test]
fn validate_rejects_empty_phases() {
    let mut types = BTreeMap::new();
    types.insert(
        "thing".to_string(),
        ItemTypeConfig {
            display_name: "Thing".into(),
            letter: "T".into(),
            phases: vec![],
            template: String::new(),
            allowed_parents: vec![],
            allow_root: true,
        },
    );
    assert_eq!(
        ProjectConfig { types }.validate().unwrap_err(),
        ConfigError::EmptyPhases("thing".into())
    );
}

#[test]
fn validate_rejects_unknown_parent() {
    let mut types = BTreeMap::new();
    types.insert(
        "task".to_string(),
        ItemTypeConfig {
            display_name: "Task".into(),
            letter: "T".into(),
            phases: vec!["todo".into(), "done".into()],
            template: String::new(),
            allowed_parents: vec!["ghost".into()],
            allow_root: false,
        },
    );
    let err = ProjectConfig { types }.validate().unwrap_err();
    assert_eq!(
        err,
        ConfigError::UnknownParent {
            ty: "task".into(),
            parent: "ghost".into()
        }
    );
}

#[test]
fn validate_rejects_unreachable_type() {
    let mut types = BTreeMap::new();
    types.insert(
        "orphan".to_string(),
        ItemTypeConfig {
            display_name: "Orphan".into(),
            letter: "O".into(),
            phases: vec!["a".into()],
            template: String::new(),
            allowed_parents: vec![],
            allow_root: false,
        },
    );
    assert_eq!(
        ProjectConfig { types }.validate().unwrap_err(),
        ConfigError::Unreachable("orphan".into())
    );
}

#[test]
fn validate_rejects_bad_type_name() {
    let mut types = BTreeMap::new();
    types.insert(
        "Bad Name".to_string(),
        ItemTypeConfig {
            display_name: "Bad".into(),
            letter: "B".into(),
            phases: vec!["a".into()],
            template: String::new(),
            allowed_parents: vec![],
            allow_root: true,
        },
    );
    assert_eq!(
        ProjectConfig { types }.validate().unwrap_err(),
        ConfigError::InvalidTypeName("Bad Name".into())
    );
}

#[test]
fn short_code_round_trips() {
    let s = format_short_code("METIS", "T", 42);
    assert_eq!(s, "METIS-T-0042");
    assert_eq!(
        parse_short_code(&s).unwrap(),
        ShortCode {
            prefix: "METIS".into(),
            letter: "T".into(),
            seq: 42
        }
    );
}

#[test]
fn short_code_handles_hyphenated_prefix() {
    let parsed = parse_short_code("my-project-I-0007").unwrap();
    assert_eq!(parsed.prefix, "my-project");
    assert_eq!(parsed.letter, "I");
    assert_eq!(parsed.seq, 7);
}

#[test]
fn short_code_rejects_malformed() {
    for bad in [
        "",
        "METIS",
        "METIS-T",
        "METIS-T-xx",
        "METIS-toolong-0001",
        "-T-0001",
    ] {
        assert!(
            parse_short_code(bad).is_err(),
            "{bad:?} should be malformed"
        );
    }
}

#[test]
fn config_json_round_trips() {
    let cfg = flight_levels();
    let json = serde_json::to_string(&cfg).unwrap();
    let back: ProjectConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(cfg, back);
}
