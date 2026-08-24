//! Cross-section validation refuses ambiguous architectural intent.

use super::validate_contract;
use crate::contract::{
    DependencyEdgeKind, DependencyReachability, DependencyRule, Effect, EffectBoundary,
    OutDirSourceContract, OwnerContract, OwnerKind, PolicyReachability, ProfileContract,
    RatchetContract,
    validate_fixture_test::{generated, layer, minimal_contract},
};

#[test]
fn duplicate_layer_names_are_rejected() {
    let mut contract = minimal_contract();
    contract.layers = vec![layer("core"), layer("core")];
    let error = validate_contract(&contract).expect_err("duplicate layers must fail");
    assert!(error.to_string().contains("duplicate layer"));
}

#[test]
fn duplicate_repository_roots_are_rejected() {
    let mut contract = minimal_contract();
    contract.repository.roots.push("crates".into());
    let error = validate_contract(&contract).expect_err("duplicate roots must fail");
    assert!(
        error
            .to_string()
            .contains("repository.roots contains duplicate")
    );
}

#[test]
fn repository_root_literal_may_cover_the_whole_repository() {
    let mut contract = minimal_contract();
    contract.repository.roots = vec![".".into()];

    validate_contract(&contract).expect("repository root should be valid");
}

#[test]
fn duplicate_effects_are_rejected() {
    let mut contract = minimal_contract();
    contract.profiles.insert(
        "offline".into(),
        ProfileContract {
            reachability: PolicyReachability::default(),
            effects: EffectBoundary {
                deny: vec![Effect::Network, Effect::Network],
            },
        },
    );
    let error = validate_contract(&contract).expect_err("duplicate effects must fail");
    assert!(error.to_string().contains("duplicate effect Network"));
}

#[test]
fn duplicate_dependency_kinds_are_rejected() {
    let mut contract = minimal_contract();
    contract.dependency_rules.push(DependencyRule {
        name: "runtime-boundary".into(),
        from: "fixture".into(),
        deny: vec!["blocked".into()],
        reachability: DependencyReachability::Transitive,
        kinds: vec![DependencyEdgeKind::Build, DependencyEdgeKind::Build],
        reason: "one reviewed first-edge kind".into(),
    });

    let error = validate_contract(&contract).expect_err("duplicate kinds must fail");

    assert!(error.to_string().contains("duplicate dependency kind"));
}

#[test]
fn wildcard_ratchet_targets_are_rejected() {
    let mut contract = minimal_contract();
    contract.ratchets.push(RatchetContract {
        rule: "rust.file-size".into(),
        selector: None,
        target: "crates/**/*.rs".into(),
        reason: "test ratchet".into(),
    });
    let error = validate_contract(&contract).expect_err("wildcard ratchets must fail");
    assert!(
        error
            .to_string()
            .contains("expected an exact repository path")
    );
}

#[test]
fn adoption_ratchets_are_rejected_without_strict_policy() {
    for rule in [
        "rust.module-docs",
        "rust.hygiene.unsafe",
        "rust.hygiene.lint-suppressions",
    ] {
        let mut contract = minimal_contract();
        contract.ratchets.push(RatchetContract {
            rule: rule.into(),
            selector: None,
            target: "crates/legacy.rs".into(),
            reason: "legacy debt".into(),
        });

        let error = validate_contract(&contract).expect_err("disabled policy must reject ratchet");
        assert!(
            error.to_string().contains("strict Rust source policy"),
            "{rule}: {error}"
        );
    }
}

#[test]
fn owner_allow_paths_must_be_inside_the_selector() {
    let mut contract = minimal_contract();
    contract.owners.push(OwnerContract {
        name: "migrations".into(),
        kind: OwnerKind::Directory,
        reachability: PolicyReachability::All,
        within: Vec::new(),
        selector: "**/migrations".into(),
        allow: vec!["crates/store/schema".into()],
        reason: "one migration owner".into(),
    });
    let error = validate_contract(&contract).expect_err("owner escapes must fail");
    assert!(error.to_string().contains("outside its selector"));
}

#[test]
fn generated_manifests_stay_inside_disjoint_roots() {
    let mut contract = minimal_contract();
    contract.source.rust.generated = vec![
        generated("src/generated", "MANIFEST.json"),
        generated("src/generated/nested", "src/generated/nested/MANIFEST.json"),
    ];

    let error = validate_contract(&contract).expect_err("ambiguous generated roots must fail");

    assert!(error.to_string().contains("must be inside root"));
    assert!(error.to_string().contains("overlap"));
}

#[test]
fn generated_source_stays_inside_analyzed_roots() {
    let mut contract = minimal_contract();
    contract.source.rust.generated = vec![generated(
        "outside/generated",
        "outside/generated/MANIFEST.json",
    )];

    let error = validate_contract(&contract).expect_err("unobserved generated source must fail");

    assert!(error.to_string().contains("inside repository.roots"));
}

#[test]
fn out_dir_sources_require_a_verified_generated_root() {
    let mut contract = minimal_contract();
    contract.source.rust.out_dir.push(OutDirSourceContract {
        path: "crates/fixture/src/lib.rs".into(),
        output: "wire.rs".into(),
        source: "crates/fixture/src/generated/wire.rs".into(),
        reason: "reviewed compiler output".into(),
    });

    let error = validate_contract(&contract).expect_err("unverified output must fail");

    assert!(error.to_string().contains("verified generated root"));
}

#[test]
fn item_macro_names_are_complete_rust_paths() {
    let mut contract = minimal_contract();
    contract
        .source
        .rust
        .item_macros
        .push(crate::ItemMacroContract {
            name: "items:::nested".into(),
            path: Some("src/lib.rs".into()),
            within: Vec::new(),
            binding: None,
            source: None,
            manifest: None,
            reason: "invalid fixture".into(),
        });

    let error = validate_contract(&contract).expect_err("empty path segments must fail");

    assert!(error.to_string().contains("must be a Rust path"));
}

#[test]
fn unused_profiles_are_rejected_as_stale_policy() {
    let mut contract = minimal_contract();
    contract.profiles.insert(
        "offline".into(),
        ProfileContract {
            reachability: PolicyReachability::default(),
            effects: EffectBoundary {
                deny: vec![Effect::Network],
            },
        },
    );

    let error = validate_contract(&contract).expect_err("unused profile must fail");

    assert!(error.to_string().contains("assigned to no layer"));
}

#[test]
fn empty_profiles_and_scopes_are_rejected() {
    let mut contract = minimal_contract();
    contract.profiles.insert(
        "empty".into(),
        ProfileContract {
            reachability: PolicyReachability::default(),
            effects: EffectBoundary { deny: Vec::new() },
        },
    );
    contract.scopes.push(crate::ScopeContract {
        name: "empty-scope".into(),
        include: vec!["crates/**".into()],
        exclude: Vec::new(),
        reason: "test scope".into(),
        symbols: crate::SymbolBoundary { deny: Vec::new() },
    });
    contract.layers.push(crate::LayerContract {
        name: "uses-empty".into(),
        packages: vec!["fixture".into()],
        may_depend_on: Vec::new(),
        profiles: vec!["empty".into()],
        reason: "test layer".into(),
        dependencies: crate::LayerDependencies::default(),
    });

    let error = validate_contract(&contract).expect_err("empty policy must fail");

    assert!(error.to_string().contains("denies no effects"));
    assert!(error.to_string().contains("denies no symbols"));
}

#[test]
fn package_selectors_are_bounded_to_cargo_name_globs() {
    let mut contract = minimal_contract();
    contract.layers.push(crate::LayerContract {
        name: "bad-selector".into(),
        packages: vec!["crates/**".into()],
        may_depend_on: Vec::new(),
        profiles: Vec::new(),
        reason: "test layer".into(),
        dependencies: crate::LayerDependencies::default(),
    });

    let error = validate_contract(&contract).expect_err("path-like selector must fail");

    assert!(error.to_string().contains("invalid package selector"));
}

#[test]
fn explicit_analysis_limits_must_be_positive() {
    let mut contract = minimal_contract();
    contract.analysis.limits.include_projection_work = Some(0);

    let error = validate_contract(&contract).expect_err("zero work limit must fail");

    assert!(
        error
            .to_string()
            .contains("analysis.limits.include_projection_work must be positive")
    );
}
