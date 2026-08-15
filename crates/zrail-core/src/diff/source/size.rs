//! Optional source-size policy changes preserve explicit permission direction.

use crate::{Contract, FileSizeContract};

use super::super::{ArchitectureChange, ChangeKind, support::compare_number};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    match (&before.source.rust.size, &after.source.rust.size) {
        (None, None) => {}
        (None, Some(_)) => changes.push(ArchitectureChange::new(
            ChangeKind::Revoke,
            "rust.file-size",
            "source.rust.size",
            "file-size policy became enforced",
        )),
        (Some(_), None) => changes.push(ArchitectureChange::new(
            ChangeKind::Grant,
            "rust.file-size",
            "source.rust.size",
            "file-size policy was removed",
        )),
        (Some(left), Some(right)) => compare_values(left, right, changes),
    }
}

fn compare_values(
    left: &FileSizeContract,
    right: &FileSizeContract,
    changes: &mut Vec<ArchitectureChange>,
) {
    for (name, old, new) in [
        ("facade", left.facade, right.facade),
        ("implementation", left.implementation, right.implementation),
        ("test", left.test, right.test),
        ("auxiliary", left.auxiliary, right.auxiliary),
    ] {
        compare_number(
            "rust.file-size",
            &format!("{name}.target"),
            old.target,
            new.target,
            changes,
        );
        compare_number(
            "rust.file-size",
            &format!("{name}.hard"),
            old.hard,
            new.hard,
            changes,
        );
    }
}
