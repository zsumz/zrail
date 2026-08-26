//! Deterministic schema migration preserves layout and rolls back later write failures.

use std::{collections::BTreeMap, fs, path::PathBuf};

use zrail_core::load_contract;

use super::{EditPlan, format, migration};

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
    let expected = legacy_contract()
        .replace("schema = 1", "schema = 2")
        .replace("\"policy/*.toml\"", "\"policy/a.toml\", \"policy/b.toml\"")
        .replace("binding =", "resolution =")
        .replace("bindings =", "namespace_effect =");
    assert_eq!(migrated, expected);
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
    assert_eq!(
        format(&root, std::path::Path::new("zrail.toml"))
            .expect("format plan")
            .changed(),
        0
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn later_write_failure_restores_every_previously_replaced_source() {
    let root = fixture_root().with_extension("rollback");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create rollback fixture");
    fs::write(root.join("a.toml"), "original\n").expect("write original");
    fs::write(root.join("blocked"), "not a directory\n").expect("write blocker");
    let plan = EditPlan {
        root: root.clone(),
        originals: BTreeMap::from([
            ("a.toml".into(), "original\n".into()),
            ("blocked/b.toml".into(), "missing\n".into()),
        ]),
        rendered: BTreeMap::from([
            ("a.toml".into(), "migrated\n".into()),
            ("blocked/b.toml".into(), "migrated\n".into()),
        ]),
    };

    let error = plan.write().expect_err("second replacement must fail");

    assert!(error.to_string().contains("verify blocked/b.toml"));
    assert_eq!(
        fs::read_to_string(root.join("a.toml")).expect("read restored source"),
        "original\n",
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rollback_failure_is_reported_beside_the_original_write_failure() {
    let plan = EditPlan {
        root: PathBuf::from("fixture"),
        originals: BTreeMap::from([
            ("a.toml".into(), "original\n".into()),
            ("b.toml".into(), "original\n".into()),
        ]),
        rendered: BTreeMap::from([
            ("a.toml".into(), "migrated\n".into()),
            ("b.toml".into(), "migrated\n".into()),
        ]),
    };
    let mut calls = Vec::new();

    let error = plan
        .write_with(
            |_, _| Ok(()),
            |path, content| {
                calls.push((path.to_owned(), content.to_owned()));
                match calls.len() {
                    1 => Ok(()),
                    2 => Err("primary failure".into()),
                    3 => Err("restore failure".into()),
                    _ => Err("unexpected replacement call".into()),
                }
            },
        )
        .expect_err("write and rollback must both fail");

    assert_eq!(
        calls,
        vec![
            (PathBuf::from("fixture/a.toml"), "migrated\n".into()),
            (PathBuf::from("fixture/b.toml"), "migrated\n".into()),
            (PathBuf::from("fixture/a.toml"), "original\n".into()),
        ]
    );
    assert!(error.to_string().contains("write b.toml: primary failure"));
    assert!(
        error
            .to_string()
            .contains("rollback also failed: restore a.toml: restore failure")
    );
}

#[test]
fn source_changed_after_planning_is_never_overwritten() {
    let root = fixture_root().with_extension("concurrent");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create concurrent fixture");
    fs::write(root.join("zrail.toml"), "concurrent\n").expect("write concurrent source");
    let plan = EditPlan {
        root: root.clone(),
        originals: BTreeMap::from([("zrail.toml".into(), "original\n".into())]),
        rendered: BTreeMap::from([("zrail.toml".into(), "migrated\n".into())]),
    };

    let error = plan
        .write()
        .expect_err("concurrent change must fail closed");

    assert!(
        error
            .to_string()
            .contains("changed after the edit was planned")
    );
    assert_eq!(
        fs::read_to_string(root.join("zrail.toml")).expect("read concurrent source"),
        "concurrent\n"
    );
    let _ = fs::remove_dir_all(root);
}

fn legacy_contract() -> &'static str {
    r#"# Architecture narrative retained through migration.
schema = 1
adapters = ["rust"]
imports = ["policy/*.toml"]

# >>> generated policy marker

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

# <<< generated policy marker

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
