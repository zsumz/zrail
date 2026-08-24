//! Ordinary repository size is input, not an analysis failure condition.

use std::{fmt::Write as _, fs, path::PathBuf};

use zrail_rust::build_lock;

const MODULES: usize = 10_000;

#[test]
fn ten_thousand_physical_rust_files_produce_stable_complete_lock_state() {
    let root = fixture();

    let first = build_lock(&root, "zrail.toml".as_ref()).expect("build large repository lock");
    let repeated = build_lock(&root, "zrail.toml".as_ref()).expect("repeat large repository lock");
    let analysis = first.analysis.as_ref().expect("completeness certificate");

    assert_eq!(first, repeated);
    assert_eq!(analysis.physical_rust_files, MODULES + 1);
    assert!(analysis.base_source_contexts > MODULES);
    assert_eq!(analysis.derived_source_contexts, 0);
    assert_eq!(analysis.projection_queries, 0);
    assert_eq!(analysis.projected_facts, 0);
    assert_eq!(analysis.unresolved_bindings, 0);
    reset(&root);
}

fn fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-scale-adoption-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source directory");
    fs::write(root.join("Cargo.toml"), CARGO).expect("write Cargo manifest");
    fs::write(root.join("zrail.toml"), CONTRACT).expect("write contract");

    let mut library = String::from("//! Generated repository-scale module graph.\n");
    for index in 0..MODULES {
        writeln!(library, "mod unit_{index:05};").expect("render module declaration");
        fs::write(
            root.join(format!("src/unit_{index:05}.rs")),
            "//! One ordinary generated scale unit.\npub fn marker() {}\n",
        )
        .expect("write scale source");
    }
    fs::write(root.join("src/lib.rs"), library).expect("write library root");
    root
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const CARGO: &str = r#"[package]
name = "scale-adoption"
version = "0.0.0"
edition = "2024"
"#;

const CONTRACT: &str = r#"schema = 2
adapters = ["rust"]

[repository]
roots = ["."]
exclude = []
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"

[dependencies]
mode = "observed"
unassigned_packages = "allow"
cycles = "deny"

[source.rust]
module_docs = "required"
facades = "allow"
entrypoints = "allow"
tests = "allow"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "deny"
"#;
