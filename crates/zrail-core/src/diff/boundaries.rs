//! Repository, source-scope, and ownership boundary changes.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, OwnerContract, ScopeContract};

use super::{
    ArchitectureChange, ChangeKind,
    support::{compare_named_set, compare_ordered_mode, rank_policy, rank_symlinks},
};

pub(super) fn compare_repository(
    before: &Contract,
    after: &Contract,
    changes: &mut Vec<ArchitectureChange>,
) {
    compare_named_set(
        "repository.root",
        "repository",
        &before.repository.roots,
        &after.repository.roots,
        ChangeKind::Revoke,
        ChangeKind::Grant,
        "checks repository source",
        changes,
    );
    compare_named_set(
        "repository.exclude",
        "repository",
        &before.repository.exclude,
        &after.repository.exclude,
        ChangeKind::Grant,
        ChangeKind::Revoke,
        "excludes repository source",
        changes,
    );
    compare_ordered_mode(
        "repository.nested-git",
        "repository.nested_git",
        rank_policy(before.repository.nested_git),
        rank_policy(after.repository.nested_git),
        changes,
    );
    compare_ordered_mode(
        "repository.submodules",
        "repository.submodules",
        rank_policy(before.repository.submodules),
        rank_policy(after.repository.submodules),
        changes,
    );
    compare_ordered_mode(
        "repository.symlinks",
        "repository.symlinks",
        rank_symlinks(before.repository.symlinks),
        rank_symlinks(after.repository.symlinks),
        changes,
    );
}

pub(super) fn compare_scopes(
    before: &Contract,
    after: &Contract,
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = scopes_by_name(&before.scopes);
    let new = scopes_by_name(&after.scopes);
    let names = old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for name in names {
        match (old.get(name), new.get(name)) {
            (Some(left), Some(right)) => compare_scope(left, right, changes),
            (None, Some(scope)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "symbol.scope",
                &scope.name,
                "new scope introduces symbol restrictions",
            )),
            (Some(scope), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "symbol.scope",
                &scope.name,
                "scope and its symbol restrictions were removed",
            )),
            (None, None) => {}
        }
    }
}

pub(super) fn compare_owners(
    before: &Contract,
    after: &Contract,
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = owners_by_name(&before.owners);
    let new = owners_by_name(&after.owners);
    let names = old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for name in names {
        match (old.get(name), new.get(name)) {
            (Some(left), Some(right)) => {
                if left.kind != right.kind {
                    changes.push(
                        ArchitectureChange::new(
                            ChangeKind::Unknown,
                            "owner.kind",
                            name,
                            "ownership kind changed and requires review",
                        )
                        .values(format!("{:?}", left.kind), format!("{:?}", right.kind)),
                    );
                }
                if left.selector != right.selector {
                    changes.push(
                        ArchitectureChange::new(
                            ChangeKind::Unknown,
                            "owner.selector",
                            name,
                            "ownership selector changed and requires review",
                        )
                        .values(&left.selector, &right.selector),
                    );
                }
                compare_named_set(
                    "owner.within",
                    name,
                    &left.within,
                    &right.within,
                    ChangeKind::Revoke,
                    ChangeKind::Grant,
                    "enforces ownership within this source scope",
                    changes,
                );
                compare_named_set(
                    "owner.allow",
                    name,
                    &left.allow,
                    &right.allow,
                    ChangeKind::Grant,
                    ChangeKind::Revoke,
                    "may own this architectural resource",
                    changes,
                );
            }
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "owner",
                name,
                "new ownership boundary was declared",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "owner",
                name,
                "ownership boundary was removed",
            )),
            (None, None) => {}
        }
    }
}

fn compare_scope(
    before: &ScopeContract,
    after: &ScopeContract,
    changes: &mut Vec<ArchitectureChange>,
) {
    compare_named_set(
        "symbol.scope-include",
        &before.name,
        &before.include,
        &after.include,
        ChangeKind::Revoke,
        ChangeKind::Grant,
        "applies restrictions to source",
        changes,
    );
    compare_named_set(
        "symbol.scope-exclude",
        &before.name,
        &before.exclude,
        &after.exclude,
        ChangeKind::Grant,
        ChangeKind::Revoke,
        "exempts source from restrictions",
        changes,
    );
    compare_named_set(
        "symbol.deny",
        &before.name,
        &before.symbols.deny,
        &after.symbols.deny,
        ChangeKind::Revoke,
        ChangeKind::Grant,
        "denies an exact symbol",
        changes,
    );
}

fn scopes_by_name(scopes: &[ScopeContract]) -> BTreeMap<&str, &ScopeContract> {
    scopes
        .iter()
        .map(|scope| (scope.name.as_str(), scope))
        .collect()
}

fn owners_by_name(owners: &[OwnerContract]) -> BTreeMap<&str, &OwnerContract> {
    owners
        .iter()
        .map(|owner| (owner.name.as_str(), owner))
        .collect()
}
