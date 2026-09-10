//! File assertions compare accepted states, never TOML order or numerical direction alone.

mod literal;
mod predicate;
mod selection;

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, RepositoryFileRule};

use super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    let old = named(&before.repository.files);
    let new = named(&after.repository.files);
    for name in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(name), new.get(name)) {
            (Some(left), Some(right)) => {
                for kind in selection::compare(left, right) {
                    changed(changes, name, "selection", kind, left, right);
                }
                for kind in predicate::compare(&left.predicate, &right.predicate) {
                    changed(
                        changes,
                        name,
                        "predicate",
                        kind,
                        &left.predicate,
                        &right.predicate,
                    );
                }
                if left.reason != right.reason {
                    changed(
                        changes,
                        name,
                        "justification",
                        ChangeKind::Unknown,
                        &left.reason,
                        &right.reason,
                    );
                }
            }
            (Some(rule), None) => changed(
                changes,
                name,
                "required-guard",
                ChangeKind::Grant,
                rule,
                &"removed",
            ),
            (None, Some(rule)) => changed(
                changes,
                name,
                "required-guard",
                ChangeKind::Revoke,
                &"absent",
                rule,
            ),
            (None, None) => {}
        }
    }
}

fn named(rules: &[RepositoryFileRule]) -> BTreeMap<&str, &RepositoryFileRule> {
    rules
        .iter()
        .map(|rule| (rule.name.as_str(), rule))
        .collect()
}

fn changed(
    changes: &mut Vec<ArchitectureChange>,
    name: &str,
    field: &str,
    kind: ChangeKind,
    before: &impl std::fmt::Debug,
    after: &impl std::fmt::Debug,
) {
    changes.push(
        ArchitectureChange::new(
            kind,
            "repository.files",
            format!("repository:file:{name}"),
            format!(
                "repository-file {field} changed; comparison governs physical/raw predicates only"
            ),
        )
        .values(format!("{before:?}"), format!("{after:?}")),
    );
}

fn set(values: &[String]) -> BTreeSet<&str> {
    values.iter().map(String::as_str).collect()
}

fn interval(left: (usize, Option<usize>), right: (usize, Option<usize>)) -> Vec<ChangeKind> {
    let mut kinds = Vec::new();
    if right.0 != left.0 {
        kinds.push(if right.0 > left.0 {
            ChangeKind::Revoke
        } else {
            ChangeKind::Grant
        });
    }
    let (left, right) = (left.1.unwrap_or(usize::MAX), right.1.unwrap_or(usize::MAX));
    if right != left {
        kinds.push(if right < left {
            ChangeKind::Revoke
        } else {
            ChangeKind::Grant
        });
    }
    kinds.sort();
    kinds.dedup();
    kinds
}

#[cfg(test)]
#[path = "files_test.rs"]
mod files_test;
