//! Exact lock drift for content-bound local macro definitions.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{Finding, FindingSink, LockFile, LockedMacroDefinition};

pub(super) fn compare(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let old = by_identity(&current.macros);
    let new = by_identity(&candidate.macros);
    for identity in old
        .keys()
        .chain(new.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(&identity), new.get(&identity)) {
            (None, Some(_)) => findings.push(Finding::error(
                "LOCK-017",
                "lock.macro-definition",
                "lock",
                format!("local macro definition {identity:?} is not reviewed in zrail.lock"),
            )),
            (Some(_), None) => findings.push(Finding::error(
                "LOCK-018",
                "lock.macro-definition",
                "lock",
                format!("zrail.lock retains stale local macro definition {identity:?}"),
            )),
            (Some(left), Some(right)) if left.sha256 != right.sha256 => {
                findings.push(Finding::error(
                    "LOCK-019",
                    "lock.macro-definition",
                    "lock",
                    format!("reviewed local macro definition {identity:?} changed"),
                ));
            }
            _ => {}
        }
    }
}

fn by_identity(definitions: &[LockedMacroDefinition]) -> BTreeMap<String, &LockedMacroDefinition> {
    definitions
        .iter()
        .map(|definition| {
            (
                format!(
                    "{}:{}:{}",
                    definition.path, definition.name, definition.ordinal
                ),
                definition,
            )
        })
        .collect()
}
