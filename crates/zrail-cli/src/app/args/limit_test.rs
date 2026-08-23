//! Configurable diagnostic-retention argument examples.

use std::ffi::OsString;

use zrail_core::DiagnosticLimit;

use super::super::{Command, parse};

#[test]
fn diagnostic_limits_are_available_on_diagnostic_commands() {
    let Command::Check(common) = parse([
        OsString::from("zrail"),
        OsString::from("check"),
        OsString::from("--limit"),
        OsString::from("0"),
    ])
    .expect("parse check limit") else {
        panic!("expected check command");
    };
    assert_eq!(common.limit, DiagnosticLimit::Bounded(0));

    let Command::Review(options) = parse([
        OsString::from("zrail"),
        OsString::from("review"),
        OsString::from("--root"),
        OsString::from("proposal"),
        OsString::from("--limit"),
        OsString::from("10000"),
    ])
    .expect("parse review limit") else {
        panic!("expected review command");
    };
    assert_eq!(options.common.limit, DiagnosticLimit::Bounded(10_000));
}

#[test]
fn commands_without_finding_payloads_reject_diagnostic_limits() {
    for arguments in [
        vec!["doctor", "--limit", "50000"],
        vec!["explain", "--path", "src/lib.rs", "--limit", "all"],
    ] {
        let arguments = std::iter::once(OsString::from("zrail"))
            .chain(arguments.into_iter().map(OsString::from));
        let error = parse(arguments).expect_err("inert limit must be rejected");
        assert!(error.message.contains("unknown option \"--limit\""));
    }
}

#[test]
fn diagnostic_limits_reject_negative_and_unknown_values() {
    for value in ["-1", "everything"] {
        let error = parse([
            OsString::from("zrail"),
            OsString::from("check"),
            OsString::from("--limit"),
            OsString::from(value),
        ])
        .expect_err("invalid limit must fail closed");
        assert!(error.message.contains("diagnostic limit"));
    }
}
