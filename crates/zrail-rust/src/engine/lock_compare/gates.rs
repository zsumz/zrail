//! Exact lock drift for reviewed qualification-gate contents.

use std::collections::BTreeMap;

use zrail_core::{Finding, FindingSink, LockFile, LockedGate};

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
        if old_gate != *new_gate {
            findings.push(Finding::error(
                "LOCK-016",
                "lock.qualification-gate",
                "lock",
                format!("reviewed qualification gate {name:?} changed"),
            ));
        }
    }
}

fn by_name(gates: &[LockedGate]) -> BTreeMap<&str, &LockedGate> {
    gates
        .iter()
        .map(|gate| (gate.name.as_str(), gate))
        .collect()
}
