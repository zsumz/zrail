//! Scoped budgets preserve warnings, physical quantities, and reviewed debt boundaries.

#[path = "strict_facades/fixture.rs"]
mod fixture;
#[path = "scoped_budgets/parity_test.rs"]
mod parity_test;
#[path = "scoped_budgets/selection_test.rs"]
mod selection_test;
#[path = "scoped_budgets/support.rs"]
mod support;

use fixture::{Repository, TEST_FACADE};
use support::{EXCEPTION, RATCHET, SCOPE, measured, size_finding};
use zrail_core::{ReportStatus, Severity, SizeRole};

#[test]
fn warning_targets_and_soft_thresholds_do_not_become_release_failures() {
    let repository = Repository::new(
        "declarative",
        &SCOPE.replace("target = 5", "target = 5, target_mode = 'warn'"),
    );
    for lines in [5, 6, 8, 9, 10, 11] {
        repository.write("src/child.rs", &measured(lines));
        repository.lock();
        let result = repository.check();
        assert_eq!(
            result.report.status,
            if lines > 10 {
                ReportStatus::Fail
            } else {
                ReportStatus::Pass
            },
            "{lines}: {:?}",
            result.report
        );
        assert_eq!(size_finding(&result, "RUST-SIZE-002").is_some(), lines > 5);
        assert_eq!(size_finding(&result, "RUST-SIZE-007").is_some(), lines > 8);
        for finding in result
            .report
            .findings
            .iter()
            .filter(|finding| matches!(finding.id.as_str(), "RUST-SIZE-002" | "RUST-SIZE-007"))
        {
            assert_eq!(finding.severity, Severity::Warning);
        }
    }
}

#[test]
fn scope_intersects_packages_paths_and_roles_without_changing_default_families() {
    let repository = Repository::new("declarative", SCOPE);
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    assert_eq!(repository.explain("src/lib.rs").design_target, Some(80));
    assert_eq!(
        repository.explain("tests/suite/cases.rs").design_target,
        Some(320)
    );
    let explanation = repository.explain("src/child.rs");
    assert_eq!(explanation.design_target, Some(5));
    assert_eq!(
        explanation
            .effective_budget
            .as_ref()
            .expect("budget")
            .policy_id,
        "rust:size:scope:core"
    );
    assert!(explanation.human().contains("soft 8"));
    repository.write("src/child.rs", &measured(6));
    let result = repository.check();
    assert!(
        size_finding(&result, "RUST-SIZE-002").is_some_and(|finding| finding
            .help
            .as_ref()
            .is_some_and(|help| help.contains("rust:size:scope:core")))
    );
}

#[test]
fn test_facade_budget_is_independent_of_test_execution_and_ordinary_test_limit() {
    let scope = SCOPE
        .replace("include = [\"src/**\"]", "include = [\"tests/**\"]")
        .replace("roles = [\"implementation\"]", "roles = [\"test-facade\"]");
    let repository = Repository::new("declarative", &format!("{TEST_FACADE}{scope}"));
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let explanation = repository.explain("tests/suite.rs");
    assert_eq!(explanation.reachability, "test-only");
    assert_eq!(explanation.file_class, "test");
    assert_eq!(explanation.design_target, Some(5));
    assert_eq!(
        explanation.effective_budget.expect("budget").role,
        SizeRole::TestFacade
    );
    assert_eq!(
        repository.explain("tests/suite/cases.rs").design_target,
        Some(320)
    );
}

#[test]
fn overlapping_same_tier_scopes_reject_check_coverage_and_lock_building() {
    let other = SCOPE
        .replace("name = \"core\"", "name = \"other\"")
        .replace("src/**", "src/child.rs");
    for policies in [format!("{SCOPE}{other}"), format!("{other}{SCOPE}")] {
        let repository = Repository::new("declarative", &policies);
        for error in [
            zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref())
                .expect_err("ambiguous budget"),
            zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref())
                .expect_err("ambiguous coverage"),
            zrail_rust::check_repository(
                &repository.0,
                "zrail.toml".as_ref(),
                "zrail.lock".as_ref(),
            )
            .expect_err("ambiguous check"),
        ] {
            assert!(error.to_string().contains("RUST-SIZE-006"), "{error}");
            assert!(error.to_string().contains("core, other"), "{error}");
        }
    }
}

#[test]
fn lock_updates_cannot_grant_independent_hard_exceptions() {
    let repository = Repository::new("declarative", &format!("{SCOPE}{RATCHET}"));
    repository.write("src/child.rs", &measured(11));
    repository.lock();
    assert!(size_finding(&repository.check(), "RUST-SIZE-001").is_some());
    repository.lock();
    assert!(size_finding(&repository.check(), "RUST-SIZE-001").is_some());
}

#[test]
fn exact_hard_allowance_and_authored_baseline_accept_only_the_reviewed_file() {
    let repository = Repository::new("declarative", &format!("{SCOPE}{EXCEPTION}{RATCHET}"));
    repository.write("src/child.rs", &measured(11));
    repository.lock();
    let result = repository.check();
    assert_eq!(
        result.report.status,
        ReportStatus::Pass,
        "{:?}",
        result.report
    );
    assert_eq!(
        size_finding(&result, "RUST-SIZE-010")
            .expect("active debt")
            .severity,
        Severity::Warning
    );
    repository.write("src/newcomer.rs", &measured(11));
    repository.write("src/lib.rs", "//! Wiring.\nmod child;\nmod newcomer;\n");
    let result = repository.check();
    assert!(
        result
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-SIZE-001"
                && finding.path.as_deref() == Some("src/newcomer.rs"))
    );
}

#[test]
fn authored_and_locked_measurements_reject_growth_shrinkage_and_removed_debt() {
    let repository = Repository::new("declarative", &format!("{SCOPE}{EXCEPTION}{RATCHET}"));
    repository.write("src/child.rs", &measured(11));
    repository.lock();
    for lines in [12, 10, 9] {
        repository.write("src/child.rs", &measured(lines));
        assert!(size_finding(&repository.check(), "RUST-SIZE-009").is_some());
        repository.lock();
        assert!(size_finding(&repository.check(), "RUST-SIZE-009").is_some());
    }
    repository.write("src/child.rs", &measured(13));
    assert!(size_finding(&repository.check(), "RUST-SIZE-001").is_some());
    repository.write("src/child.rs", &measured(2));
    let result = repository.check();
    assert!(size_finding(&result, "RUST-SIZE-004").is_some());
    assert!(size_finding(&result, "RUST-SIZE-008").is_some());
}

#[test]
fn missing_exception_target_is_stale_even_after_the_source_is_deleted() {
    let repository = Repository::new("declarative", &format!("{SCOPE}{EXCEPTION}"));
    repository.lock();
    std::fs::remove_file(repository.0.join("src/child.rs")).expect("remove test-owned source");
    repository.write("src/lib.rs", "//! Wiring.\n");
    assert!(size_finding(&repository.check(), "RUST-SIZE-008").is_some());
}

#[test]
fn coverage_counts_physical_lines_once_and_preserves_all_debt_metadata() {
    let repository = Repository::new("declarative", &format!("{SCOPE}{EXCEPTION}{RATCHET}"));
    repository.write("src/child.rs", &measured(11));
    repository.write(
        "src/lib.rs",
        "//! Wiring.\nmod child;\n#[path = \"child.rs\"] mod alternate;\n",
    );
    repository.lock();
    let report = zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref())
        .expect("coverage");
    let matches = report
        .size_budgets
        .iter()
        .filter(|budget| budget.path == "src/child.rs")
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1);
    let budget = matches[0];
    assert_eq!(
        (
            budget.lines,
            budget.above_target,
            budget.above_soft,
            budget.above_hard,
            budget.above_effective_hard
        ),
        (11, 6, 3, 1, 0)
    );
    assert_eq!(budget.ratchet.as_ref().expect("ratchet").baseline, Some(11));
    assert_eq!(
        budget
            .effective
            .exception
            .as_ref()
            .expect("exception")
            .issue
            .as_deref(),
        Some("ARCH-42")
    );
    assert!(
        report
            .enabled_rails
            .contains(&"rust:size:exception:src/child.rs".into())
    );
    assert_eq!(
        report,
        zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref())
            .expect("repeat coverage")
    );
}

#[test]
fn old_contracts_preserve_the_documented_rc8_ratchet_behavior() {
    let ratchet = RATCHET.replace("baseline = 11\n", "");
    let repository = Repository::new("declarative", &ratchet);
    repository.write("src/child.rs", &measured(250));
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    assert!(
        !repository
            .explain("src/child.rs")
            .effective_budget
            .expect("budget")
            .independent_hard
    );
}
