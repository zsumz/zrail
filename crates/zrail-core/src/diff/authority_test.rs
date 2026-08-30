//! Exact content and source identities remain visible in semantic review.

use crate::{
    ChangeKind, CrateRootContract, CrateRootSource, LockedMacroImplementation, MacroBindingMode,
    MacroExpansionAllow, MacroExpansionMode, MacroInputMode,
};

use super::{compare_architecture, compare_fixture_test::contract_with_hard_limit};

#[test]
fn opaque_macro_input_is_a_grant_and_local_body_change_is_unknown() {
    let mut before = contract_with_hard_limit(300);
    before.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    before.source.rust.macros.allow.push(MacroExpansionAllow {
        name: "local::query".into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: crate::MacroExpansionBindings::Opaque,
        async_syntax: crate::MacroAsyncSyntax::Opaque,
        duplication_effect: crate::MacroDuplicationEffect::Opaque,
        source_operations: crate::MacroSourceOperations::Opaque,
        field_mutation: crate::MacroFieldMutation::Opaque,
        definition: Some("src/lib.rs".into()),
        source: None,
        reason: "Reviewed local macro.".into(),
    });
    let mut after = before.clone();
    after.source.rust.macros.allow[0].inputs = MacroInputMode::Opaque;
    let mut old_lock = crate::LockFile::new("0".repeat(64));
    let mut new_lock = old_lock.clone();
    old_lock.macro_implementations.push(implementation("a"));
    new_lock.macro_implementations.push(implementation("b"));

    let report = compare_architecture(&before, Some(&old_lock), &after, Some(&new_lock));
    assert!(
        report.changes.iter().any(|change| {
            change.kind == ChangeKind::Grant && change.rail == "rust.macro-input"
        })
    );
    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Unknown && change.rail == "rust.macro-implementation"
    }));
}

#[test]
fn repository_macro_package_authority_adds_as_grant_and_removes_as_revoke() {
    let contract = contract_with_hard_limit(300);
    let empty = crate::LockFile::new("0".repeat(64));
    let mut trusted = empty.clone();
    trusted.macro_implementations.push(implementation("a"));

    let granted = compare_architecture(&contract, Some(&empty), &contract, Some(&trusted));
    let revoked = compare_architecture(&contract, Some(&trusted), &contract, Some(&empty));

    assert!(granted.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "rust.macro-implementation"
    }));
    assert!(revoked.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke && change.rail == "rust.macro-implementation"
    }));
}

#[test]
fn conservative_macro_binding_is_a_grant_and_exact_binding_is_a_revoke() {
    let mut exact = contract_with_hard_limit(300);
    exact.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    exact.source.rust.macros.allow.push(MacroExpansionAllow {
        name: "reviewed".into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: crate::MacroExpansionBindings::Opaque,
        async_syntax: crate::MacroAsyncSyntax::Opaque,
        duplication_effect: crate::MacroDuplicationEffect::Opaque,
        source_operations: crate::MacroSourceOperations::Opaque,
        field_mutation: crate::MacroFieldMutation::Opaque,
        definition: None,
        source: None,
        reason: "Reviewed unresolved spelling.".into(),
    });
    let mut conservative = exact.clone();
    conservative.source.rust.macros.allow[0].binding = MacroBindingMode::Conservative;

    let grant = compare_architecture(&exact, None, &conservative, None);
    let revoke = compare_architecture(&conservative, None, &exact, None);
    assert!(
        grant.changes.iter().any(|change| {
            change.kind == ChangeKind::Grant && change.rail == "rust.macro-binding"
        })
    );
    assert!(revoke.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke && change.rail == "rust.macro-binding"
    }));
}

#[test]
fn no_binding_attestation_is_a_grant_and_removal_is_a_revoke() {
    let mut opaque = contract_with_hard_limit(300);
    opaque.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    opaque.source.rust.macros.allow.push(MacroExpansionAllow {
        name: "serde::Serialize".into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: crate::MacroExpansionBindings::Opaque,
        async_syntax: crate::MacroAsyncSyntax::Opaque,
        duplication_effect: crate::MacroDuplicationEffect::Opaque,
        source_operations: crate::MacroSourceOperations::Opaque,
        field_mutation: crate::MacroFieldMutation::Opaque,
        definition: None,
        source: Some(registry("1")),
        reason: "Reviewed expansion.".into(),
    });
    let mut preserved = opaque.clone();
    preserved.source.rust.macros.allow[0].bindings = crate::MacroExpansionBindings::None;

    let grant = compare_architecture(&opaque, None, &preserved, None);
    let revoke = compare_architecture(&preserved, None, &opaque, None);
    assert!(grant.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "rust.macro-bindings"
    }));
    assert!(revoke.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke && change.rail == "rust.macro-bindings"
    }));
}

#[test]
fn crate_root_and_external_macro_source_changes_fail_closed() {
    let mut before = contract_with_hard_limit(300);
    let registry_one = registry("1");
    before.dependencies.crate_roots.push(CrateRootContract {
        package: "runtime".into(),
        root: "runtime".into(),
        reason: "Reviewed source.".into(),
        source: registry_one.clone(),
    });
    before.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    before.source.rust.macros.allow.push(MacroExpansionAllow {
        name: "runtime::select".into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: crate::MacroExpansionBindings::Opaque,
        async_syntax: crate::MacroAsyncSyntax::Opaque,
        duplication_effect: crate::MacroDuplicationEffect::Opaque,
        source_operations: crate::MacroSourceOperations::Opaque,
        field_mutation: crate::MacroFieldMutation::Opaque,
        definition: None,
        source: Some(registry_one),
        reason: "Reviewed expansion.".into(),
    });
    let mut after = before.clone();
    after.dependencies.crate_roots[0].source = registry("2");
    after.source.rust.macros.allow[0].source = Some(registry("2"));

    let report = compare_architecture(&before, None, &after, None);
    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "dependency.crate-root"
    }));
    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Unknown && change.rail == "rust.macro-source"
    }));
}

fn implementation(digit: &str) -> LockedMacroImplementation {
    LockedMacroImplementation {
        package: "fixture".into(),
        directory: ".".into(),
        inputs_sha256: digit.repeat(64),
    }
}

fn registry(requirement: &str) -> CrateRootSource {
    CrateRootSource::Registry {
        registry: None,
        index: None,
        requirement: requirement.into(),
    }
}
