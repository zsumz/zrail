//! Wiring-only facades preserve rc8 declarations and independent test compilation.

#[path = "strict_facades/fixture.rs"]
mod fixture;

use fixture::{Repository, TEST_FACADE};
use zrail_core::{FacadeMode, ReportStatus};

#[test]
fn wiring_only_rejects_each_non_wiring_item_through_the_facade_rail() {
    let repository = Repository::new("wiring-only", "");
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    for item in [
        "pub struct Extra;",
        "const EXTRA: u8 = 1;",
        "enum Extra { A }",
        "type Extra = u8;",
        "fn helper() {}",
        "impl State {}",
        "mod inline {}",
        "extern crate core;",
        "macro_rules! extra { () => {} }",
    ] {
        repository.write(
            "src/lib.rs",
            &format!("//! Wiring.\nmod child;\npub use child::State;\n{item}\n"),
        );
        let result = repository.check();
        assert!(
            result.report.analysis.complete,
            "{item}: {:?}",
            result.report
        );
        assert!(
            result.report.findings.iter().any(|finding| {
                finding.id == "RUST-FACADE-001"
                    && finding.rule == "rust.facades"
                    && finding.path.as_deref() == Some("src/lib.rs")
            }),
            "{item}: {:?}",
            result.report
        );
    }
}

#[test]
fn import_modes_preserve_the_two_consumers_distinct_written_visibility_rules() {
    for mode in ["wiring-only", "wiring-reexports"] {
        let repository = Repository::new(mode, "");
        for (import, permitted_reexport) in [
            ("use child::State;", false),
            ("pub(self) use child::State;", false),
            ("pub use child::State;", true),
            ("pub(crate) use child::{State};", true),
            ("pub(in crate) use child::State;", true),
        ] {
            repository.write(
                "src/lib.rs",
                &format!("//! Wiring.\nmod child;\n{import}\n"),
            );
            repository.lock();
            let result = repository.check();
            let expected = if mode == "wiring-only" || permitted_reexport {
                ReportStatus::Pass
            } else {
                ReportStatus::Fail
            };
            assert_eq!(
                result.report.status, expected,
                "{mode} {import}: {:?}",
                result.report
            );
            if expected == ReportStatus::Fail {
                assert!(
                    result
                        .report
                        .findings
                        .iter()
                        .any(|finding| finding.id == "RUST-FACADE-001")
                );
            }
        }
    }
}

#[test]
fn frozen_kafkars_negative_fixture_reports_each_architectural_violation() {
    let repository = Repository::new("wiring-reexports", "");
    repository.write("src/declared.rs", "//! Vocabulary.\npub struct Declared;\n");
    repository.write("src/lib.rs", "//! Wiring.\nmod child;\nmod declared;\n");
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let fixture = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/rc9/kafkars-facade-invalid.rs.txt"),
    )
    .expect("read frozen facade fixture");
    repository.write("src/lib.rs", &fixture);
    // Keep the unrelated child mounted so orphan diagnostics cannot satisfy the test.
    repository.write(
        "src/declared.rs",
        "//! Vocabulary.\n#[path = \"child.rs\"] mod child;\npub struct Declared;\n",
    );
    let result = repository.check();
    assert!(result.report.analysis.complete);
    let findings = result
        .report
        .findings
        .iter()
        .filter(|finding| {
            finding.id == "RUST-FACADE-001" && finding.path.as_deref() == Some("src/lib.rs")
        })
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 4, "{:?}", result.report);
    for kind in ["function", "inline module", "non-reexport import"] {
        assert!(
            findings
                .iter()
                .any(|finding| finding.message.ends_with(kind)),
            "missing {kind}"
        );
    }
}

#[test]
fn test_facade_structure_keeps_test_reachability_budgets_and_scenario_identity() {
    let repository = Repository::new("declarative", TEST_FACADE);
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let facade = repository.explain("tests/suite.rs");
    assert_eq!(facade.file_class, "test");
    assert_eq!(facade.effective_file_role, "test");
    assert_eq!(facade.reachability, "test-only");
    assert_eq!(facade.design_target, Some(320));
    assert_eq!(facade.facade_mode, Some(FacadeMode::WiringOnly));
    assert_eq!(
        facade.file_role_reason.as_deref(),
        Some("Scenario implementations belong in child modules.")
    );
    assert!(facade.human().contains("facade mode: wiring-only"));
    let scenario = repository.explain("tests/suite/cases.rs");
    assert_eq!(scenario.reachability, "test-only");
    assert_eq!(scenario.facade_mode, None);
    repository.write(
        "tests/suite.rs",
        "//! Scenario wiring.\n#[path = \"suite/cases.rs\"] mod cases;\n#[test] fn misplaced() {}\n",
    );
    let result = repository.check();
    assert!(result.report.findings.iter().any(|finding| {
        finding.id == "RUST-FACADE-001" && finding.path.as_deref() == Some("tests/suite.rs")
    }));
    let coverage = zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref())
        .expect("coverage");
    let governed = coverage
        .facades
        .iter()
        .find(|facade| facade.path == "tests/suite.rs")
        .expect("test facade");
    assert_eq!(governed.policy_id, "rust:file-role:tests/suite.rs");
    assert!(governed.test_only);
    assert_eq!(governed.violations.len(), 1);
    assert_eq!(governed.violations[0].kind, "function");
    assert_eq!(
        coverage,
        zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref())
            .expect("repeat coverage")
    );
}

#[test]
fn test_facade_cannot_reclassify_production_or_hide_a_missing_mount() {
    let repository = Repository::new(
        "declarative",
        &TEST_FACADE.replace("tests/suite.rs", "src/child.rs"),
    );
    repository.lock();
    assert!(
        repository
            .check()
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-ROLE-002")
    );
    repository.write("src/lib.rs", "//! Wiring.\n");
    assert!(
        repository
            .check()
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-ROLE-001")
    );
}

#[test]
fn declarative_mode_retains_rc8_data_declarations() {
    let repository = Repository::new("declarative", "");
    repository.write(
        "src/lib.rs",
        "//! Vocabulary.\nmod child;\npub struct Extra;\nconst EXTRA: u8 = 1;\n",
    );
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
}

#[test]
fn production_mount_of_a_test_facade_fails_without_changing_its_test_identity() {
    let repository = Repository::new("declarative", TEST_FACADE);
    repository.lock();
    repository.write(
        "src/lib.rs",
        "//! Wiring.\nmod child;\n#[path = \"../tests/suite.rs\"] mod leaked;\n",
    );
    let result = repository.check();
    assert!(
        result
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-ROLE-002"
                && finding.path.as_deref() == Some("tests/suite.rs"))
    );
    assert_eq!(repository.explain("tests/suite.rs").reachability, "both");
}

#[test]
fn expression_fragments_cannot_pass_as_empty_wiring_facades() {
    let repository = Repository::new(
        "allow",
        r#"
[[source.rust.file_roles]]
path = "src/expression.rs"
role = "facade"
mode = "wiring-only"
reason = "This must be an item facade."
"#,
    );
    repository.write("src/expression.rs", "3");
    repository.write("src/lib.rs", "//! Expression owner.\nmod child;\npub fn value() -> i32 { include!(\"expression.rs\") }\n");
    repository.lock();
    let result = repository.check();
    assert!(result.report.analysis.complete, "{:?}", result.report);
    assert!(
        result
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-FACADE-002"
                && finding.path.as_deref() == Some("src/expression.rs"))
    );
    let coverage = zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref())
        .expect("coverage");
    assert!(
        !coverage
            .facades
            .iter()
            .find(|facade| facade.path == "src/expression.rs")
            .expect("fragment facade")
            .syntax_allowed
    );
}
