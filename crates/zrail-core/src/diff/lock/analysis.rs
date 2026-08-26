//! Completeness-certificate authority changes are scoped and reviewable.

use crate::{LockFile, LockedAnalysis};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
    match (&before.analysis, &after.analysis) {
        (None, Some(_)) => changes.push(ArchitectureChange::new(
            ChangeKind::Revoke,
            "lock.analysis",
            "certificate",
            "the analyzed universe became content-bound",
        )),
        (Some(_), None) => changes.push(ArchitectureChange::new(
            ChangeKind::Grant,
            "lock.analysis",
            "certificate",
            "the analyzed universe is no longer content-bound",
        )),
        (Some(left), Some(right)) => compare_present(left, right, changes),
        (None, None) => {}
    }
}

fn compare_present(
    before: &LockedAnalysis,
    after: &LockedAnalysis,
    changes: &mut Vec<ArchitectureChange>,
) {
    changed(
        &before.inventory_sha256,
        &after.inventory_sha256,
        "inventory",
        "the canonical analyzed inventory changed",
        changes,
    );
    changed(
        &before.exclusions_sha256,
        &after.exclusions_sha256,
        "exclusions",
        "the analyzed exclusion set changed",
        changes,
    );
    if before.cargo_lock_sha256 != after.cargo_lock_sha256 {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "lock.analysis",
                "cargo-lock",
                "the exact resolved Cargo graph input changed",
            )
            .values(
                before.cargo_lock_sha256.as_deref().unwrap_or("<none>"),
                after.cargo_lock_sha256.as_deref().unwrap_or("<none>"),
            ),
        );
    }
    changed(
        &before.cargo_features_sha256,
        &after.cargo_features_sha256,
        "cargo-features",
        "Cargo feature definitions changed",
        changes,
    );
    changed(
        &before.feature_worlds_sha256,
        &after.feature_worlds_sha256,
        "feature-worlds",
        "configured or resolved Cargo feature worlds changed",
        changes,
    );
    if before.feature_worlds != after.feature_worlds {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "lock.analysis",
                "feature-world-count",
                "the number of exact Cargo feature worlds changed",
            )
            .values(
                before
                    .feature_worlds
                    .map_or_else(|| "<absent>".into(), |value| value.to_string()),
                after
                    .feature_worlds
                    .map_or_else(|| "<absent>".into(), |value| value.to_string()),
            ),
        );
    }
    if before.analyzer_semantics != after.analyzer_semantics {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "lock.analysis",
                "semantics",
                "the completeness certificate changed interpretation",
            )
            .values(
                before.analyzer_semantics.to_string(),
                after.analyzer_semantics.to_string(),
            ),
        );
    }
    if before.contract_sources != after.contract_sources {
        changes.push(ArchitectureChange::new(
            ChangeKind::Unknown,
            "lock.analysis",
            "contract-sources",
            "the exact contract-source census changed",
        ));
    }
}

fn changed(
    before: &str,
    after: &str,
    subject: &str,
    message: &str,
    changes: &mut Vec<ArchitectureChange>,
) {
    if before != after {
        changes.push(
            ArchitectureChange::new(ChangeKind::Unknown, "lock.analysis", subject, message)
                .values(before, after),
        );
    }
}
