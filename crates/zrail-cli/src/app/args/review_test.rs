//! Protected review parsing never invents the untrusted proposal root.

use std::ffi::OsString;

use super::{super::Command, parse};

#[test]
fn missing_proposal_root_is_rejected() {
    let error = parse(&[
        OsString::from("--authority-root"),
        OsString::from("trusted"),
        OsString::from("--base"),
        OsString::from("HEAD"),
    ])
    .expect_err("review must not default the proposal root");

    assert_eq!(error.message, "review requires --root <proposal>");
}

#[test]
fn repeated_proposal_root_is_rejected() {
    let error = parse(&[
        OsString::from("--root"),
        OsString::from("proposal-a"),
        OsString::from("--root"),
        OsString::from("proposal-b"),
    ])
    .expect_err("review proposal authority must be singular");

    assert_eq!(error.message, "proposal root may be specified only once");
}

#[test]
fn one_explicit_proposal_root_is_preserved() {
    let command = parse(&[OsString::from("--root"), OsString::from("proposal")])
        .expect("parse explicit proposal root");
    let Command::Review(options) = command else {
        panic!("expected review command");
    };

    assert_eq!(options.common.root.to_string_lossy(), "proposal");
}
