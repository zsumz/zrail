//! Item-producing macro authority is exact, scoped, stale, and explainable.

use std::{fs, path::PathBuf};

use zrail_core::Report;
use zrail_rust::{check_repository, explain_path};

#[test]
fn item_macro_authority_selects_scope_and_requires_exact_origin_when_requested() {
    let root = repository();
    let report = check(&root);

    assert!(has(
        &report,
        "RUST-GRAPH-003",
        "src/outside.rs",
        "criterion_group"
    ));
    assert!(has(
        &report,
        "RUST-GRAPH-003",
        "src/ambiguous.rs",
        "mystery"
    ));
    assert!(!has(&report, "RUST-GRAPH-003", "src/exact.rs", "legacy"));
    assert!(!has(
        &report,
        "RUST-GRAPH-003",
        "src/builtin.rs",
        "thread_local"
    ));
    assert!(!has(
        &report,
        "RUST-GRAPH-003",
        "benches/throughput.rs",
        "criterion_group"
    ));
    assert!(message(&report, "RUST-GRAPH-005", "unused"));
    assert!(message(&report, "RUST-GRAPH-005", "mystery"));
    assert!(!message(&report, "RUST-GRAPH-005", "legacy"));
    assert!(!message(&report, "RUST-GRAPH-005", "criterion_group"));

    let benchmark = explain_path(
        &root,
        "zrail.toml".as_ref(),
        "benches/throughput.rs".as_ref(),
    )
    .expect("explain scoped benchmark authority");
    assert_eq!(benchmark.item_macro_authorities.len(), 1);
    assert_eq!(benchmark.item_macro_authorities[0].name, "criterion_group");
    assert_eq!(
        benchmark.item_macro_authorities[0].selector,
        "within [benches/**]"
    );
    assert!(
        benchmark
            .human()
            .contains("criterion_group within [benches/**]")
    );

    let repository = explain_path(&root, "zrail.toml".as_ref(), "src/repository.rs".as_ref())
        .expect("explain repository-wide authority");
    assert_eq!(
        repository.item_macro_authorities[0].selector,
        "repository-wide"
    );

    write(
        &root,
        "src/builtin.rs",
        "//! Local shadow.\nmacro_rules! thread_local { () => {} }\nthread_local!();\n",
    );
    let shadowed = check(&root);
    assert!(has(
        &shadowed,
        "RUST-GRAPH-003",
        "src/builtin.rs",
        "thread_local"
    ));

    write(
        &root,
        "benches/throughput.rs",
        "//! Empty benchmark.\nfn main() {}\n",
    );
    let stale = check(&root);
    assert!(message(&stale, "RUST-GRAPH-005", "criterion_group"));
    reset(&root);
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-item-macro-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    for directory in ["src", "benches"] {
        fs::create_dir_all(root.join(directory)).expect("create fixture directory");
    }
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", CONTRACT);
    write(&root, "src/lib.rs", LIBRARY);
    write(&root, "src/exact.rs", "//! Exact.\nlegacy!();\n");
    write(
        &root,
        "src/builtin.rs",
        "//! Compiler-owned macro.\nthread_local! { static VALUE: u8 = const { 0 }; }\n",
    );
    write(
        &root,
        "src/outside.rs",
        "//! Outside scope.\nuse criterion::criterion_group;\ncriterion_group!();\n",
    );
    write(
        &root,
        "src/ambiguous.rs",
        "//! Unresolved exact origin.\nmystery!();\n",
    );
    write(
        &root,
        "src/repository.rs",
        "//! Repository scope.\nrepository_items!();\n",
    );
    write(
        &root,
        "benches/throughput.rs",
        "//! Benchmark.\nuse criterion::criterion_group;\ncriterion_group!();\nfn main() {}\n",
    );
    root
}

fn check(root: &std::path::Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check item-macro fixture")
        .report
}

fn has(report: &Report, id: &str, path: &str, text: &str) -> bool {
    report.findings.iter().any(|finding| {
        finding.id == id && finding.path.as_deref() == Some(path) && finding.message.contains(text)
    })
}

fn message(report: &Report, id: &str, text: &str) -> bool {
    report
        .findings
        .iter()
        .any(|finding| finding.id == id && finding.message.contains(text))
}

fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = r#"[package]
name = "fixture"
version = "0.0.0"
edition = "2024"

[dependencies]
criterion = "1"

[[bench]]
name = "throughput"
path = "benches/throughput.rs"
harness = false
"#;

const LIBRARY: &str = concat!(
    "//! Library.\n",
    "mod ambiguous;\n",
    "mod builtin;\n",
    "mod exact;\n",
    "mod outside;\n",
    "mod repository;\n",
);

const CONTRACT: &str = r#"schema = 1
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
[[dependencies.crate_root]]
package = "criterion"
root = "criterion"
reason = "Reviewed external crate-root identity."
[dependencies.crate_root.source]
kind = "registry"
requirement = "1"
[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"

[[source.rust.item_macros]]
name = "criterion_group"
within = ["benches/**"]
binding = "exact"
reason = "Reviewed benchmark harness."
[source.rust.item_macros.source]
kind = "registry"
requirement = "1"

[[source.rust.item_macros]]
name = "legacy"
path = "src/exact.rs"
reason = "Existing exact-path contract."

[[source.rust.item_macros]]
name = "repository_items"
reason = "Reviewed repository-wide item generator."

[[source.rust.item_macros]]
name = "mystery"
path = "src/ambiguous.rs"
binding = "exact"
reason = "Exact authority must fail on unresolved origin."

[[source.rust.item_macros]]
name = "unused"
within = ["src/**"]
reason = "Stale scoped authority must fail."
"#;
