//! Configurable diagnostic-retention argument examples.

use std::ffi::OsString;

use zrail_core::DiagnosticLimit;

use super::super::{Command, parse};

#[test]
fn diagnostic_limits_are_available_on_diagnostic_commands() {
    let cases = [
        ("check", "0", DiagnosticLimit::Bounded(0)),
        ("doctor", "50000", DiagnosticLimit::Bounded(50_000)),
    ];
    for (name, value, expected) in cases {
        let command = parse([
            OsString::from("zrail"),
            OsString::from(name),
            OsString::from("--limit"),
            OsString::from(value),
        ])
        .expect("parse diagnostic limit");
        let (Command::Check(common) | Command::Doctor(common)) = command else {
            panic!("expected common diagnostic command");
        };
        assert_eq!(common.limit, expected);
    }

    let Command::Explain { common, .. } = parse([
        OsString::from("zrail"),
        OsString::from("explain"),
        OsString::from("--path"),
        OsString::from("src/lib.rs"),
        OsString::from("--limit"),
        OsString::from("all"),
    ])
    .expect("parse explain limit") else {
        panic!("expected explain command");
    };
    assert_eq!(common.limit, DiagnosticLimit::All);

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
