//! Async syntax, glob imports, and opaque macro authority have directed semantic diffs.

use crate::{
    AsyncSyntax, ChangeKind, EffectBoundary, GlobImportMode, MacroAsyncSyntax, MacroBindingMode,
    MacroExpansionAllow, MacroExpansionBindings, MacroExpansionMode, MacroInputMode,
    PolicyReachability, ProfileContract, SyntaxBoundary, compare_architecture,
};

use super::compare_fixture_test::contract_with_hard_limit;

#[test]
fn stricter_glob_import_modes_are_revocations() {
    let allow = contract_with_hard_limit(300);
    let mut facade_only = allow.clone();
    facade_only.source.rust.hygiene.glob_imports = GlobImportMode::FacadeReexportsOnly;
    let mut deny = facade_only.clone();
    deny.source.rust.hygiene.glob_imports = GlobImportMode::Deny;

    for (before, after) in [(&allow, &facade_only), (&facade_only, &deny)] {
        let report = compare_architecture(before, None, after, None);
        assert!(report.changes.iter().any(|change| {
            change.kind == ChangeKind::Revoke && change.rail == "rust.glob-imports"
        }));
    }
    let report = compare_architecture(&deny, None, &allow, None);
    assert!(
        report.changes.iter().any(|change| {
            change.kind == ChangeKind::Grant && change.rail == "rust.glob-imports"
        })
    );
}

#[test]
fn adding_async_syntax_denials_is_a_revocation() {
    let mut open = contract_with_hard_limit(300);
    open.profiles.insert("sync".into(), profile(Vec::new()));
    let mut denied = open.clone();
    denied
        .profiles
        .get_mut("sync")
        .expect("profile")
        .syntax
        .deny = vec![AsyncSyntax::Await];

    let report = compare_architecture(&open, None, &denied, None);
    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke
            && change.rail == "syntax.boundary"
            && change.subject.contains("Await")
    }));
}

#[test]
fn trusting_macro_output_to_be_async_free_is_a_grant() {
    let mut opaque = contract_with_hard_limit(300);
    opaque.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    opaque.source.rust.macros.allow.push(MacroExpansionAllow {
        name: "local::make".into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: MacroExpansionBindings::Opaque,
        async_syntax: MacroAsyncSyntax::Opaque,
        duplication_effect: crate::MacroDuplicationEffect::Opaque,
        definition: Some("src/lib.rs".into()),
        source: None,
        reason: "Reviewed local macro output.".into(),
    });
    let mut trusted = opaque.clone();
    trusted.source.rust.macros.allow[0].async_syntax = MacroAsyncSyntax::None;

    let grant = compare_architecture(&opaque, None, &trusted, None);
    let revoke = compare_architecture(&trusted, None, &opaque, None);
    assert!(grant.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "rust.macro-async-syntax"
    }));
    assert!(revoke.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke && change.rail == "rust.macro-async-syntax"
    }));
}

#[test]
fn trusting_macro_output_to_be_duplication_free_is_a_grant() {
    let mut opaque = contract_with_hard_limit(300);
    opaque.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    opaque.source.rust.macros.allow.push(MacroExpansionAllow {
        name: "local::make".into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: MacroExpansionBindings::Opaque,
        async_syntax: MacroAsyncSyntax::Opaque,
        duplication_effect: crate::MacroDuplicationEffect::Opaque,
        definition: Some("src/lib.rs".into()),
        source: None,
        reason: "Reviewed local macro output.".into(),
    });
    let mut trusted = opaque.clone();
    trusted.source.rust.macros.allow[0].duplication_effect = crate::MacroDuplicationEffect::None;

    let grant = compare_architecture(&opaque, None, &trusted, None);
    let revoke = compare_architecture(&trusted, None, &opaque, None);
    assert!(grant.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "rust.macro-duplication-effect"
    }));
    assert!(revoke.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke && change.rail == "rust.macro-duplication-effect"
    }));
}

fn profile(deny: Vec<AsyncSyntax>) -> ProfileContract {
    ProfileContract {
        reachability: PolicyReachability::All,
        effects: EffectBoundary { deny: Vec::new() },
        syntax: SyntaxBoundary { deny },
    }
}
