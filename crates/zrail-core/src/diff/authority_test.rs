//! Exact content and source identities remain visible in semantic review.

use crate::{
    ChangeKind, CrateRootContract, CrateRootSource, LockedMacroDefinition, MacroExpansionAllow,
    MacroExpansionMode, MacroInputMode,
};

use super::{compare_architecture, compare_fixture_test::contract_with_hard_limit};

#[test]
fn opaque_macro_input_is_a_grant_and_local_body_change_is_unknown() {
    let mut before = contract_with_hard_limit(300);
    before.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    before.source.rust.macros.allow.push(MacroExpansionAllow {
        name: "local::query".into(),
        inputs: MacroInputMode::Inspect,
        definition: Some("src/lib.rs".into()),
        source: None,
        reason: "Reviewed local macro.".into(),
    });
    let mut after = before.clone();
    after.source.rust.macros.allow[0].inputs = MacroInputMode::Opaque;
    let mut old_lock = crate::LockFile::new("0".repeat(64));
    let mut new_lock = old_lock.clone();
    old_lock.macros.push(definition("a"));
    new_lock.macros.push(definition("b"));

    let report = compare_architecture(&before, Some(&old_lock), &after, Some(&new_lock));
    assert!(
        report.changes.iter().any(|change| {
            change.kind == ChangeKind::Grant && change.rail == "rust.macro-input"
        })
    );
    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Unknown && change.rail == "rust.macro-definition"
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

fn definition(digit: &str) -> LockedMacroDefinition {
    LockedMacroDefinition {
        path: "src/lib.rs".into(),
        name: "local::query".into(),
        ordinal: 1,
        sha256: digit.repeat(64),
    }
}

fn registry(requirement: &str) -> CrateRootSource {
    CrateRootSource::Registry {
        registry: None,
        index: None,
        requirement: requirement.into(),
    }
}
