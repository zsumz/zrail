//! Explain-specific CLI parser examples.

use std::ffi::OsString;

use crate::app::args::{Command, parse};

#[test]
fn canonical_path_flag_accepts_any_option_order() {
    let command = parse([
        OsString::from("zrail"),
        OsString::from("explain"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--path"),
        OsString::from("src/lib.rs"),
    ])
    .expect("parse explain");
    let Command::Explain {
        common,
        path,
        hypothetical,
    } = command
    else {
        panic!("expected explain command");
    };

    assert_eq!(path.to_string_lossy(), "src/lib.rs");
    assert!(!hypothetical);
    assert_eq!(common.format, crate::app::output::OutputFormat::Json);
}

#[test]
fn positional_path_remains_compatible_without_ambiguity() {
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
fn hypothetical_path_is_explicit() {
    let command = parse([
        OsString::from("zrail"),
        OsString::from("explain"),
        OsString::from("--hypothetical-path"),
        OsString::from("src/future.rs"),
    ])
    .expect("parse hypothetical explanation");
    let Command::Explain {
        path, hypothetical, ..
    } = command
    else {
        panic!("expected explain command");
    };
    assert_eq!(path.to_string_lossy(), "src/future.rs");
    assert!(hypothetical);
}

#[test]
fn concrete_and_hypothetical_paths_are_mutually_exclusive() {
    let error = parse([
        OsString::from("zrail"),
        OsString::from("explain"),
        OsString::from("--path"),
        OsString::from("src/lib.rs"),
        OsString::from("--hypothetical-path"),
        OsString::from("src/future.rs"),
    ])
    .expect_err("path modes must not mix");
    assert!(error.message.contains("only once"));
}
