//! Exact lock drift includes ratchet value changes.

use zrail_core::{
    FindingSink, LockFile, LockedGate, LockedGateInput, LockedGeneratedSource,
    LockedMacroImplementation, LockedMacroSource, LockedRatchet,
};

use super::compare_locks;

#[test]
fn changed_ratchet_values_are_not_treated_as_equal() {
    let mut current = LockFile::new("0".repeat(64));
    current.ratchets.push(ratchet(260));
    let mut candidate = LockFile::new("0".repeat(64));
    candidate.ratchets.push(ratchet(240));
    let mut findings = FindingSink::default();

    compare_locks(&current, &candidate, &mut findings);

    assert!(findings.iter().any(|finding| finding.id == "LOCK-010"));
}

#[test]
fn changed_generated_manifest_is_stale_lock_state() {
    let mut current = LockFile::new("0".repeat(64));
    current.generated.push(generated("1"));
    let mut candidate = LockFile::new("0".repeat(64));
    candidate.generated.push(generated("2"));
    let mut findings = FindingSink::default();

    compare_locks(&current, &candidate, &mut findings);

    assert!(findings.iter().any(|finding| finding.id == "LOCK-013"));
}

#[test]
fn changed_gate_contents_are_stale_lock_state() {
    let mut current = LockFile::new("0".repeat(64));
    current.gates.push(gate("1"));
    let mut candidate = LockFile::new("0".repeat(64));
    candidate.gates.push(gate("2"));
    let mut findings = FindingSink::default();

    compare_locks(&current, &candidate, &mut findings);

    assert!(findings.iter().any(|finding| finding.id == "LOCK-016"));
}

#[test]
fn gate_input_drift_is_exact_and_directional() {
    let mut current = LockFile::new("0".repeat(64));
    let mut candidate = LockFile::new("0".repeat(64));
    current.gates.push(gate_with_input("1"));
    candidate.gates.push(gate_with_input("2"));
    let mut findings = FindingSink::default();

    compare_locks(&current, &candidate, &mut findings);

    assert!(findings.iter().any(|finding| finding.id == "LOCK-026"));

    candidate.gates[0].inputs.clear();
    let mut stale = FindingSink::default();
    compare_locks(&current, &candidate, &mut stale);
    assert!(stale.iter().any(|finding| finding.id == "LOCK-025"));

    let mut added = FindingSink::default();
    compare_locks(&candidate, &current, &mut added);
    assert!(added.iter().any(|finding| finding.id == "LOCK-024"));
}

#[test]
fn changed_repository_macro_package_is_stale_lock_state() {
    let mut current = LockFile::new("0".repeat(64));
    current.macro_implementations.push(implementation("1"));
    let mut candidate = LockFile::new("0".repeat(64));
    candidate.macro_implementations.push(implementation("2"));
    let mut findings = FindingSink::default();

    compare_locks(&current, &candidate, &mut findings);

    assert!(findings.iter().any(|finding| finding.id == "LOCK-023"));
}

#[test]
fn same_name_macro_source_addition_is_not_hidden() {
    let mut current = LockFile::new("0".repeat(64));
    current
        .macro_sources
        .push(macro_source("derive-one", "1.0.0"));
    let mut candidate = current.clone();
    candidate
        .macro_sources
        .push(macro_source("derive-two", "2.0.0"));
    let mut findings = FindingSink::default();

    compare_locks(&current, &candidate, &mut findings);

    assert!(findings.iter().any(|finding| finding.id == "LOCK-036"));
}

#[test]
fn changed_feature_certificate_fields_are_stale_lock_state() {
    let current = LockFile::new("0".repeat(64));
    let mut candidate = current.clone();
    let analysis = candidate.analysis.as_mut().expect("analysis");
    analysis.cargo_features_sha256 = "1".repeat(64);
    analysis.feature_worlds_sha256 = "2".repeat(64);
    analysis.feature_worlds = Some(1);
    let mut findings = FindingSink::default();

    compare_locks(&current, &candidate, &mut findings);

    for id in ["LOCK-043", "LOCK-044", "LOCK-045"] {
        assert!(findings.iter().any(|finding| finding.id == id));
    }
}

fn ratchet(value: usize) -> LockedRatchet {
    LockedRatchet {
        rule: "rust.file-size".into(),
        selector: None,
        target: "src/large.rs".into(),
        value,
    }
}

fn generated(digit: &str) -> LockedGeneratedSource {
    LockedGeneratedSource {
        root: "src/generated".into(),
        manifest_sha256: digit.repeat(64),
    }
}

fn gate(digit: &str) -> LockedGate {
    LockedGate {
        name: "check".into(),
        path: "scripts/check".into(),
        sha256: digit.repeat(64),
        inputs: Vec::new(),
    }
}

fn gate_with_input(digit: &str) -> LockedGate {
    let mut gate = gate("1");
    gate.inputs.push(LockedGateInput {
        path: "scripts/helper".into(),
        sha256: digit.repeat(64),
    });
    gate
}

fn implementation(digit: &str) -> LockedMacroImplementation {
    LockedMacroImplementation {
        package: "fixture".into(),
        directory: ".".into(),
        inputs_sha256: digit.repeat(64),
    }
}

fn macro_source(package: &str, version: &str) -> LockedMacroSource {
    LockedMacroSource {
        allowance: "derive".into(),
        package: package.into(),
        version: version.into(),
        source: "path+file:///fixture".into(),
        checksum: None,
    }
}
