//! Architecture changes are classified by effective permission.

use crate::{
    ChangeKind, CrateRootContract, CrateRootSource, DependencyMode, FacadeMode,
    GeneratedSourceContract, ItemMacroContract, LintSuppressionMode, LockFile, LockedDependency,
    LockedDependencyKind, LockedDependencyScope, LockedDependencySource, LockedGeneratedSource,
    LockedPackage, LockedRatchet, MacroBindingMode, MacroExpansionAllow, MacroExpansionMode,
    MacroInputMode, OutDirSourceContract, OwnerContract, OwnerKind,
};

use super::compare_architecture;
use crate::diff::compare_fixture_test::contract_with_hard_limit;

#[test]
fn raising_a_hard_ceiling_is_a_grant() {
    let before = contract_with_hard_limit(300);
    let after = contract_with_hard_limit(500);
    let report = compare_architecture(&before, None, &after, None);

    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.subject == "implementation.hard"
    }));
    assert!(report.denies_grants());
}

#[test]
fn relaxing_exact_dependencies_is_a_grant() {
    let before = contract_with_hard_limit(300);
    let mut after = before.clone();
    after.dependencies.mode = DependencyMode::Observed;

    let report = compare_architecture(&before, None, &after, None);

    assert!(
        report
            .changes
            .iter()
            .any(|change| { change.kind == ChangeKind::Grant && change.rail == "dependency.lock" })
    );
}

#[test]
fn external_crate_root_attestation_is_a_grant_and_change_is_unknown() {
    let before = contract_with_hard_limit(300);
    let mut trusted = before.clone();
    trusted.dependencies.crate_roots.push(CrateRootContract {
        package: "tokio".into(),
        root: "runtime".into(),
        reason: "Reviewed dependency metadata.".into(),
        source: CrateRootSource::Registry {
            registry: None,
            index: None,
            requirement: "1".into(),
        },
    });
    let added = compare_architecture(&before, None, &trusted, None);
    assert!(added.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "dependency.crate-root"
    }));

    let mut changed = trusted.clone();
    changed.dependencies.crate_roots[0].root = "executor".into();
    let changed = compare_architecture(&trusted, None, &changed, None);
    assert!(changed.changes.iter().any(|change| {
        change.kind == ChangeKind::Unknown && change.rail == "dependency.crate-root"
    }));
}

#[test]
fn macro_expansion_denial_revokes_power_and_allowance_grants_it() {
    let before = contract_with_hard_limit(300);
    let mut denied = before.clone();
    denied.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    let tightened = compare_architecture(&before, None, &denied, None);
    assert!(tightened.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke && change.rail == "rust.macro-expansion"
    }));

    let mut allowed = denied.clone();
    allowed.source.rust.macros.allow.push(MacroExpansionAllow {
        name: "tokio::select".into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        definition: None,
        source: None,
        reason: "Reviewed async control-flow expansion.".into(),
    });
    let widened = compare_architecture(&denied, None, &allowed, None);
    assert!(widened.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "rust.macro-expansion.allow"
    }));
}

#[test]
fn declaring_generated_source_is_a_grant() {
    let before = contract_with_hard_limit(300);
    let mut after = before.clone();
    after.source.rust.generated.push(GeneratedSourceContract {
        root: "src/generated".into(),
        manifest: "src/generated/MANIFEST.json".into(),
        inputs: vec!["schema/**".into()],
        target: 1_000,
        hard: 2_000,
        reason: "compiler-owned output".into(),
        auxiliary: Vec::new(),
    });

    let report = compare_architecture(&before, None, &after, None);

    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant
            && change.rail == "rust.generated-source"
            && change.subject == "src/generated"
    }));
}

#[test]
fn generated_selector_removal_auxiliary_and_item_macro_trust_are_grants() {
    let mut before = contract_with_hard_limit(300);
    before.source.rust.generated.push(GeneratedSourceContract {
        root: "src/generated".into(),
        manifest: "src/generated/MANIFEST.json".into(),
        inputs: vec!["schema/**".into()],
        target: 1_000,
        hard: 2_000,
        reason: "compiler-owned output".into(),
        auxiliary: Vec::new(),
    });
    let mut after = before.clone();
    after.source.rust.generated[0]
        .auxiliary
        .push("exports.rsi".into());
    after.source.rust.generated[0].inputs.clear();
    after.source.rust.item_macros.push(ItemMacroContract {
        path: "src/lib.rs".into(),
        name: "items".into(),
        reason: "local macro emits no source edges".into(),
    });

    let report = compare_architecture(&before, None, &after, None);

    assert_eq!(
        report
            .changes
            .iter()
            .filter(|change| change.kind == ChangeKind::Grant)
            .count(),
        3
    );
}

#[test]
fn trusting_out_dir_source_is_a_grant() {
    let before = contract_with_hard_limit(300);
    let mut after = before.clone();
    after.source.rust.out_dir.push(OutDirSourceContract {
        path: "src/lib.rs".into(),
        output: "wire.rs".into(),
        source: "src/generated/wire.rs".into(),
        reason: "verified snapshot".into(),
    });

    let report = compare_architecture(&before, None, &after, None);

    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "rust.source-graph.out-dir"
    }));
}

#[test]
fn widening_a_capability_owner_is_a_grant() {
    let mut before = contract_with_hard_limit(300);
    before.owners.push(OwnerContract {
        name: "filesystem".into(),
        kind: OwnerKind::Capability,
        within: vec!["src/**".into()],
        selector: "std::fs".into(),
        allow: vec!["src/io.rs".into()],
        reason: "one filesystem owner".into(),
    });
    let mut after = before.clone();
    after.owners[0].allow.push("src/other.rs".into());

    let report = compare_architecture(&before, None, &after, None);

    assert!(
        report
            .changes
            .iter()
            .any(|change| { change.kind == ChangeKind::Grant && change.rail == "owner.allow" })
    );
}

#[test]
fn relaxing_entrypoints_and_lint_reasons_is_a_grant() {
    let before = contract_with_hard_limit(300);
    let mut after = before.clone();
    after.source.rust.entrypoints = FacadeMode::Allow;
    after.source.rust.hygiene.lint_suppressions = LintSuppressionMode::Reasoned;

    let report = compare_architecture(&before, None, &after, None);

    assert!(
        report.changes.iter().any(|change| {
            change.kind == ChangeKind::Grant && change.rail == "rust.entrypoints"
        })
    );
    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "rust.lint-suppressions"
    }));
}

#[test]
fn resolved_packages_and_ratchets_have_opposite_directions() {
    let contract = contract_with_hard_limit(300);
    let mut before_lock = LockFile::new("before");
    before_lock.packages.push(LockedPackage {
        name: "core".into(),
        dependencies: Vec::new(),
    });
    before_lock.ratchets.push(LockedRatchet {
        rule: "rust.file-size".into(),
        target: "crates/core/src/model.rs".into(),
        value: 260,
    });
    let mut after_lock = LockFile::new("after");
    after_lock.packages = before_lock.packages.clone();
    after_lock.packages.push(LockedPackage {
        name: "adapter".into(),
        dependencies: vec![LockedDependency {
            alias: Some("core".into()),
            name: "core".into(),
            crate_root: Some("core".into()),
            kind: LockedDependencyKind::Normal,
            scope: LockedDependencyScope::Internal,
            target: None,
            optional: Some(false),
            default_features: Some(true),
            features: Vec::new(),
            source: Some(LockedDependencySource::WorkspaceMember {
                directory: "crates/core".into(),
                requirement: None,
            }),
        }],
    });

    let report = compare_architecture(&contract, Some(&before_lock), &contract, Some(&after_lock));

    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant
            && change.rail == "repository.package"
            && change.subject == "adapter"
    }));
    assert!(
        report
            .changes
            .iter()
            .any(|change| { change.kind == ChangeKind::Cleanup && change.rail == "ratchet" })
    );
}

#[test]
fn generated_provenance_lock_changes_are_semantic() {
    let contract = contract_with_hard_limit(300);
    let mut before = LockFile::new("before");
    before.generated.push(locked_generated("1"));
    let mut changed = LockFile::new("after");
    changed.generated.push(locked_generated("2"));
    let removed = LockFile::new("after");

    let changed_report = compare_architecture(&contract, Some(&before), &contract, Some(&changed));
    let removed_report = compare_architecture(&contract, Some(&before), &contract, Some(&removed));
    let added_report = compare_architecture(&contract, Some(&removed), &contract, Some(&changed));

    assert!(changed_report.changes.iter().any(|change| {
        change.kind == ChangeKind::Unknown && change.rail == "rust.generated-provenance"
    }));
    assert!(removed_report.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "rust.generated-provenance"
    }));
    assert!(added_report.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke && change.rail == "rust.generated-provenance"
    }));
}

fn locked_generated(digit: &str) -> LockedGeneratedSource {
    LockedGeneratedSource {
        root: "src/generated".into(),
        manifest_sha256: digit.repeat(64),
    }
}
