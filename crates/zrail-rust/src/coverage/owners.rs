//! Owner coverage reuses the exact source-operation matcher used by enforcement.

mod occurrences;

use zrail_core::{OwnerKind, PolicyReachability};

use crate::engine::RepositoryModel;

use super::GovernedOwnerRule;
use occurrences::occurrences;

pub(super) fn report(model: &RepositoryModel) -> Vec<GovernedOwnerRule> {
    let mut owners = model
        .bundle
        .contract
        .owners
        .iter()
        .map(|owner| {
            let kind = owner_kind(owner.kind).to_owned();
            let mut occurrences = occurrences(model, owner);
            occurrences.sort();
            GovernedOwnerRule {
                policy_id: format!("owner:{kind}:{}", owner.name),
                name: owner.name.clone(),
                kind,
                target: owner.selector.clone(),
                mutating_methods: sorted(&owner.mutating_methods),
                reachability: reachability(owner.reachability).into(),
                within: sorted(&owner.within),
                allow: sorted(&owner.allow),
                reason: owner.reason.clone(),
                occurrences,
            }
        })
        .collect::<Vec<_>>();
    owners.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
    owners
}

pub(super) fn sorted(values: &[String]) -> Vec<String> {
    let mut values = values.to_vec();
    values.sort();
    values.dedup();
    values
}

const fn owner_kind(kind: OwnerKind) -> &'static str {
    match kind {
        OwnerKind::Call => "call",
        OwnerKind::Capability => "capability",
        OwnerKind::Directory => "directory",
        OwnerKind::TypeConstruction => "type-construction",
        OwnerKind::MethodName => "method-name",
        OwnerKind::FieldRead => "field-read",
        OwnerKind::FieldWrite => "field-write",
        OwnerKind::FieldMutableBorrow => "field-mutable-borrow",
        OwnerKind::FieldMutation => "field-mutation",
        OwnerKind::FieldAuthority => "field-authority",
    }
}

const fn reachability(value: PolicyReachability) -> &'static str {
    match value {
        PolicyReachability::All => "all",
        PolicyReachability::Production => "production",
    }
}
