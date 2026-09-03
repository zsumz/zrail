//! Staged macro reviews are inert until the enforcement mode changes.

use super::{ChangeKind, compare_architecture, compare_fixture_test::contract_with_hard_limit};
use crate::{
    CrateRootSource, LockedMacroImplementation, MacroAsyncSyntax, MacroBindingMode,
    MacroDuplicationEffect, MacroExpansionAllow, MacroExpansionBindings, MacroExpansionMode,
    MacroFieldMutation, MacroInputMode, MacroSourceOperations,
};

#[test]
fn staging_allowances_is_neutral_and_the_mode_flip_is_a_revoke() {
    let before = contract_with_hard_limit(300);
    let mut staged = before.clone();
    staged.source.rust.macros.allow.push(allowance());

    let staging = compare_architecture(&before, None, &staged, None);
    assert!(!staging.changes.iter().any(|change| {
        change.rail.starts_with("rust.macro-") || change.rail == "rust.macro-expansion.allow"
    }));

    let mut enforced = staged.clone();
    enforced.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    let flip = compare_architecture(&staged, None, &enforced, None);
    assert!(flip.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke && change.rail == "rust.macro-expansion"
    }));
    assert!(!flip.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "rust.macro-expansion.allow"
    }));
}

#[test]
fn same_name_provenance_removal_is_not_hidden() {
    let mut before = contract_with_hard_limit(300);
    before.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    before.source.rust.macros.allow = vec![allowance_from("=1.0.0"), allowance_from("=2.0.0")];
    let mut after = before.clone();
    after.source.rust.macros.allow.pop();

    let report = compare_architecture(&before, None, &after, None);
    let changes = report
        .changes
        .iter()
        .filter(|change| change.rail == "rust.macro-expansion.allow")
        .collect::<Vec<_>>();

    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].kind, ChangeKind::Revoke);
    assert!(changes[0].subject.contains("=2.0.0"));
}

#[test]
fn staged_macro_lock_evidence_is_not_active_authority() {
    let before = contract_with_hard_limit(300);
    let mut staged = before.clone();
    staged.source.rust.macros.allow.push(allowance());
    let old_lock = crate::LockFile::new("0".repeat(64));
    let mut staged_lock = old_lock.clone();
    staged_lock
        .macro_implementations
        .push(LockedMacroImplementation {
            package: "derive-impl".into(),
            directory: "crates/derive-impl".into(),
            inputs_sha256: "1".repeat(64),
        });

    let report = compare_architecture(&before, Some(&old_lock), &staged, Some(&staged_lock));

    assert!(
        report
            .changes
            .iter()
            .all(|change| !change.rail.starts_with("rust.macro-"))
    );
}

fn allowance() -> MacroExpansionAllow {
    MacroExpansionAllow {
        name: "reviewed::expand".into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: MacroExpansionBindings::Opaque,
        async_syntax: MacroAsyncSyntax::Opaque,
        duplication_effect: MacroDuplicationEffect::Opaque,
        source_operations: MacroSourceOperations::Opaque,
        field_mutation: MacroFieldMutation::Opaque,
        definition: None,
        source: None,
        reason: "Reviewed before enforcement is enabled.".into(),
    }
}

fn allowance_from(requirement: &str) -> MacroExpansionAllow {
    let mut allowance = allowance();
    allowance.name = "derive::Model".into();
    allowance.source = Some(CrateRootSource::Registry {
        registry: None,
        index: None,
        requirement: requirement.into(),
    });
    allowance
}
