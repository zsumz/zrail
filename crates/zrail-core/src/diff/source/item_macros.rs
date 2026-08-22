//! Scoped item-macro authority changes fail closed under semantic comparison.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, ItemMacroContract, MacroBindingMode};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    let old = allowances(before);
    let new = allowances(after);
    for selector in old
        .keys()
        .chain(new.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(&selector), new.get(&selector)) {
            (None, Some(_)) => changes.push(change(
                ChangeKind::Grant,
                &selector,
                "contract now trusts an item macro not to create source edges",
            )),
            (Some(_), None) => changes.push(change(
                ChangeKind::Revoke,
                &selector,
                "contract no longer trusts an item macro not to create source edges",
            )),
            (Some(left), Some(right)) => compare_existing(&selector, left, right, changes),
            (None, None) => {}
        }
    }
}

fn allowances(contract: &Contract) -> BTreeMap<String, &ItemMacroContract> {
    contract
        .source
        .rust
        .item_macros
        .iter()
        .map(|allowance| (selector_identity(allowance), allowance))
        .collect()
}

fn selector_identity(allowance: &ItemMacroContract) -> String {
    let selector = allowance.path.as_ref().map_or_else(
        || {
            if allowance.within.is_empty() {
                "repository".into()
            } else {
                let mut patterns = allowance.within.clone();
                patterns.sort();
                format!("within={}", patterns.join(","))
            }
        },
        |path| format!("path={path}"),
    );
    format!("{}:{selector}", allowance.name)
}

fn compare_existing(
    selector: &str,
    left: &ItemMacroContract,
    right: &ItemMacroContract,
    changes: &mut Vec<ArchitectureChange>,
) {
    if left.binding != right.binding {
        let kind = binding_change(left.binding, right.binding);
        changes.push(change(kind, selector, "item-macro origin binding changed"));
    }
    if left.source != right.source {
        let kind = match (&left.source, &right.source) {
            (None, Some(_)) => ChangeKind::Revoke,
            (Some(_), None) => ChangeKind::Grant,
            _ => ChangeKind::Unknown,
        };
        changes.push(
            change(kind, selector, "item-macro dependency provenance changed").values(
                left.source
                    .as_ref()
                    .map_or_else(|| "<none>".into(), crate::CrateRootSource::identity),
                right
                    .source
                    .as_ref()
                    .map_or_else(|| "<none>".into(), crate::CrateRootSource::identity),
            ),
        );
    }
}

const fn binding_change(
    before: Option<MacroBindingMode>,
    after: Option<MacroBindingMode>,
) -> ChangeKind {
    match (before, after) {
        (None, Some(_)) | (Some(MacroBindingMode::Conservative), Some(MacroBindingMode::Exact)) => {
            ChangeKind::Revoke
        }
        (Some(_), None) | (Some(MacroBindingMode::Exact), Some(MacroBindingMode::Conservative)) => {
            ChangeKind::Grant
        }
        _ => ChangeKind::Neutral,
    }
}

fn change(kind: ChangeKind, selector: &str, message: &str) -> ArchitectureChange {
    ArchitectureChange::new(kind, "rust.source-graph.item-macro", selector, message)
}

#[cfg(test)]
#[path = "item_macros_test.rs"]
mod item_macros_test;
