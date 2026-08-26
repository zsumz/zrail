//! Exact type-shape and duplication policy changes remain review-visible.

use std::collections::{BTreeMap, BTreeSet};

use crate::{CloneCopyPolicy, Contract, DuplicationTrait, RustTypeContract, TypeProhibition};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    compare_global(before, after, changes);
    let old = policies(before);
    let new = policies(after);
    for name in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(name), new.get(name)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.type-policy",
                name,
                "adds exact type-shape or duplication enforcement",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.type-policy",
                name,
                "removes exact type-shape and duplication enforcement",
            )),
            (Some(left), Some(right)) => compare_existing(name, left, right, changes),
            (None, None) => {}
        }
    }
}

fn compare_global(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    compare_traits(
        "rust.duplication.import",
        &before.source.rust.duplication.deny_imports,
        &after.source.rust.duplication.deny_imports,
        changes,
    );
    compare_traits(
        "rust.duplication.macro-token",
        &before.source.rust.duplication.deny_macro_tokens,
        &after.source.rust.duplication.deny_macro_tokens,
        changes,
    );
    if before.source.rust.duplication.reachability != after.source.rust.duplication.reachability {
        let kind = if after.source.rust.duplication.reachability == crate::PolicyReachability::All {
            ChangeKind::Revoke
        } else {
            ChangeKind::Grant
        };
        changes.push(ArchitectureChange::new(
            kind,
            "rust.duplication.reachability",
            "source.rust.duplication",
            "changes source reachability covered by written duplication syntax policy",
        ));
    }
}

fn compare_traits(
    rail: &str,
    before: &[DuplicationTrait],
    after: &[DuplicationTrait],
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = before.iter().copied().collect::<BTreeSet<_>>();
    let new = after.iter().copied().collect::<BTreeSet<_>>();
    for value in new.difference(&old) {
        changes.push(ArchitectureChange::new(
            ChangeKind::Revoke,
            rail,
            trait_name(*value),
            "adds a written duplication prohibition",
        ));
    }
    for value in old.difference(&new) {
        changes.push(ArchitectureChange::new(
            ChangeKind::Grant,
            rail,
            trait_name(*value),
            "removes a written duplication prohibition",
        ));
    }
}

fn policies(contract: &Contract) -> BTreeMap<&str, &RustTypeContract> {
    contract
        .source
        .rust
        .types
        .iter()
        .map(|policy| (policy.name.as_str(), policy))
        .collect()
}

fn compare_existing(
    name: &str,
    left: &RustTypeContract,
    right: &RustTypeContract,
    changes: &mut Vec<ArchitectureChange>,
) {
    if left.identity != right.identity || left.path != right.path || left.kind != right.kind {
        changes.push(ArchitectureChange::new(
            ChangeKind::Unknown,
            "rust.type-policy.subject",
            name,
            "changes the exact governed type subject",
        ));
    }
    compare_prohibitions(name, left, right, changes);
    if left.clone_copy != right.clone_copy {
        changes.push(ArchitectureChange::new(
            if right.clone_copy == CloneCopyPolicy::Forbidden {
                ChangeKind::Revoke
            } else {
                ChangeKind::Grant
            },
            "rust.type-policy.clone-copy",
            name,
            "changes bundled Clone/Copy surface closure",
        ));
    }
    if left.reachability != right.reachability {
        changes.push(ArchitectureChange::new(
            if right.reachability == crate::PolicyReachability::All {
                ChangeKind::Revoke
            } else {
                ChangeKind::Grant
            },
            "rust.type-policy.reachability",
            name,
            "changes source reachability covered by the exact type policy",
        ));
    }
    if left.visibility != right.visibility
        || left.leaf_module != right.leaf_module
        || left.fields != right.fields
    {
        changes.push(ArchitectureChange::new(
            ChangeKind::Unknown,
            "rust.type-policy.shape",
            name,
            "changes the accepted exact Rust type representation",
        ));
    }
}

fn compare_prohibitions(
    name: &str,
    before: &RustTypeContract,
    after: &RustTypeContract,
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = effective_prohibitions(before);
    let new = effective_prohibitions(after);
    for value in new.difference(&old) {
        changes.push(ArchitectureChange::new(
            ChangeKind::Revoke,
            "rust.type-policy.prohibition",
            format!("{name}:{}", prohibition_name(*value)),
            "adds a per-type duplication prohibition",
        ));
    }
    for value in old.difference(&new) {
        changes.push(ArchitectureChange::new(
            ChangeKind::Grant,
            "rust.type-policy.prohibition",
            format!("{name}:{}", prohibition_name(*value)),
            "removes a per-type duplication prohibition",
        ));
    }
}

fn effective_prohibitions(policy: &RustTypeContract) -> BTreeSet<TypeProhibition> {
    if policy.clone_copy == CloneCopyPolicy::Forbidden {
        return [
            TypeProhibition::DeriveClone,
            TypeProhibition::DeriveCopy,
            TypeProhibition::ImplClone,
            TypeProhibition::ImplCopy,
            TypeProhibition::OpaqueExpansion,
        ]
        .into_iter()
        .collect();
    }
    policy.deny.iter().copied().collect()
}

const fn trait_name(value: DuplicationTrait) -> &'static str {
    match value {
        DuplicationTrait::Clone => "clone",
        DuplicationTrait::Copy => "copy",
    }
}

const fn prohibition_name(value: TypeProhibition) -> &'static str {
    match value {
        TypeProhibition::DeriveClone => "derive-clone",
        TypeProhibition::DeriveCopy => "derive-copy",
        TypeProhibition::ImplClone => "impl-clone",
        TypeProhibition::ImplCopy => "impl-copy",
        TypeProhibition::OpaqueExpansion => "opaque-expansion",
    }
}
