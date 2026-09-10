//! Prefix rejection, native representation and exact comparison are separately observed.

use super::{model, native, original};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use zrail_core::FindingSink;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Matrix {
    schema: u32,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    template: String,
    prefix_accepted: bool,
    native_contract_accepted: bool,
    native_graph_accepted: bool,
}
#[derive(Serialize)]
pub(super) struct Row {
    policy_id: String,
    case: String,
    input_version: String,
    suffix: Option<String>,
    prefix_error: Option<String>,
    original_helper_error: Option<String>,
    native_version: String,
    native: native::Native,
}

pub(super) fn run() -> Vec<Row> {
    let matrix: Matrix =
        serde_json::from_slice(&fs::read(model::project().join(model::CASES)).expect("cases"))
            .expect("typed matrix");
    assert_eq!(matrix.schema, 1);
    assert_eq!(matrix.cases.len(), 24);
    let fixture = model::Fixture::new();
    let (rules, _) = model::lock_model::policies();
    let mut rows = Vec::new();
    for rule in &rules {
        for case in &matrix.cases {
            let input = case
                .template
                .replace("$VERSION", &model::identity(rule).version);
            let prefix =
                std::panic::catch_unwind(|| original::expected_version(&rule.package, &input))
                    .map_err(|payload| model::panic_text(payload.as_ref()));
            assert_eq!(
                prefix.is_ok(),
                case.prefix_accepted,
                "{}:{} prefix",
                rule.name,
                case.name
            );
            let version = prefix.as_deref().unwrap_or(&input).to_owned();
            let candidate = model::candidate(rule, &version);
            let (lock, native) = native::paired(&fixture.0, &candidate);
            let original = model::original_helper(&lock, rule, &input);
            assert_eq!(
                original.is_ok(),
                case.prefix_accepted,
                "paired literal original helper"
            );
            if let Err(error) = &original {
                assert_eq!(
                    error,
                    &format!("{} guardrail version must be exact", rule.package)
                );
            }
            assert_eq!(
                native.contract_error.is_none(),
                case.native_contract_accepted,
                "{}:{} contract",
                rule.name,
                case.name
            );
            assert_eq!(
                native.graph_error.is_none(),
                case.native_graph_accepted,
                "{}:{} graph",
                rule.name,
                case.name
            );
            if let Some(row) = &native.observation {
                assert!(row.satisfied);
                assert_eq!(row.observed_count, 1);
            }
            rows.push(Row {
                policy_id: format!("dependency:lock-package:{}", rule.name),
                case: case.name.clone(),
                input_version: input.clone(),
                suffix: prefix.as_ref().ok().cloned(),
                prefix_error: prefix.err(),
                original_helper_error: original.err(),
                native_version: version,
                native,
            });
        }
    }
    rows
}

#[derive(Serialize)]
pub(super) struct Literal {
    policy_id: String,
    case: String,
    expected_version: String,
    original_helper_error: String,
    native_diagnostic: String,
    observation: crate::GovernedLockPackage,
}

pub(super) fn literals(root: &Path) -> Vec<Literal> {
    let fixture = model::Fixture::new();
    let lock: toml::Value = fs::read_to_string(root.join("Cargo.lock"))
        .expect("frozen lock")
        .parse()
        .expect("TOML");
    let workspace = model::lock_model::workspace(root);
    let (rules, _) = model::lock_model::policies();
    let mut rows = Vec::new();
    for rule in &rules {
        for (case, version) in [
            ("caret", format!("^{}", model::identity(rule).version)),
            ("wildcard", "*".into()),
            ("different", "9.9.9".into()),
            (
                "build-metadata",
                format!("{}+build.1", model::identity(rule).version),
            ),
        ] {
            let candidate = model::candidate(rule, &version);
            native::contract(&fixture.0, &candidate)
                .1
                .expect("literal accepted syntactically");
            let error = model::original_helper(&lock, rule, &format!("={version}"))
                .expect_err("exact original mismatch");
            assert!(!error.contains("guardrail version must be exact"));
            let observed =
                model::lock_model::native(root, &workspace, std::slice::from_ref(&candidate))
                    .expect("complete frozen graph")
                    .remove(0);
            assert!(!observed.satisfied);
            assert_eq!(observed.observed_count, 1);
            let mut sink = FindingSink::default();
            crate::lock_packages::evaluate(std::slice::from_ref(&observed), &mut sink);
            let findings = sink.into_findings();
            assert_eq!(findings.len(), 1);
            assert_eq!(findings[0].id, "DEP-LOCK-001");
            assert_eq!(findings[0].rule, observed.policy_id);
            rows.push(Literal {
                policy_id: observed.policy_id.clone(),
                case: case.into(),
                expected_version: version,
                original_helper_error: error,
                native_diagnostic: findings[0].id.clone(),
                observation: observed,
            });
        }
    }
    rows
}
