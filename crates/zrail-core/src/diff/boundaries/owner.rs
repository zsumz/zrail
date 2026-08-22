//! Permission changes for source-owner reachability.

use crate::{OwnerContract, PolicyReachability};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare_reachability(
    before: &OwnerContract,
    after: &OwnerContract,
    changes: &mut Vec<ArchitectureChange>,
) {
    if before.reachability == after.reachability {
        return;
    }
    let kind = if after.reachability == PolicyReachability::Production {
        ChangeKind::Grant
    } else {
        ChangeKind::Revoke
    };
    changes.push(
        ArchitectureChange::new(
            kind,
            "owner.reachability",
            &before.name,
            "owner source reachability changed",
        )
        .values(
            format!("{:?}", before.reachability),
            format!("{:?}", after.reachability),
        ),
    );
}
