//! Permission direction for exact test-mirror execution authority.

use std::collections::{BTreeMap, BTreeSet};

use crate::TestMirrorContract;

use super::super::{ArchitectureChange, ChangeKind, support::compare_named_set};

pub(super) fn compare(
    before: &[TestMirrorContract],
    after: &[TestMirrorContract],
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = by_production(before);
    let new = by_production(after);
    for production in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(production), new.get(production)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.test-mirror",
                production,
                "production source gained an exact execution-backed test mirror",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.test-mirror",
                production,
                "production source lost its exact execution-backed test mirror",
            )),
            (Some(left), Some(right)) if identity(left) != identity(right) => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Unknown,
                    "rust.test-mirror",
                    production,
                    "test mirror identity or receipt path changed and requires review",
                )
                .values(identity(left), identity(right)),
            ),
            (Some(left), Some(right)) => compare_context(left, right, changes),
            _ => {}
        }
    }
}

fn compare_context(
    before: &TestMirrorContract,
    after: &TestMirrorContract,
    changes: &mut Vec<ArchitectureChange>,
) {
    compare_named_set(
        "rust.test-mirror-input",
        &before.production,
        &before.inputs,
        &after.inputs,
        ChangeKind::Revoke,
        ChangeKind::Grant,
        "content-addresses this execution input",
        changes,
    );
    if before.execution != after.execution {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "rust.test-mirror-execution",
                &before.production,
                "test command, package, features, target, or toolchain changed",
            )
            .values(execution_identity(before), execution_identity(after)),
        );
    }
}

fn execution_identity(mirror: &TestMirrorContract) -> String {
    let execution = &mirror.execution;
    format!(
        "{}|package={}|default-features={}|features={}|target={}|toolchain={}",
        execution.command,
        execution.package,
        execution.default_features,
        execution.features.join(","),
        execution.target,
        execution.toolchain
    )
}

fn by_production(mirrors: &[TestMirrorContract]) -> BTreeMap<&str, &TestMirrorContract> {
    mirrors
        .iter()
        .map(|mirror| (mirror.production.as_str(), mirror))
        .collect()
}

fn identity(mirror: &TestMirrorContract) -> String {
    format!("{}::{}@{}", mirror.test, mirror.name, mirror.receipt)
}
