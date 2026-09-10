//! Permission changes for exact Rust source-role overrides.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, FacadeMode, FileRole, FileRoleContract};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    let old = roles(before);
    let new = roles(after);
    for path in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        let left = old.get(path).copied();
        let right = new.get(path).copied();
        if let (Some(left), Some(right)) = (left, right)
            && left.role == right.role
            && left.role != FileRole::Implementation
        {
            super::super::support::compare_ordered_mode(
                "rust.facades",
                path,
                super::super::support::rank_facades(mode(left, before)),
                super::super::support::rank_facades(mode(right, after)),
                changes,
            );
        }
        match (old.get(path), new.get(path)) {
            (Some(left), Some(right)) if left.role != right.role => {
                change(path, Some(left.role), Some(right.role), changes);
            }
            (None, Some(right)) => change(path, None, Some(right.role), changes),
            (Some(left), None) => change(path, Some(left.role), None, changes),
            _ => {}
        }
    }
}

fn change(
    path: &str,
    before: Option<FileRole>,
    after: Option<FileRole>,
    changes: &mut Vec<ArchitectureChange>,
) {
    let kind = match (before, after) {
        (Some(FileRole::Facade), Some(FileRole::TestFacade))
        | (Some(FileRole::TestFacade), Some(FileRole::Facade)) => ChangeKind::Unknown,
        (_, Some(FileRole::Implementation))
        | (Some(FileRole::Facade | FileRole::TestFacade), None) => ChangeKind::Grant,
        (_, Some(FileRole::Facade | FileRole::TestFacade))
        | (Some(FileRole::Implementation), None) => ChangeKind::Revoke,
        (None, None) => return,
    };
    changes.push(
        ArchitectureChange::new(
            kind,
            "rust.file-role",
            path,
            "effective Rust source role changed",
        )
        .values(format!("{before:?}"), format!("{after:?}")),
    );
}

fn mode(role: &FileRoleContract, contract: &Contract) -> FacadeMode {
    role.mode.unwrap_or(contract.source.rust.facades)
}

fn roles(contract: &Contract) -> BTreeMap<&str, &FileRoleContract> {
    contract
        .source
        .rust
        .file_roles
        .iter()
        .map(|role| (role.path.as_str(), role))
        .collect()
}

#[cfg(test)]
#[path = "file_roles_test.rs"]
mod file_roles_test;
