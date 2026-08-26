//! Test-mirror CLI argument examples.

use std::{ffi::OsString, path::PathBuf};

use crate::app::{
    args::{Command, MirrorsAction},
    output::OutputFormat,
};

#[test]
fn parses_json_plan_and_exact_verify_plan() {
    let plan = crate::app::args::parse([
        OsString::from("zrail"),
        OsString::from("mirrors"),
        OsString::from("plan"),
        OsString::from("--format"),
        OsString::from("json"),
    ])
    .expect("parse mirror plan");
    let Command::Mirrors(plan) = plan else {
        panic!("expected mirrors command");
    };
    assert_eq!(plan.action, MirrorsAction::Plan);
    assert_eq!(plan.format, OutputFormat::Json);

    let verify = crate::app::args::parse([
        OsString::from("zrail"),
        OsString::from("mirrors"),
        OsString::from("verify"),
        OsString::from("--plan"),
        OsString::from("evidence/mirrors.json"),
    ])
    .expect("parse mirror verification");
    let Command::Mirrors(verify) = verify else {
        panic!("expected mirrors command");
    };
    assert_eq!(
        verify.action,
        MirrorsAction::Verify {
            plan: PathBuf::from("evidence/mirrors.json")
        }
    );
}

#[test]
fn verify_requires_one_plan_and_plan_rejects_it() {
    let missing = crate::app::args::parse([
        OsString::from("zrail"),
        OsString::from("mirrors"),
        OsString::from("verify"),
    ])
    .expect_err("verify requires plan");
    assert!(missing.to_string().contains("requires --plan"));

    let unexpected = crate::app::args::parse([
        OsString::from("zrail"),
        OsString::from("mirrors"),
        OsString::from("plan"),
        OsString::from("--plan"),
        OsString::from("plan.json"),
    ])
    .expect_err("plan must reject input plan");
    assert!(unexpected.to_string().contains("does not accept --plan"));
}

#[test]
fn receipts_requires_exact_plan_and_result_paths() {
    let parsed = crate::app::args::parse([
        OsString::from("zrail"),
        OsString::from("mirrors"),
        OsString::from("receipts"),
        OsString::from("--plan"),
        OsString::from("evidence/plan.json"),
        OsString::from("--results"),
        OsString::from("evidence/results.json"),
        OsString::from("--format"),
        OsString::from("json"),
    ])
    .expect("parse receipt renderer");
    let Command::Mirrors(options) = parsed else {
        panic!("expected mirrors command");
    };
    assert_eq!(
        options.action,
        MirrorsAction::Receipts {
            plan: PathBuf::from("evidence/plan.json"),
            results: PathBuf::from("evidence/results.json"),
        }
    );

    let missing = crate::app::args::parse([
        OsString::from("zrail"),
        OsString::from("mirrors"),
        OsString::from("receipts"),
        OsString::from("--plan"),
        OsString::from("plan.json"),
    ])
    .expect_err("receipt renderer requires results");
    assert!(missing.to_string().contains("requires --results"));
}
