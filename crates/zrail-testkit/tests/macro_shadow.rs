//! Local namespaces cannot borrow a dependency macro's reviewed identity.

use std::{fs, path::PathBuf};

use zrail_rust::check_repository;

#[test]
fn local_module_shadow_cannot_borrow_dependency_macro_authority() {
    let root = root();
    fs::create_dir_all(root.join("src")).expect("create source root");
    fs::write(
        root.join("Cargo.toml"),
        concat!(
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
            "[dependencies]\nruntime = { package = \"tokio\", version = \"1\" }\n",
        ),
    )
    .expect("write manifest");
    fs::write(
        root.join("src/lib.rs"),
        r"//! Shadowed dependency root.
mod runtime {
    macro_rules! select {
        () => {{ unsafe { core::ptr::read_volatile(&0) } }};
    }
    pub(crate) use select;
}
pub fn run() { runtime::select!(); }
",
    )
    .expect("write source");
    fs::write(root.join("zrail.toml"), CONTRACT).expect("write contract");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("analyze shadowed macro")
        .report;

    assert!(report.findings.iter().any(|finding| {
        finding.id == "RUST-MACRO-001" && finding.message.contains("runtime::select")
    }));
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn bare_local_macro_cannot_borrow_a_standard_name_allowance() {
    let root = root();
    fs::create_dir_all(root.join("src")).expect("create source root");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write manifest");
    fs::write(
        root.join("src/lib.rs"),
        "//! Local shadow.\nmacro_rules! panic { () => {{ unsafe { core::ptr::read_volatile(&0) } }}; }\npub fn run() { panic!(); }\n",
    )
    .expect("write source");
    let contract = CONTRACT.replace("tokio::select", "panic");
    fs::write(root.join("zrail.toml"), contract).expect("write contract");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("analyze locally shadowed macro")
        .report;

    assert!(
        report
            .findings
            .iter()
            .any(|finding| { finding.id == "RUST-MACRO-001" && finding.message.contains("panic") })
    );
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn cross_file_macro_scope_cannot_borrow_a_standard_name_allowance() {
    let root = root();
    fs::create_dir_all(root.join("src")).expect("create source root");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write manifest");
    fs::write(
        root.join("src/lib.rs"),
        "//! Cross-file shadow.\nmacro_rules! panic { () => {{ unsafe { core::ptr::read_volatile(&0) } }}; }\nmod child;\n",
    )
    .expect("write crate root");
    fs::write(
        root.join("src/child.rs"),
        "//! Child module.\npub fn run() { panic!(); }\n",
    )
    .expect("write child module");
    let contract = CONTRACT.replace("tokio::select", "panic");
    fs::write(root.join("zrail.toml"), contract).expect("write contract");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("analyze cross-file macro shadow")
        .report;

    assert!(report.findings.iter().any(|finding| {
        finding.id == "RUST-MACRO-001"
            && finding.path.as_deref() == Some("src/child.rs")
            && finding.analysis == zrail_core::AnalysisQuality::Unresolved
    }));
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn block_local_macro_cannot_borrow_a_standard_name_allowance() {
    let root = root();
    fs::create_dir_all(root.join("src")).expect("create source root");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write manifest");
    fs::write(
        root.join("src/lib.rs"),
        "//! Block shadow.\npub fn run() { macro_rules! panic { () => {{ unsafe { core::ptr::read_volatile(&0) } }}; } panic!(); }\n",
    )
    .expect("write source");
    let contract = CONTRACT.replace("tokio::select", "panic");
    fs::write(root.join("zrail.toml"), contract).expect("write contract");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("analyze block-local macro shadow")
        .report;

    assert!(report.findings.iter().any(|finding| {
        finding.id == "RUST-MACRO-001"
            && finding.message.contains("panic")
            && finding.analysis == zrail_core::AnalysisQuality::Unresolved
    }));
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn macro_use_namespace_cannot_be_allowlisted_as_exact() {
    let root = root();
    fs::create_dir_all(root.join("src")).expect("create source root");
    fs::write(
        root.join("Cargo.toml"),
        concat!(
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
            "[dependencies]\ntokio = { package = \"tokio\", version = \"1\" }\n",
        ),
    )
    .expect("write manifest");
    fs::write(
        root.join("src/lib.rs"),
        "//! Macro import.\n#[macro_use] extern crate tokio;\npub fn run() {}\n",
    )
    .expect("write source");
    let contract = CONTRACT.replace("tokio::select", "macro_use");
    fs::write(root.join("zrail.toml"), contract).expect("write contract");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("analyze macro-use namespace")
        .report;

    assert!(report.findings.iter().any(|finding| {
        finding.id == "RUST-MACRO-001"
            && finding.message.contains("macro_use")
            && finding.analysis == zrail_core::AnalysisQuality::Unresolved
    }));
    fs::remove_dir_all(root).expect("remove fixture");
}

fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-shadow-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("reset fixture");
    }
    root
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
name = "tokio::select"
reason = "The dependency macro expansion is reviewed."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#;
