//! CLI parser examples.

use std::ffi::OsString;

use super::{Command, DiffMode, InitPreset, parse};

#[test]
fn check_defaults_are_quiet_and_repository_local() {
    let command = parse([OsString::from("zrail"), OsString::from("check")]).expect("parse command");
    let Command::Check(options) = command else {
        panic!("expected check command");
    };
    assert_eq!(options.config.to_string_lossy(), "zrail.toml");
    assert_eq!(options.lock.to_string_lossy(), "zrail.lock");
}

#[test]
fn diff_requires_a_base_or_two_explicit_repository_states() {
    let error = parse([OsString::from("zrail"), OsString::from("diff")])
        .expect_err("diff should require states");
    assert!(error.message.contains("--base"));
    assert!(error.message.contains("--before"));
}

#[test]
fn explain_accepts_the_canonical_path_flag_in_any_option_order() {
    let command = parse([
        OsString::from("zrail"),
        OsString::from("explain"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--path"),
        OsString::from("src/lib.rs"),
    ])
    .expect("parse explain");
    let Command::Explain { common, path } = command else {
        panic!("expected explain command");
    };

    assert_eq!(path.to_string_lossy(), "src/lib.rs");
    assert_eq!(common.format, crate::app::output::OutputFormat::Json);
}

#[test]
fn explain_keeps_the_positional_path_compatible_without_ambiguity() {
    let command = parse([
        OsString::from("zrail"),
        OsString::from("explain"),
        OsString::from("src/lib.rs"),
    ])
    .expect("parse positional explain");
    assert!(matches!(command, Command::Explain { .. }));

    let error = parse([
        OsString::from("zrail"),
        OsString::from("explain"),
        OsString::from("src/lib.rs"),
        OsString::from("--path"),
        OsString::from("src/main.rs"),
    ])
    .expect_err("two paths are ambiguous");
    assert!(error.message.contains("only once"));
}

#[test]
fn diff_base_and_explicit_states_are_mutually_exclusive() {
    let command = parse([
        OsString::from("zrail"),
        OsString::from("diff"),
        OsString::from("--base"),
        OsString::from("HEAD"),
        OsString::from("--root"),
        OsString::from("repo"),
        OsString::from("--deny-grants"),
    ])
    .expect("parse base diff");
    let Command::Diff(options) = command else {
        panic!("expected diff command");
    };
    assert!(matches!(options.mode, DiffMode::Base { .. }));

    let error = parse([
        OsString::from("zrail"),
        OsString::from("diff"),
        OsString::from("--base"),
        OsString::from("HEAD"),
        OsString::from("--before"),
        OsString::from("before"),
    ])
    .expect_err("base and explicit states must not mix");
    assert!(error.message.contains("cannot be combined"));
}

#[test]
fn update_requires_explicit_grant_acceptance() {
    let command = parse([
        OsString::from("zrail"),
        OsString::from("update"),
        OsString::from("--accept-grants"),
    ])
    .expect("parse update command");
    let Command::Update(options) = command else {
        panic!("expected update command");
    };

    assert!(options.accept_grants);
    assert_eq!(options.base, OsString::from("HEAD"));

    let based = parse([
        OsString::from("zrail"),
        OsString::from("update"),
        OsString::from("--base"),
        OsString::from("origin/main"),
    ])
    .expect("parse protected base");
    let Command::Update(options) = based else {
        panic!("expected based update command");
    };
    assert_eq!(options.base, OsString::from("origin/main"));

    let error = parse([
        OsString::from("zrail"),
        OsString::from("check"),
        OsString::from("--accept-grants"),
    ])
    .expect_err("check must not accept update authority");
    assert!(error.message.contains("unknown option"));
}

#[test]
fn review_separates_trusted_authority_from_proposed_source() {
    let command = parse([
        OsString::from("zrail"),
        OsString::from("review"),
        OsString::from("--authority-root"),
        OsString::from("trusted"),
        OsString::from("--root"),
        OsString::from("proposal"),
        OsString::from("--base"),
        OsString::from("protected"),
        OsString::from("--deny-grants"),
    ])
    .expect("parse protected review");
    let Command::Review(options) = command else {
        panic!("expected review command");
    };

    assert_eq!(options.authority_root.to_string_lossy(), "trusted");
    assert_eq!(options.common.root.to_string_lossy(), "proposal");
    assert_eq!(options.base, OsString::from("protected"));
    assert!(options.deny_grants);
}

#[test]
fn init_defaults_to_zsumz_without_baseline() {
    let command =
        parse([OsString::from("zrail"), OsString::from("init")]).expect("parse strict default");
    let Command::Init(options) = command else {
        panic!("expected init command");
    };
    assert_eq!(options.preset, InitPreset::Zsumz);
    assert!(!options.baseline);
}

#[test]
fn init_accepts_a_rust_preset_with_baseline_in_any_order() {
    let command = parse([
        OsString::from("zrail"),
        OsString::from("init"),
        OsString::from("--preset"),
        OsString::from("rust"),
        OsString::from("repository"),
        OsString::from("--baseline"),
    ])
    .expect("parse baseline mode");
    let Command::Init(options) = command else {
        panic!("expected init command");
    };
    assert_eq!(options.root.to_string_lossy(), "repository");
    assert_eq!(options.preset, InitPreset::Rust);
    assert!(options.baseline);
}

#[test]
fn init_rejects_unknown_or_repeated_preset_authority() {
    let unknown = parse([
        OsString::from("zrail"),
        OsString::from("init"),
        OsString::from("--preset"),
        OsString::from("house"),
    ])
    .expect_err("unknown presets must fail");
    assert!(unknown.message.contains("zsumz"));
    assert!(unknown.message.contains("rust"));

    let duplicate = parse([
        OsString::from("zrail"),
        OsString::from("init"),
        OsString::from("--preset"),
        OsString::from("zsumz"),
        OsString::from("--preset"),
        OsString::from("rust"),
    ])
    .expect_err("preset authority must be singular");
    assert!(duplicate.message.contains("only once"));

    let legacy = parse([
        OsString::from("zrail"),
        OsString::from("init"),
        OsString::from("--strict"),
    ])
    .expect_err("unreleased mode spelling should be removed");
    assert!(legacy.message.contains("unknown option"));

    let repeated_baseline = parse([
        OsString::from("zrail"),
        OsString::from("init"),
        OsString::from("--baseline"),
        OsString::from("--baseline"),
    ])
    .expect_err("baseline authority must be singular");
    assert!(repeated_baseline.message.contains("only once"));

    let missing = parse([
        OsString::from("zrail"),
        OsString::from("init"),
        OsString::from("--preset"),
    ])
    .expect_err("preset value is required");
    assert!(missing.message.contains("requires a value"));
}
