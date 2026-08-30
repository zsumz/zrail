//! Proc-macro target spellings follow Cargo's explicit crate-type precedence.

use std::{fs, path::PathBuf};

use toml::Value;

use super::{CargoTargetKind, targets::collect_target_roots};

#[test]
fn proc_macro_boolean_is_preserved_as_a_host_target() {
    assert_kind(
        "proc-macro-boolean",
        r#"
            [package]
            name = "fixture-macros"
            version = "0.0.0"

            [lib]
            proc-macro = true
        "#,
        CargoTargetKind::ProcMacro,
    );
}

#[test]
fn proc_macro_crate_type_is_preserved_as_a_host_target() {
    assert_kind(
        "proc-macro-crate-type",
        r#"
            [package]
            name = "fixture-macros"
            version = "0.0.0"

            [lib]
            crate-type = ["proc-macro"]
        "#,
        CargoTargetKind::ProcMacro,
    );
}

#[test]
fn explicit_normal_crate_type_overrides_proc_macro_boolean() {
    assert_kind(
        "normal-crate-type",
        r#"
            [package]
            name = "fixture-macros"
            version = "0.0.0"

            [lib]
            proc-macro = true
            crate-type = ["lib"]
        "#,
        CargoTargetKind::Library,
    );
}

fn assert_kind(name: &str, source: &str, expected: CargoTargetKind) {
    let root = fixture_root(name);
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source");
    fs::write(root.join("src/lib.rs"), "//! library\n").expect("write library");
    let manifest = source.parse::<Value>().expect("parse manifest");

    let roots = collect_target_roots(&manifest, &root, None).expect("collect roots");

    assert_eq!(roots[0].kind, expected);
    reset(&root);
}

fn fixture_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zrail-proc-macro-targets-{}-{name}",
        std::process::id()
    ))
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
