//! Permission changes for exact Rust source-role overrides.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, FileRole};

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
        match (old.get(path), new.get(path)) {
            (Some(left), Some(right)) if left != right => {
                change(path, Some(**left), Some(**right), changes);
            }
            (None, Some(right)) => change(path, None, Some(**right), changes),
            (Some(left), None) => change(path, Some(**left), None, changes),
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
        (_, Some(FileRole::Implementation)) | (Some(FileRole::Facade), None) => ChangeKind::Grant,
        (_, Some(FileRole::Facade)) | (Some(FileRole::Implementation), None) => ChangeKind::Revoke,
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

fn roles(contract: &Contract) -> BTreeMap<&str, &FileRole> {
    contract
        .source
        .rust
        .file_roles
        .iter()
        .map(|role| (role.path.as_str(), &role.role))
        .collect()
}

#[cfg(test)]
#[path = "file_roles_test.rs"]
mod file_roles_test;
