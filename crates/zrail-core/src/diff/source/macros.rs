//! Macro expansion, input, and local-definition authority changes.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, MacroBindingMode, MacroExpansionBindings, MacroInputMode};

use super::super::{
    ArchitectureChange, ChangeKind,
    support::{compare_ordered_mode, rank_macro_expansion},
};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    compare_ordered_mode(
        "rust.macro-expansion",
        "source.rust.macros.mode",
        rank_macro_expansion(before.source.rust.macros.mode),
        rank_macro_expansion(after.source.rust.macros.mode),
        changes,
    );
    let old = allowances(before);
    let new = allowances(after);
    for name in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(name), new.get(name)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.macro-expansion.allow",
                name,
                "trusts an uninspected macro expansion",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.macro-expansion.allow",
                name,
                "no longer trusts an uninspected macro expansion",
            )),
            (Some(left), Some(right)) => compare_existing(name, left, right, changes),
            (None, None) => {}
        }
    }
}

fn allowances(contract: &Contract) -> BTreeMap<&str, &crate::MacroExpansionAllow> {
    contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .map(|allowed| (allowed.name.as_str(), allowed))
        .collect()
}

fn compare_existing(
    name: &str,
    left: &crate::MacroExpansionAllow,
    right: &crate::MacroExpansionAllow,
    changes: &mut Vec<ArchitectureChange>,
) {
    if left.binding != right.binding {
        let kind = if right.binding == MacroBindingMode::Conservative {
            ChangeKind::Grant
        } else {
            ChangeKind::Revoke
        };
        changes.push(ArchitectureChange::new(
            kind,
            "rust.macro-binding",
            name,
            "changes whether unresolved written macro names may bind",
        ));
    }
    if left.inputs != right.inputs {
        let kind = if right.inputs == MacroInputMode::Opaque {
            ChangeKind::Grant
        } else {
            ChangeKind::Revoke
        };
        changes.push(ArchitectureChange::new(
            kind,
            "rust.macro-input",
            name,
            "changes whether opaque macro input is trusted",
        ));
    }
    if left.bindings != right.bindings {
        let kind = if right.bindings == MacroExpansionBindings::None {
            ChangeKind::Grant
        } else {
            ChangeKind::Revoke
        };
        changes.push(ArchitectureChange::new(
            kind,
            "rust.macro-bindings",
            name,
            "changes whether reviewed expansion may replace items or introduce lexical bindings",
        ));
    }
    if left.definition != right.definition {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "rust.macro-definition",
                name,
                "trusted local macro definition path changed",
            )
            .values(
                left.definition.as_deref().unwrap_or("<none>"),
                right.definition.as_deref().unwrap_or("<none>"),
            ),
        );
    }
    if left.source != right.source {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "rust.macro-source",
                name,
                "trusted external macro dependency source changed",
            )
            .values(
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
