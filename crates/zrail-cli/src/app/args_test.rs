//! CLI parser examples.

use std::ffi::OsString;

use super::{Command, DiffMode, InitMode, parse};

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

    let error = parse([
        OsString::from("zrail"),
        OsString::from("check"),
        OsString::from("--accept-grants"),
    ])
    .expect_err("check must not accept update authority");
    assert!(error.message.contains("unknown option"));
}

#[test]
fn init_defaults_to_strict_and_accepts_one_explicit_mode() {
    let command =
        parse([OsString::from("zrail"), OsString::from("init")]).expect("parse strict default");
    let Command::Init(options) = command else {
        panic!("expected init command");
    };
    assert_eq!(options.mode, InitMode::Strict);

    let command = parse([
        OsString::from("zrail"),
        OsString::from("init"),
        OsString::from("repository"),
        OsString::from("--baseline"),
    ])
    .expect("parse baseline mode");
    let Command::Init(options) = command else {
        panic!("expected init command");
    };
    assert_eq!(options.root.to_string_lossy(), "repository");
    assert_eq!(options.mode, InitMode::Baseline);

    let error = parse([
        OsString::from("zrail"),
        OsString::from("init"),
        OsString::from("--strict"),
        OsString::from("--baseline"),
    ])
    .expect_err("init modes must be exclusive");
    assert!(error.message.contains("init mode"));
}
