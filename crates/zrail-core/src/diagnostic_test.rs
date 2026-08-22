//! Stable diagnostic ordering and fingerprints.

use super::{DiagnosticLimit, Finding, FindingSink, Severity, sort_findings};

#[test]
fn fingerprints_are_stable_and_sensitive_to_location() {
    let first =
        Finding::error("CAP-001", "core", "capability", "network is denied").at("src/lib.rs", None);
    let same =
        Finding::error("CAP-001", "core", "capability", "network is denied").at("src/lib.rs", None);
    let moved =
        Finding::error("CAP-001", "core", "capability", "network is denied").at("src/net.rs", None);

    assert_eq!(first.fingerprint, same.fingerprint);
    assert_ne!(first.fingerprint, moved.fingerprint);
}

#[test]
fn ordering_is_path_then_location_then_rule() {
    let mut findings = vec![
        Finding::error("B", "b", "test", "second").at("b.rs", None),
        Finding::error("A", "a", "test", "first").at("a.rs", None),
    ];
    sort_findings(&mut findings);
    assert_eq!(findings[0].id, "A");
}

#[test]
fn finding_sink_counts_exact_groups_after_payload_truncation() {
    let mut findings = FindingSink::with_limit(DiagnosticLimit::Bounded(1));
    findings.push(Finding::error("TEST", "first", "test", "one"));
    findings.push(Finding::error("TEST", "first", "test", "two"));
    findings.push(Finding::error("NOTE", "second", "test", "three").with_severity(Severity::Note));

    assert_eq!(findings.iter().count(), 1);
    assert_eq!(findings.totals().severity[&Severity::Error], 2);
    assert_eq!(findings.totals().severity[&Severity::Note], 1);
    assert_eq!(findings.totals().groups.values().sum::<usize>(), 3);
    assert!(findings.iter().all(|finding| finding.id != "ZR-LIMIT-001"));
}
