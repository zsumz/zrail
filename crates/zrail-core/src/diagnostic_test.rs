//! Stable diagnostic ordering and fingerprints.

use super::{Finding, FindingSink, MAX_REPORT_FINDINGS, sort_findings};

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
fn finding_sink_caps_output_and_marks_omissions_unresolved() {
    let mut findings = FindingSink::default();
    for index in 0..=MAX_REPORT_FINDINGS {
        findings.push(Finding::error("TEST", "test", "test", index.to_string()));
    }

    let findings = findings.into_findings();

    assert_eq!(findings.len(), MAX_REPORT_FINDINGS);
    assert_eq!(
        findings.last().map(|finding| finding.id.as_str()),
        Some("ZR-LIMIT-001")
    );
}
