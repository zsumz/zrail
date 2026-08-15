//! Inventory traversal is deterministic and excludes build output.

use std::{collections::BTreeMap, fs, path::PathBuf};

use zrail_core::contract::{
    Budget, Contract, CycleMode, DependenciesContract, DependencyMode, ExactMode, FacadeMode,
    FileSizeContract, HygieneContract, LintSuppressionMode, ModuleDocsMode, PolicyMode,
    RepositoryContract, RustSourceContract, SourceContract, SymlinkMode, TestMode,
};

use super::{MAX_DIRECTORY_DEPTH, MAX_RUST_SOURCE_BYTES, inventory_repository};

#[test]
fn inventory_reads_rust_under_declared_roots() {
    let root = fixture_root("inventory");
    if root.exists() {
        fs::remove_dir_all(&root).expect("reset fixture");
    }
    fs::create_dir_all(root.join("crates/a/src")).expect("create fixture");
    fs::write(root.join("crates/a/src/lib.rs"), "//! contract\n").expect("write fixture");
    let inventory = inventory_repository(&root, &contract()).expect("inventory fixture");
    assert_eq!(inventory.rust_files.len(), 1);
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn repository_root_and_manifest_names_are_exact() {
    let root = fixture_root("inventory-root");
    if root.exists() {
        fs::remove_dir_all(&root).expect("reset fixture");
    }
    fs::create_dir_all(root.join("src")).expect("create source");
    fs::create_dir_all(root.join("nested")).expect("create package");
    fs::write(root.join("src/lib.rs"), "//! contract\n").expect("write source");
    fs::write(root.join("NotCargo.toml"), "[package]\nname = 'wrong'\n").expect("write decoy");
    fs::write(
        root.join("nested/Cargo.toml"),
        "[package]\nname = 'nested'\n",
    )
    .expect("write manifest");
    let mut contract = contract();
    contract.repository.roots = vec![".".into()];

    let inventory = inventory_repository(&root, &contract).expect("inventory fixture");

    assert_eq!(inventory.rust_files.len(), 1);
    assert_eq!(
        inventory.manifest_paths,
        [inventory.root.join("nested/Cargo.toml")]
    );
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn inventory_rejects_oversized_rust_source() {
    let root = fixture_root("inventory-oversized");
    if root.exists() {
        fs::remove_dir_all(&root).expect("reset fixture");
    }
    fs::create_dir_all(root.join("crates/a/src")).expect("create fixture");
    fs::write(
        root.join("crates/a/src/lib.rs"),
        vec![b'x'; MAX_RUST_SOURCE_BYTES + 1],
    )
    .expect("write fixture");

    let error = inventory_repository(&root, &contract()).expect_err("oversized source must fail");

    assert!(error.to_string().contains("safety limit"));
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn inventory_rejects_excessive_directory_depth() {
    let root = fixture_root("inventory-depth");
    if root.exists() {
        fs::remove_dir_all(&root).expect("reset fixture");
    }
    let mut directory = root.join("crates");
    for _ in 0..=MAX_DIRECTORY_DEPTH {
        directory.push("nested");
    }
    fs::create_dir_all(&directory).expect("create deep fixture");

    let error = inventory_repository(&root, &contract()).expect_err("deep tree must fail");

    assert!(error.to_string().contains("directory-depth safety limit"));
    fs::remove_dir_all(root).expect("remove fixture");
}

fn fixture_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("zrail-{name}-{}", std::process::id()))
}

fn contract() -> Contract {
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
                module_docs: ModuleDocsMode::Required,
                facades: FacadeMode::Declarative,
                entrypoints: FacadeMode::Declarative,
                tests: TestMode::Sibling,
                generated: Vec::new(),
                out_dir: Vec::new(),
                item_macros: Vec::new(),
                hygiene: HygieneContract {
                    unsafe_code: PolicyMode::Deny,
                    lint_suppressions: LintSuppressionMode::Deny,
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
