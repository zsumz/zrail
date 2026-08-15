//! Acyclic qualification paths must terminate at local gates.

use std::collections::{BTreeMap, BTreeSet};

use super::super::{GateContract, GateKind, validate_limits::ValidationErrors};

pub(super) fn validate(gates: &BTreeMap<&str, &GateContract>, errors: &mut ValidationErrors) {
    for gate in gates.values() {
        if reaches(&gate.name, &gate.name, gates, &mut BTreeSet::new()) {
            errors.push(format!(
                "gate graph contains a cycle through {:?}",
                gate.name
            ));
        }
        if gate.kind != GateKind::Local && !reaches_local(&gate.name, gates, &mut BTreeSet::new()) {
            errors.push(format!(
                "{:?} gate {:?} is not connected to a local gate",
                gate.kind, gate.name
            ));
        }
    }
}

pub(super) fn mark_required(
    name: &str,
    gates: &BTreeMap<&str, &GateContract>,
    used: &mut BTreeSet<String>,
) {
    let Some(gate) = gates.get(name) else {
        return;
    };
    for required in &gate.requires {
        if used.insert(required.clone()) {
            mark_required(required, gates, used);
        }
    }
}

fn reaches(
    origin: &str,
    current: &str,
    gates: &BTreeMap<&str, &GateContract>,
    seen: &mut BTreeSet<String>,
) -> bool {
    let Some(gate) = gates.get(current) else {
        return false;
    };
    gate.requires.iter().any(|required| {
        required == origin
            || (seen.insert(required.clone()) && reaches(origin, required, gates, seen))
    })
}

fn reaches_local(
    current: &str,
    gates: &BTreeMap<&str, &GateContract>,
    seen: &mut BTreeSet<String>,
) -> bool {
    let Some(gate) = gates.get(current) else {
        return false;
    };
    gate.kind == GateKind::Local
        || gate
            .requires
            .iter()
            .any(|required| seen.insert(required.clone()) && reaches_local(required, gates, seen))
}
