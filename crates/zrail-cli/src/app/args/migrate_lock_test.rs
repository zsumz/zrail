//! Migration parser keeps the optional descendant authority explicit.

use std::ffi::OsString;

use super::parse;
use crate::app::args::Command;

#[test]
fn cross_revision_target_is_distinct_from_the_base() {
    let arguments = [
        "--base",
        "old-good",
        "--target",
        "fixed",
        "--output",
        "migration.json",
    ]
    .map(OsString::from);
    let Command::MigrateLock(options) = parse(&arguments).expect("parse migration bridge") else {
        panic!("expected migrate-lock command");
    };

    assert_eq!(options.base, OsString::from("old-good"));
    assert_eq!(options.target, Some(OsString::from("fixed")));
    assert_eq!(
        options.output.as_deref(),
        Some(std::path::Path::new("migration.json"))
    );
    assert!(!options.discover_base);
}

#[test]
fn base_discovery_is_read_only_and_needs_no_output() {
    let arguments = ["--discover-base", "--root", "repository"].map(OsString::from);
    let Command::MigrateLock(options) = parse(&arguments).expect("parse base discovery") else {
        panic!("expected migrate-lock command");
    };

    assert!(options.discover_base);
    assert!(options.output.is_none());
    assert_eq!(options.base, OsString::from("HEAD"));
}

#[test]
fn base_discovery_rejects_migration_authority() {
    for flag in ["--base", "--target", "--output"] {
        let arguments = ["--discover-base", flag, "value"].map(OsString::from);
        let error = parse(&arguments).expect_err("discovery must remain read-only");
        assert!(error.message.contains("cannot be combined"), "{flag}");
    }
}
