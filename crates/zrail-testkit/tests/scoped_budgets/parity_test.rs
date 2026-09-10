//! Frozen Kafkars baseline/allow predicates are compared with stock rule diagnostics.

#[path = "legacy_kafkars.rs"]
mod legacy_kafkars;

use super::{
    fixture::Repository,
    support::{EXCEPTION, RATCHET, SCOPE, measured, size_finding},
};
use zrail_core::ReportStatus;

#[derive(Clone, Copy)]
struct Budget {
    target: usize,
    soft: usize,
    hard: usize,
}
struct BudgetBaseline {
    lines: usize,
    reason: String,
}
struct BudgetAllow {
    reason: String,
    owner: String,
    issue: String,
}

#[derive(Clone, Copy)]
enum MetadataField {
    BaselineReason,
    HardReason,
    Owner,
    Issue,
}

#[test]
fn frozen_required_metadata_rejections_match_strict_contract_validation() {
    for (field, remove, expected) in [
        (
            MetadataField::BaselineReason,
            "Exact reviewed measurement.",
            "requires a reason",
        ),
        (
            MetadataField::HardReason,
            "Split the existing seam.",
            "requires a reason",
        ),
        (MetadataField::Owner, "architecture", "empty owner"),
        (MetadataField::Issue, "ARCH-42", "empty issue"),
    ] {
        assert!(!legacy_kafkars::invalid_metadata(field).is_empty());
        let contract = format!(
            "[source.rust.budgets]\nexception_metadata = 'owner-issue'\n{SCOPE}{EXCEPTION}{RATCHET}"
        )
        .replace(remove, "");
        let repository = Repository::new("declarative", &contract);
        let error = zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref())
            .expect_err("invalid metadata");
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn imported_oversized_fixture_preserves_target_rejection_and_exact_allowance_acceptance() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/rc9/kafkars-oversized.rs.txt"),
    )
    .expect("frozen fixture");
    assert_eq!(source.lines().count(), 10);
    for (contract, passes) in [
        (SCOPE.to_owned(), false),
        (
            format!("{SCOPE}{RATCHET}").replace("baseline = 11", "baseline = 10"),
            true,
        ),
        (
            format!("{SCOPE}{EXCEPTION}{RATCHET}")
                .replace("hard = 10", "hard = 9")
                .replace("hard = 12", "hard = 10")
                .replace("baseline = 11", "baseline = 10"),
            true,
        ),
    ] {
        let repository = Repository::new("declarative", &contract);
        repository.write("src/lib.rs", "//! Wiring.\nmod child;\n");
        repository.write("src/child.rs", &source);
        repository.lock();
        let result = repository.check();
        assert_eq!(
            result.report.status == ReportStatus::Pass,
            passes,
            "{:?}",
            result.report
        );
        if !passes {
            assert!(size_finding(&result, "RUST-SIZE-002").is_some());
        }
    }
}

#[test]
fn frozen_baseline_and_hard_allowance_predicates_match_all_fifty_four_cases() {
    for lines in [2, 5, 6, 8, 9, 10, 11, 12, 13] {
        for baseline in [None, Some(11), Some(12)] {
            for allowed in [false, true] {
                compare_case(lines, baseline, allowed);
            }
        }
    }
}

fn compare_case(lines: usize, baseline: Option<usize>, allowed: bool) {
    let legacy = legacy_kafkars::violations(lines, baseline, allowed);
    let ratchet = baseline.map_or_else(String::new, |baseline| {
        RATCHET.replace("baseline = 11", &format!("baseline = {baseline}"))
    });
    let exception = if allowed { EXCEPTION } else { "" };
    let repository = Repository::new(
        "declarative",
        &format!(
            "[source.rust.budgets]\nexception_metadata = 'owner-issue'\n{SCOPE}{exception}{ratchet}"
        ),
    );
    repository.write("src/child.rs", &measured(lines));
    repository.lock();
    let result = repository.check();
    assert!(result.report.analysis.complete);
    assert_eq!(
        result.report.status == ReportStatus::Pass,
        legacy.is_empty(),
        "{lines} lines, baseline {baseline:?}, allowed {allowed}: legacy {legacy:?}; stock {:?}",
        result.report
    );
    for violation in legacy {
        let diagnostic = if violation.contains("stale hard-ceiling") {
            "RUST-SIZE-008"
        } else if violation.contains("hard ceiling") {
            "RUST-SIZE-001"
        } else if violation.contains("stale baseline") {
            "RUST-SIZE-004"
        } else if violation.contains("baseline") {
            "RUST-SIZE-009"
        } else {
            "RUST-SIZE-002"
        };
        assert!(
            size_finding(&result, diagnostic).is_some(),
            "expected {diagnostic} for {violation}; {:?}",
            result.report
        );
    }
}
