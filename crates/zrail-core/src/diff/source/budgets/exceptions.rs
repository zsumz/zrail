//! Above-hard permission and its accountability are explicit protected authority.

use std::collections::{BTreeMap, BTreeSet};

use crate::SizeExceptionContract;

use super::super::super::{ArchitectureChange, ChangeKind, support::compare_number};

pub(super) fn compare(
    before: &[SizeExceptionContract],
    after: &[SizeExceptionContract],
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = by_path(before);
    let new = by_path(after);
    for path in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        let subject = format!("rust:size:exception:{path}");
        match (old.get(path), new.get(path)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.file-size.exception",
                subject,
                "explicit above-hard permission added",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.file-size.exception",
                subject,
                "explicit above-hard permission removed",
            )),
            (Some(left), Some(right)) => {
                compare_number(
                    "rust.file-size.exception",
                    &subject,
                    left.hard,
                    right.hard,
                    changes,
                );
                if (&left.reason, &left.owner, &left.issue, &left.tracking)
                    != (&right.reason, &right.owner, &right.issue, &right.tracking)
                {
                    changes.push(ArchitectureChange::new(
                        ChangeKind::Unknown,
                        "rust.file-size.exception",
                        subject,
                        "exception justification or accountability changed",
                    ));
                }
            }
            (None, None) => {}
        }
    }
}

fn by_path(exceptions: &[SizeExceptionContract]) -> BTreeMap<&str, &SizeExceptionContract> {
    exceptions
        .iter()
        .map(|exception| (exception.path.as_str(), exception))
        .collect()
}

pub(super) fn metadata(
    before: crate::SizeExceptionMetadata,
    after: crate::SizeExceptionMetadata,
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = metadata_worlds(before);
    let new = metadata_worlds(after);
    for (changed, kind) in [
        (new & !old != 0, ChangeKind::Grant),
        (old & !new != 0, ChangeKind::Revoke),
    ] {
        if changed {
            changes.push(
                ArchitectureChange::new(
                    kind,
                    "rust.file-size.exception",
                    "source.rust.budgets.exception-metadata",
                    "permitted exception-accountability forms changed",
                )
                .values(format!("{before:?}"), format!("{after:?}")),
            );
        }
    }
}

const fn metadata_worlds(value: crate::SizeExceptionMetadata) -> u8 {
    use crate::SizeExceptionMetadata;
    match value {
        SizeExceptionMetadata::OwnerIssueOrTracking => 0b111,
        SizeExceptionMetadata::OwnerIssue => 0b101,
        SizeExceptionMetadata::Tracking => 0b110,
        SizeExceptionMetadata::OwnerIssueAndTracking => 0b100,
    }
}
