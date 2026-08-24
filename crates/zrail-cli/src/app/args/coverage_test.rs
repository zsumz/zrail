//! Governed-surface coverage argument examples.

use std::ffi::OsString;

use crate::app::{args::Command, output::OutputFormat};

#[test]
fn coverage_accepts_repository_config_and_json_format() {
    let command = crate::app::args::parse([
        OsString::from("zrail"),
        OsString::from("coverage"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--config"),
        OsString::from("policy/zrail.toml"),
        OsString::from("--root"),
        OsString::from("repository"),
    ])
    .expect("parse coverage");
    let Command::Coverage(options) = command else {
        panic!("expected coverage command");
    };

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.root.to_string_lossy(), "repository");
    assert_eq!(options.config.to_string_lossy(), "policy/zrail.toml");
}

#[test]
fn coverage_defaults_to_a_human_repository_local_report() {
    let command = crate::app::args::parse([OsString::from("zrail"), OsString::from("coverage")])
        .expect("parse coverage defaults");
    let Command::Coverage(options) = command else {
        panic!("expected coverage command");
    };

    assert_eq!(options.format, OutputFormat::Human);
    assert_eq!(options.root.to_string_lossy(), ".");
    assert_eq!(options.config.to_string_lossy(), "zrail.toml");
}
