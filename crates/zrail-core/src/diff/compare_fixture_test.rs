//! Shared strict contract fixture for semantic comparison tests.

use std::collections::BTreeMap;

use crate::{
    Budget, Contract, CycleMode, DependenciesContract, DependencyMode, ExactMode,
    ExternalDependencyMode, FacadeMode, FileSizeContract, HygieneContract, LintSuppressionMode,
    ModuleDocsMode, PolicyMode, RepositoryContract, RustSourceContract, SourceContract,
    SymlinkMode, TestMode,
};

pub(super) fn contract_with_hard_limit(hard: usize) -> Contract {
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
            mode: DependencyMode::Locked,
            unassigned_packages: PolicyMode::Deny,
            cycles: CycleMode::Deny,
            crate_roots: Vec::new(),
        },
        source: SourceContract {
            rust: RustSourceContract {
                module_docs: ModuleDocsMode::Required,
                facades: FacadeMode::Declarative,
                entrypoints: FacadeMode::Declarative,
                tests: TestMode::Sibling,
                file_roles: Vec::new(),
                generated: Vec::new(),
                out_dir: Vec::new(),
                item_macros: Vec::new(),
                macros: crate::MacroExpansionContract::default(),
                hygiene: HygieneContract {
                    unsafe_code: PolicyMode::Deny,
                    lint_suppressions: LintSuppressionMode::Deny,
                    deny_methods: Vec::new(),
                    deny_macros: Vec::new(),
                },
                size: Some(FileSizeContract {
                    facade: Budget {
                        target: 80,
                        hard: 120,
                    },
                    implementation: Budget { target: 240, hard },
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
        layers: vec![crate::LayerContract {
            name: "tool".into(),
            packages: vec!["zrail".into()],
            may_depend_on: Vec::new(),
            profiles: Vec::new(),
            reason: "test".into(),
            dependencies: crate::LayerDependencies {
                external: ExternalDependencyMode::Locked,
            },
        }],
        dependency_rules: Vec::new(),
        scopes: Vec::new(),
        owners: Vec::new(),
        ratchets: Vec::new(),
        gates: Vec::new(),
        invariants: Vec::new(),
    }
}
