//! Macro expansion, input, and local-definition authority changes.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Contract, MacroAsyncSyntax, MacroBindingMode, MacroDuplicationEffect, MacroExpansionBindings,
    MacroExpansionMode, MacroFieldMutation, MacroInputMode, MacroSourceOperations,
};

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
    if before.source.rust.macros.mode != MacroExpansionMode::DenyUnreviewed
        || after.source.rust.macros.mode != MacroExpansionMode::DenyUnreviewed
    {
        return;
    }
    let old = allowances(before);
    let new = allowances(after);
    for name in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        compare_group(
            name,
            old.get(name).map(Vec::as_slice).unwrap_or_default(),
            new.get(name).map(Vec::as_slice).unwrap_or_default(),
            changes,
        );
    }
}

fn allowances(contract: &Contract) -> BTreeMap<&str, Vec<&crate::MacroExpansionAllow>> {
    let mut grouped = BTreeMap::<_, Vec<_>>::new();
    for allowance in &contract.source.rust.macros.allow {
        grouped
            .entry(allowance.name.as_str())
            .or_default()
            .push(allowance);
    }
    grouped
}

fn compare_group(
    name: &str,
    old: &[&crate::MacroExpansionAllow],
    new: &[&crate::MacroExpansionAllow],
    changes: &mut Vec<ArchitectureChange>,
) {
    if let ([left], [right]) = (old, new) {
        compare_existing(name, left, right, changes);
        return;
    }
    let old = by_provenance(old);
    let new = by_provenance(new);
    for provenance in old.keys().chain(new.keys()).collect::<BTreeSet<_>>() {
        let subject = format!("{name} [{provenance}]");
        match (old.get(provenance), new.get(provenance)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.macro-expansion.allow",
                subject,
                "trusts an uninspected macro expansion from this provenance",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.macro-expansion.allow",
                subject,
                "no longer trusts an uninspected macro expansion from this provenance",
            )),
            (Some(left), Some(right)) => compare_existing(&subject, left, right, changes),
            (None, None) => {}
        }
    }
}

fn by_provenance<'a>(
    allowances: &[&'a crate::MacroExpansionAllow],
) -> BTreeMap<String, &'a crate::MacroExpansionAllow> {
    allowances
        .iter()
        .map(|allowance| (provenance(allowance), *allowance))
        .collect()
}

fn provenance(allowance: &crate::MacroExpansionAllow) -> String {
    if let Some(definition) = &allowance.definition {
        format!("definition:{definition}")
    } else if let Some(source) = &allowance.source {
        format!("source:{}", source.identity())
    } else {
        "unbound".into()
    }
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
    if left.async_syntax != right.async_syntax {
        let kind = if right.async_syntax == MacroAsyncSyntax::None {
            ChangeKind::Grant
        } else {
            ChangeKind::Revoke
        };
        changes.push(ArchitectureChange::new(
            kind,
            "rust.macro-async-syntax",
            name,
            "changes whether reviewed expansion is trusted to introduce no async syntax",
        ));
    }
    if left.duplication_effect != right.duplication_effect {
        let kind = if right.duplication_effect == MacroDuplicationEffect::None {
            ChangeKind::Grant
        } else {
            ChangeKind::Revoke
        };
        changes.push(ArchitectureChange::new(
            kind,
            "rust.macro-duplication-effect",
            name,
            "changes whether reviewed expansion is trusted to add no Clone/Copy implementation",
        ));
    }
    if left.source_operations != right.source_operations {
        let kind = if right.source_operations == MacroSourceOperations::None {
            ChangeKind::Grant
        } else {
            ChangeKind::Revoke
        };
        changes.push(ArchitectureChange::new(
            kind,
            "rust.macro-source-operations",
            name,
            "changes whether reviewed expansion is trusted to introduce no source operations",
        ));
    }
    if left.field_mutation != right.field_mutation {
        let kind = if right.field_mutation == MacroFieldMutation::None {
            ChangeKind::Grant
        } else {
            ChangeKind::Revoke
        };
        changes.push(ArchitectureChange::new(
            kind,
            "rust.macro-field-mutation",
            name,
            "changes whether reviewed expansion is trusted to introduce no field mutation",
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
                "trusted macro implementation source changed",
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
