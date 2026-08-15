//! Exact lock drift includes ratchet value changes.

use zrail_core::{FindingSink, LockFile, LockedGate, LockedGeneratedSource, LockedRatchet};

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

fn ratchet(value: usize) -> LockedRatchet {
    LockedRatchet {
        rule: "rust.file-size".into(),
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
    }
}
