//! Orchestration for human-authored contract comparison.

use crate::Contract;

use super::{
    ArchitectureChange, ChangeKind, analysis, boundaries, evidence, source, support, topology,
};

pub(super) fn compare(before: &Contract, after: &Contract) -> Vec<ArchitectureChange> {
    let mut changes = Vec::new();
    support::compare_named_set(
        "adapter",
        "repository",
        &before.adapters,
        &after.adapters,
        ChangeKind::Revoke,
        ChangeKind::Grant,
        "analyzes this language adapter",
        &mut changes,
    );
    boundaries::compare_repository(before, after, &mut changes);
    if before.repository.files != after.repository.files {
        changes.push(ArchitectureChange::new(
            ChangeKind::Unknown,
            "repository.files",
            "repository",
            "repository-file policy semantics require review while the new evaluator is under development",
        ));
    }
    analysis::compare(before, after, &mut changes);
    source::compare(before, after, &mut changes);
    topology::compare(before, after, &mut changes);
    boundaries::compare_scopes(before, after, &mut changes);
    boundaries::compare_owners(before, after, &mut changes);
    evidence::compare(before, after, &mut changes);
    changes
}
