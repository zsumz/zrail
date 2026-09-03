//! Compiler derives retain their macro namespace when same-named traits are imported.

use std::{fmt::Write as _, fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn imported_standard_trait_does_not_replace_builtin_derive() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-builtin-derive-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! Fixture.\nuse std::fmt::Debug;\n#[derive(Debug)]\npub struct Model;\n",
    );
    write(&root, "zrail.toml", CONTRACT);
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build lock")
        .write(&root.join("zrail.lock"))
        .expect("write lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check builtin derive")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn repository_function_alias_does_not_replace_builtin_invocation() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-builtin-invocation-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! Fixture.\npub fn format() {}\nmod child { use super::format; pub fn run() { let _ = format!(\"ok\"); } }\n",
    );
    write(
        &root,
        "zrail.toml",
        &CONTRACT.replace("name = \"Debug\"", "name = \"format\""),
    );
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build lock")
        .write(&root.join("zrail.lock"))
        .expect("write lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check builtin invocation")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn dependency_alias_cannot_borrow_builtin_derive_authority() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-shadowed-derive-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nreviewed = { package = \"serde\", version = \"1\" }\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! Fixture.\nuse reviewed::Serialize as Debug;\n#[derive(Debug)]\npub struct Model;\n",
    );
    write(&root, "zrail.toml", CONTRACT);

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check shadowed derive")
        .report;

    assert!(report.findings.iter().any(|finding| {
        finding.id.starts_with("RUST-MACRO") && finding.message.contains("Debug")
    }));
    reset(&root);
}

#[test]
fn dependency_glob_candidates_remain_subject_to_macro_policy() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-globbed-derive-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nproptest = \"1\"\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! Fixture.\nuse proptest::prelude::*;\n#[derive(Clone, Debug)]\npub struct Model;\n",
    );
    write(&root, "zrail.toml", CONTRACT);

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check globbed derive")
        .report;

    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-MACRO-006" && finding.message.contains("proptest::prelude")
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

#[test]
fn inherited_standard_trait_does_not_replace_builtin_derive() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-inherited-builtin-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! Fixture.\nuse std::fmt::Debug;\nmod child {\n    use super::*;\n    #[derive(Debug)]\n    pub struct Model;\n}\n",
    );
    write(&root, "zrail.toml", CONTRACT);
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build lock")
        .write(&root.join("zrail.lock"))
        .expect("write lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check inherited builtin derive")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn inherited_dependency_alias_cannot_borrow_builtin_derive_authority() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-inherited-shadow-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nreviewed = { package = \"serde\", version = \"1\" }\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! Fixture.\nuse reviewed::Serialize as Debug;\nmod child {\n    use super::*;\n    #[derive(Debug)]\n    pub struct Model;\n}\n",
    );
    write(&root, "zrail.toml", CONTRACT);

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check inherited shadowed derive")
        .report;

    assert!(report.findings.iter().any(|finding| {
        finding.id.starts_with("RUST-MACRO") && finding.message.contains("Debug")
    }));
    reset(&root);
}

#[test]
fn builtin_derive_survives_an_overflowed_local_macro_catalog() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-overflowed-builtin-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    );
    let mut source = "//! Fixture.\nuse std::fmt::Debug;\n".to_owned();
    for index in 0..=256 {
        writeln!(source, "macro_rules! local_{index} {{ () => {{}} }}").expect("write macro");
    }
    source.push_str("#[derive(Debug)]\npub struct Model;\n");
    write(&root, "src/lib.rs", &source);
    write(&root, "zrail.toml", CONTRACT);
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build lock")
        .write(&root.join("zrail.lock"))
        .expect("write lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check builtin derive with overflowed macro catalog")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

fn write(root: &Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture file");
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

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
[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "deny-unreviewed"
[[source.rust.macros.allow]]
name = "Debug"
reason = "Reviewed compiler Debug derive."
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#;
