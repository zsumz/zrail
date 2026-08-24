//! Deterministic schema migration rewrites every exact contract source atomically.

use std::{fs, path::PathBuf};

use zrail_core::load_contract;

use super::migration;

#[test]
fn schema_migration_expands_imports_and_renames_macro_authority() {
    let root = fixture_root();
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("policy")).expect("create policy directory");
    fs::write(root.join("policy/a.toml"), "").expect("write fragment a");
    fs::write(root.join("policy/b.toml"), "").expect("write fragment b");
    fs::write(root.join("zrail.toml"), legacy_contract()).expect("write legacy contract");

    let plan = migration(&root, std::path::Path::new("zrail.toml")).expect("plan migration");
    assert!(plan.changed() >= 1);
    plan.write().expect("write migration");

    let migrated = fs::read_to_string(root.join("zrail.toml")).expect("read migration");
    assert!(migrated.contains("schema = 2"));
    assert!(migrated.contains("\"policy/a.toml\""));
    assert!(migrated.contains("\"policy/b.toml\""));
    assert!(migrated.contains("resolution = \"conservative\""));
    assert!(migrated.contains("namespace_effect = \"opaque\""));
    assert!(!migrated.contains("binding ="));
    assert!(!migrated.contains("bindings ="));
    let bundle =
        load_contract(&root, std::path::Path::new("zrail.toml")).expect("load schema-2 migration");
    assert_eq!(bundle.contract.schema, 2);
    assert_eq!(
        migration(&root, std::path::Path::new("zrail.toml"))
            .expect("replan")
            .changed(),
        0
    );
    let _ = fs::remove_dir_all(root);
}

fn legacy_contract() -> &'static str {
    r#"schema = 1
adapters = ["rust"]
imports = ["policy/*.toml"]

[repository]
roots = ["."]
exclude = []
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"

[dependencies]
mode = "locked"
unassigned_packages = "allow"
cycles = "deny"

[source.rust]
module_docs = "allow"
facades = "allow"
entrypoints = "allow"
tests = "allow"

[source.rust.macros]
mode = "deny-unreviewed"

[[source.rust.macros.allow]]
name = "reviewed"
inputs = "inspect"
binding = "conservative"
bindings = "opaque"
reason = "reviewed macro boundary"

[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#
}

fn fixture_root() -> PathBuf {
    std::env::temp_dir().join(format!("zrail-config-edit-test-{}", std::process::id()))
}
