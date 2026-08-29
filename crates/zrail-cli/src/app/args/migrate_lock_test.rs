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
}
