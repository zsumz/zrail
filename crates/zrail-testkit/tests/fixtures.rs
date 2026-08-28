//! Ballast-derived good, bad, stale, and bypass-oriented repository cases.

use std::path::{Path, PathBuf};

use zrail_core::ReportStatus;
use zrail_rust::check_repository;

#[test]
fn known_good_repository_passes() {
    let report = check("good");
    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
}

#[test]
fn root_package_workspace_passes() {
    let report = check("root_package");
    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
}

#[test]
fn reasoned_lints_and_permissive_entrypoints_pass() {
    let report = check("reasoned_entrypoint");
    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
}

#[test]
fn verified_generated_fragments_pass() {
    let report = check("generated_source");
    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
}

#[test]
fn exact_rust_source_graphs_pass() {
    for fixture in ["source_graph_good", "path_loaded_module"] {
        let report = check(fixture);
        assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    }
}

#[test]
fn nested_inline_test_source_graph_passes() {
    let report = check("nested_test_context");
    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
}

#[test]
fn exact_invariant_evidence_and_reviewed_gate_pass() {
    let report = check("evidence_good");
    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
}

#[test]
fn removed_declared_test_evidence_is_rejected() {
    assert_rule("evidence_missing", "EVID-003");
}

#[test]
fn duplicate_simple_test_names_cannot_masquerade_as_exact_evidence() {
    assert_rule("evidence_ambiguous", "EVID-004");
}

#[test]
fn changed_qualification_gate_bytes_require_lock_review() {
    assert_rule("evidence_stale_gate", "LOCK-016");
}

#[test]
fn missing_module_source_is_rejected() {
    assert_rule("missing_module", "RUST-GRAPH-001");
}

#[test]
fn source_graph_escape_is_rejected() {
    assert_rule("source_graph_escape", "RUST-GRAPH-002");
}

#[test]
fn item_macro_source_edges_are_unresolved() {
    let report = check("macro_source_escape");
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-GRAPH-003" && finding.message.contains("item-position macro")
        }),
        "{}",
        report.human()
    );
}

#[test]
fn stale_item_macro_exemptions_are_rejected() {
    assert_rule("macro_source_escape", "RUST-GRAPH-005");
}

#[test]
fn stale_out_dir_bindings_are_rejected() {
    assert_rule("stale_out_dir", "RUST-GRAPH-006");
}

#[test]
fn conditional_paths_and_generated_includes_are_unresolved() {
    let report = check("unresolved_graph");
    let unresolved = report
        .findings
        .iter()
        .filter(|finding| finding.id == "RUST-GRAPH-003")
        .map(|finding| finding.message.as_str())
        .collect::<Vec<_>>();
    assert!(
        unresolved
            .iter()
            .any(|message| message.contains("conditional"))
    );
    assert!(
        unresolved
            .iter()
            .any(|message| message.contains("generated"))
    );
    assert!(
        unresolved
            .iter()
            .any(|message| message.contains("Rust items")),
        "{}",
        report.human()
    );
}

#[test]
fn unreachable_rust_source_is_rejected() {
    assert_rule("orphan_source", "RUST-GRAPH-004");
}

#[test]
fn facade_implementation_is_rejected() {
    assert_rule("bad_facade", "RUST-FACADE-001");
}

#[test]
fn aliased_forbidden_capability_is_rejected() {
    assert_rule("forbidden_capability", "CAP-001");
}

#[test]
fn aliased_capability_use_outside_its_owner_is_rejected() {
    let report = check("forbidden_capability");
    let trespasses = report
        .findings
        .iter()
        .filter(|finding| finding.id == "OWN-003" && finding.rule == "filesystem-owner")
        .collect::<Vec<_>>();
    assert!(
        trespasses.iter().any(|finding| finding
            .path
            .as_deref()
            .is_some_and(|path| path.ends_with("trespasser.rs"))),
        "{}",
        report.human()
    );
    assert!(!trespasses.iter().any(|finding| {
        finding
            .path
            .as_deref()
            .is_some_and(|path| path.ends_with("/owner.rs"))
    }));
}

#[test]
fn inline_tests_are_rejected() {
    assert_rule("inline_test", "RUST-TEST-001");
}

#[test]
fn missing_module_contract_is_rejected() {
    assert_rule("missing_contract", "RUST-DOC-001");
}

#[test]
fn design_target_and_hard_ceiling_are_distinct() {
    let report = check("oversized");
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-SIZE-001")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-SIZE-002")
    );
}

#[test]
fn reversed_dependency_is_rejected() {
    assert_rule("reversed_dependency", "DEP-005");
}

#[test]
fn unchecked_failures_and_suppressions_are_rejected() {
    let report = check("source_hygiene");
    for rule in ["RUST-HYG-001", "RUST-HYG-002", "RUST-HYG-003"] {
        assert!(
            report.findings.iter().any(|finding| finding.id == rule),
            "missing {rule}: {}",
            report.human()
        );
    }
}

#[test]
fn unsafe_boundaries_are_detected_across_rust_editions() {
    let report = check("unsafe_editions");
    for package in ["edition2015", "edition2018", "edition2021", "edition2024"] {
        assert!(
            report.findings.iter().any(|finding| {
                finding.id == "RUST-HYG-004"
                    && finding
                        .path
                        .as_deref()
                        .is_some_and(|path| path.contains(package))
            }),
            "missing unsafe finding for {package}: {}",
            report.human()
        );
    }
}

#[test]
fn production_reachability_overrides_test_like_pathnames() {
    let report = check("production_test_path");
    for rule in ["RUST-HYG-001", "RUST-SIZE-001"] {
        assert!(
            report.findings.iter().any(|finding| finding.id == rule),
            "missing {rule}: {}",
            report.human()
        );
    }
}

#[test]
fn sibling_tests_cannot_be_both_test_and_production_reachable() {
    assert_rule("dual_reachable_test", "RUST-TEST-004");
}

#[test]
fn undeclared_sibling_test_is_rejected() {
    assert_rule("undeclared_test", "RUST-TEST-002");
}

#[test]
fn sibling_tests_require_an_exact_path_declaration() {
    assert_rule("wrong_test_path", "RUST-TEST-003");
}

#[test]
fn stale_dependency_edges_are_rejected() {
    assert_rule("stale_lock", "LOCK-006");
}

#[test]
fn stale_repository_exclusions_are_rejected() {
    assert_rule("stale_exclusion", "REP-006");
}

#[test]
fn stale_capability_scopes_are_rejected() {
    assert_rule("stale_scope", "CAP-002");
}

#[test]
fn stale_capability_owners_are_rejected() {
    assert_rule("stale_scope", "OWN-004");
}

#[test]
fn stale_package_layers_are_rejected() {
    assert_rule("stale_layer", "DEP-010");
}

fn assert_rule(name: &str, rule: &str) {
    let report = check(name);
    assert!(
        report.findings.iter().any(|finding| finding.id == rule),
        "fixture {name} did not produce {rule}: {}",
        report.human()
    );
}

fn check(name: &str) -> zrail_core::Report {
    let root = fixture_root(name);
    check_repository(&root, Path::new("zrail.toml"), Path::new("zrail.lock"))
        .unwrap_or_else(|error| panic!("check {}: {error}", root.display()))
        .report
}

fn fixture_root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}
