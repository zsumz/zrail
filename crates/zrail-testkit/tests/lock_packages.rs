//! Whole-lock inventories preserve quantities and identities independently of reachability.

#[path = "lock_packages/boundary_test.rs"]
mod boundary_test;
#[path = "strict_facades/fixture.rs"]
mod fixture;
#[path = "lock_packages/support.rs"]
mod support;

use support::{configured, count, coverage, exact, graph, node, violation};
use zrail_core::{AnalysisQuality, ReportStatus};

#[test]
fn counts_include_unreachable_nodes_and_absent_bans_remain_active() {
    let repository = configured(
        &count("wire", "wire", 1),
        &graph(&[node("wire", "1.0.0", 'a')]),
    );
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let observed = coverage(&repository);
    assert_eq!(observed.lock_packages[0].lock_node_count, 2);
    assert_eq!(observed.lock_packages[0].observed_count, 1);
    assert_eq!(observed.lock_packages[0].scope, "whole-cargo-lock");
    assert_eq!(observed.lock_packages[0].quality, AnalysisQuality::Exact);
    repository.write("Cargo.lock", &graph(&[]));
    violation(&repository, "wire");
    repository.write(
        "Cargo.lock",
        &graph(&[node("wire", "1.0.0", 'a'), node("wire", "2.0.0", 'b')]),
    );
    violation(&repository, "wire");
    let policy = std::fs::read_to_string(repository.0.join("zrail.toml")).expect("contract");
    repository.write(
        "zrail.toml",
        &format!("{policy}\n{}", count("retired", "retired", 0)),
    );
    repository.write("Cargo.lock", &graph(&[node("wire", "1.0.0", 'a')]));
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    repository.write(
        "Cargo.lock",
        &graph(&[node("wire", "1.0.0", 'a'), node("retired", "9.0.0", 'c')]),
    );
    violation(&repository, "retired");
}

#[test]
fn exact_sets_reject_same_count_version_source_and_checksum_substitutions() {
    let original = node("wire", "1.0.0", 'a');
    let repository = configured(
        &exact("wire", "wire", &["1.0.0"]),
        &graph(std::slice::from_ref(&original)),
    );
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    for replacement in [
        node("wire", "2.0.0", 'a'),
        node("wire", "1.0.0", 'b'),
        original.replace("index.crates.io", "other.invalid"),
    ] {
        repository.write("Cargo.lock", &graph(&[replacement]));
        violation(&repository, "wire");
        let observed = coverage(&repository);
        let rule = &observed.lock_packages[0];
        assert_eq!(rule.observed_count, 1);
        let delta = rule
            .exact_difference
            .as_ref()
            .expect("identity differences");
        assert_eq!(delta.missing.len(), 1);
        assert_eq!(delta.unexpected_count, 1);
    }
    repository.write(
        "Cargo.lock",
        &graph(&[original, node("wire", "2.0.0", 'a')]),
    );
    violation(&repository, "wire");
    repository.write("Cargo.lock", &graph(&[node("other", "1.0.0", 'a')]));
    violation(&repository, "wire");
    let delta = coverage(&repository)
        .lock_packages
        .remove(0)
        .exact_difference
        .expect("missing required identity");
    assert_eq!(delta.missing.len(), 1);
    assert_eq!(delta.unexpected_count, 0);
}

#[test]
fn set_order_is_irrelevant_and_local_nodes_have_exact_path_and_checksum_absence() {
    let policies = format!(
        "{}\n{}",
        exact("wire", "wire", &["2.0.0", "1.0.0"]),
        "[[dependencies.lock_package]]\nname = 'local'\npackage = 'fixture'\nreason = 'Reviewed local identity.'\nassertion = { kind = 'exact', identities = [{version = '0.0.0', source = 'path+.'}] }"
    );
    let first = graph(&[node("wire", "2.0.0", 'a'), node("wire", "1.0.0", 'a')]);
    let repository = configured(&policies, &first);
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let report = coverage(&repository);
    assert_eq!(report.lock_packages[0].policy.name, "local");
    assert_eq!(report.lock_packages[0].observed_sample[0].checksum, None);
    let explained = repository.explain("Cargo.lock");
    assert_eq!(explained.lock_packages, report.lock_packages);
    assert!(explained.human().contains("dependency:lock-package:wire"));
    assert!(
        report.human().contains("whole-cargo-lock") || report.human().contains("whole-lock nodes")
    );
    assert!(repository.explain("src/child.rs").lock_packages.is_empty());
    assert_eq!(
        report.json().expect("first JSON"),
        coverage(&repository).json().expect("repeat JSON")
    );
    let wire = &report.lock_packages[1];
    repository.write(
        "Cargo.lock",
        &graph(&[node("wire", "1.0.0", 'a'), node("wire", "2.0.0", 'a')]),
    );
    let changed = coverage(&repository);
    assert_eq!(
        wire.observed_sample,
        changed.lock_packages[1].observed_sample
    );
    assert_ne!(wire.lock_sha256, changed.lock_packages[1].lock_sha256);
    assert!(changed.lock_packages.iter().all(|policy| policy.satisfied));
}
