//! Exact lock drift for reviewed qualification-gate contents.

use std::collections::BTreeMap;

use zrail_core::{Finding, FindingSink, LockFile, LockedGate, LockedGateInput};

pub(super) fn compare(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let old = by_name(&current.gates);
    let new = by_name(&candidate.gates);
    for name in new.keys().filter(|name| !old.contains_key(*name)) {
        findings.push(Finding::error(
            "LOCK-014",
            "lock.qualification-gate",
            "lock",
            format!("qualification gate {name:?} has no reviewed digest in zrail.lock"),
        ));
    }
    for name in old.keys().filter(|name| !new.contains_key(*name)) {
        findings.push(Finding::error(
            "LOCK-015",
            "lock.qualification-gate",
            "lock",
            format!("zrail.lock retains stale qualification gate {name:?}"),
        ));
    }
    for (name, old_gate) in old {
        let Some(new_gate) = new.get(name) else {
            continue;
        };
        if old_gate.path != new_gate.path || old_gate.sha256 != new_gate.sha256 {
            findings.push(Finding::error(
                "LOCK-016",
                "lock.qualification-gate",
                "lock",
                format!("reviewed qualification gate {name:?} changed"),
            ));
        }
        compare_inputs(name, old_gate, new_gate, findings);
    }
}

fn compare_inputs(
    name: &str,
    current: &LockedGate,
    candidate: &LockedGate,
    findings: &mut FindingSink,
) {
    let old = inputs_by_path(&current.inputs);
    let new = inputs_by_path(&candidate.inputs);
    for path in new.keys().filter(|path| !old.contains_key(*path)) {
        findings.push(input_finding(
            "LOCK-024",
            name,
            path,
            "has no reviewed digest in zrail.lock",
        ));
    }
    for path in old.keys().filter(|path| !new.contains_key(*path)) {
        findings.push(input_finding(
            "LOCK-025",
            name,
            path,
            "is retained stale in zrail.lock",
        ));
    }
    for (path, old_input) in old {
        if new
            .get(path)
            .is_some_and(|new_input| old_input.sha256 != new_input.sha256)
        {
            findings.push(input_finding(
                "LOCK-026",
                name,
                path,
                "changed after review",
            ));
        }
    }
}

fn input_finding(id: &str, gate: &str, path: &str, state: &str) -> Finding {
    Finding::error(
        id,
        "lock.qualification-gate-input",
        "lock",
        format!("qualification gate {gate:?} input {path:?} {state}"),
    )
}

fn by_name(gates: &[LockedGate]) -> BTreeMap<&str, &LockedGate> {
    gates
        .iter()
        .map(|gate| (gate.name.as_str(), gate))
        .collect()
}

fn inputs_by_path(inputs: &[LockedGateInput]) -> BTreeMap<&str, &LockedGateInput> {
    inputs
        .iter()
        .map(|input| (input.path.as_str(), input))
        .collect()
}
