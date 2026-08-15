//! Shared minimal contract fixtures for cross-section validation tests.

use std::collections::BTreeMap;

use crate::contract::{
    Budget, Contract, CycleMode, DependenciesContract, DependencyMode, ExactMode, FacadeMode,
    FileSizeContract, GeneratedSourceContract, HygieneContract, LintSuppressionMode,
    ModuleDocsMode, PolicyMode, RepositoryContract, RustSourceContract, SourceContract,
    SymlinkMode, TestMode,
};

pub(super) fn generated(root: &str, manifest: &str) -> GeneratedSourceContract {
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

pub(super) fn layer(name: &str) -> crate::LayerContract {
    crate::LayerContract {
        name: name.into(),
        packages: vec![format!("{name}-*")],
        may_depend_on: Vec::new(),
        profiles: Vec::new(),
        reason: "test layer".into(),
        dependencies: crate::LayerDependencies::default(),
    }
}

pub(super) fn minimal_contract() -> Contract {
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
                size: Some(FileSizeContract {
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
                }),
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
