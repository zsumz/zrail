//! Cross-section validation refuses ambiguous architectural intent.

use std::collections::BTreeMap;

use super::validate_contract;
use crate::contract::{
    Budget, Contract, CycleMode, DependenciesContract, DependencyMode, Effect, EffectBoundary,
    ExactMode, FacadeMode, FileSizeContract, GeneratedSourceContract, HygieneContract,
    LintSuppressionMode, ModuleDocsMode, OutDirSourceContract, OwnerContract, OwnerKind,
    PolicyMode, ProfileContract, RatchetContract, RepositoryContract, RustSourceContract,
    SourceContract, SymlinkMode, TestMode,
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
            effects: EffectBoundary {
                deny: vec![Effect::Network, Effect::Network],
            },
        },
    );
    let error = validate_contract(&contract).expect_err("duplicate effects must fail");
    assert!(error.to_string().contains("duplicate effect Network"));
}

#[test]
fn wildcard_ratchet_targets_are_rejected() {
    let mut contract = minimal_contract();
    contract.ratchets.push(RatchetContract {
        rule: "rust.file-size".into(),
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
fn owner_allow_paths_must_be_inside_the_selector() {
    let mut contract = minimal_contract();
    contract.owners.push(OwnerContract {
        name: "migrations".into(),
        kind: OwnerKind::Directory,
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
            path: "src/lib.rs".into(),
            name: "items:::nested".into(),
            reason: "invalid fixture".into(),
        });

    let error = validate_contract(&contract).expect_err("empty path segments must fail");

    assert!(error.to_string().contains("must be a Rust path"));
}

fn generated(root: &str, manifest: &str) -> GeneratedSourceContract {
    GeneratedSourceContract {
        root: root.into(),
        manifest: manifest.into(),
        inputs: vec!["schema/**".into()],
        target: 1_000,
        hard: 2_000,
        reason: "compiler-owned output".into(),
        auxiliary: Vec::new(),
    }
}

fn layer(name: &str) -> crate::LayerContract {
    crate::LayerContract {
        name: name.into(),
        packages: vec![format!("{name}-*")],
        may_depend_on: Vec::new(),
        profiles: Vec::new(),
        reason: "test layer".into(),
        dependencies: crate::LayerDependencies::default(),
    }
}

fn minimal_contract() -> Contract {
    Contract {
        schema: 1,
        adapters: vec!["rust".into()],
        repository: RepositoryContract {
            roots: vec!["crates".into()],
            exclude: Vec::new(),
            workspace_members: ExactMode::Exact,
            nested_git: PolicyMode::Deny,
            submodules: PolicyMode::Deny,
            symlinks: SymlinkMode::Inside,
        },
        dependencies: DependenciesContract {
            mode: DependencyMode::Observed,
            unassigned_packages: PolicyMode::Allow,
            cycles: CycleMode::Allow,
        },
        source: SourceContract {
            rust: RustSourceContract {
                module_docs: ModuleDocsMode::Allow,
                facades: FacadeMode::Allow,
                entrypoints: FacadeMode::Allow,
                tests: TestMode::Allow,
                generated: Vec::new(),
                out_dir: Vec::new(),
                item_macros: Vec::new(),
                hygiene: HygieneContract {
                    unsafe_code: PolicyMode::Allow,
                    lint_suppressions: LintSuppressionMode::Allow,
                    deny_methods: Vec::new(),
                    deny_macros: Vec::new(),
                },
                size: FileSizeContract {
                    facade: Budget {
                        target: 80,
                        hard: 120,
                    },
                    implementation: Budget {
                        target: 240,
                        hard: 300,
                    },
                    test: Budget {
                        target: 300,
                        hard: 400,
                    },
                    auxiliary: Budget {
                        target: 300,
                        hard: 300,
                    },
                },
            },
        },
        profiles: BTreeMap::new(),
        layers: Vec::new(),
        dependency_rules: Vec::new(),
        scopes: Vec::new(),
        owners: Vec::new(),
        ratchets: Vec::new(),
        gates: Vec::new(),
        invariants: Vec::new(),
    }
}

#[test]
fn unused_profiles_are_rejected_as_stale_policy() {
    let mut contract = minimal_contract();
    contract.profiles.insert(
        "offline".into(),
        ProfileContract {
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
