//! Repository macro authority follows exact origin and package implementation content.

use std::{
    fs,
    path::{Path, PathBuf},
};

use zrail_core::{Finding, ReportStatus};
use zrail_rust::{build_lock, check_repository};

#[test]
fn qualified_local_macros_bind_the_package_and_transitive_helpers() {
    let root = root("qualified-local");
    package(&root, "fixture");
    let safe = r"//! Local macro package.
mod helpers {
    macro_rules! helper { () => { 42 }; }
    macro_rules! reviewed { () => { helper!() }; }
    pub(crate) use reviewed;
}
pub fn run() { let _ = helpers::reviewed!(); let _ = crate::helpers::reviewed!(); }
";
    write(&root, "src/lib.rs", safe);
    contract(
        &root,
        &[
            allowance_with_definition("helpers::reviewed", "src/lib.rs"),
            allowance("crate::helpers::reviewed"),
        ],
    );
    lock(&root);
    let report = check(&root);
    assert_eq!(report.status, ReportStatus::Pass, "{:#?}", report.findings);

    write(
        &root,
        "src/lib.rs",
        &safe.replace("42", "unsafe { core::ptr::read_volatile(&0) }"),
    );
    assert_finding(&check(&root).findings, "LOCK-023", "fixture");
    reset(&root);
}

#[test]
fn workspace_macro_package_changes_invalidate_repository_authority() {
    let root = root("workspace-package");
    workspace(&root, &["app", "macros"], &[]);
    member(
        &root,
        "app",
        "app",
        "workspace_macros = { package = \"workspace-macros\", path = \"../macros\" }",
    );
    member(&root, "macros", "workspace-macros", "");
    write(
        &root,
        "macros/src/lib.rs",
        "//! Macro implementation.\n#[macro_export]\nmacro_rules! reviewed { () => { 42 }; }\n",
    );
    write(
        &root,
        "app/src/lib.rs",
        "//! Consumer.\npub fn run() { let _ = workspace_macros::reviewed!(); }\n",
    );
    contract(&root, &[allowance("workspace_macros::reviewed")]);
    lock(&root);
    let report = check(&root);
    assert_eq!(report.status, ReportStatus::Pass, "{:#?}", report.findings);

    write(
        &root,
        "macros/src/lib.rs",
        "//! Changed macro implementation.\n#[macro_export]\nmacro_rules! reviewed { () => { 43 }; }\n",
    );
    assert_finding(&check(&root).findings, "LOCK-023", "workspace-macros");
    reset(&root);
}

#[test]
fn internal_proc_macro_implementation_is_package_bound() {
    let root = root("proc-macro");
    workspace(&root, &["app", "macros"], &[]);
    member(
        &root,
        "app",
        "app",
        "workspace_macros = { package = \"workspace-macros\", path = \"../macros\" }",
    );
    write(
        &root,
        "macros/Cargo.toml",
        "[package]\nname = \"workspace-macros\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[lib]\nproc-macro = true\n",
    );
    write(
        &root,
        "macros/src/lib.rs",
        "//! Proc macro.\nuse proc_macro::TokenStream;\n#[proc_macro_derive(Reviewed)]\npub fn reviewed(input: TokenStream) -> TokenStream { input }\n",
    );
    write(
        &root,
        "app/src/lib.rs",
        "//! Consumer.\n#[derive(workspace_macros::Reviewed)]\npub struct Model;\n",
    );
    contract(&root, &[allowance("workspace_macros::Reviewed")]);
    lock(&root);
    let report = check(&root);
    assert_eq!(report.status, ReportStatus::Pass, "{:#?}", report.findings);

    let source = fs::read_to_string(root.join("macros/src/lib.rs")).expect("read proc macro");
    write(
        &root,
        "macros/src/lib.rs",
        &source.replace("{ input }", "{ let _ = 1; input }"),
    );
    assert_finding(&check(&root).findings, "LOCK-023", "workspace-macros");
    reset(&root);
}

#[test]
fn excluded_repository_path_macro_package_is_content_bound() {
    let root = root("repository-path");
    workspace(&root, &["app"], &["vendor/macros"]);
    member(
        &root,
        "app",
        "app",
        "vendor_macros = { package = \"vendor-macros\", path = \"../vendor/macros\" }",
    );
    member(&root, "vendor/macros", "vendor-macros", "");
    write(
        &root,
        "vendor/macros/src/lib.rs",
        "//! Repository path macro.\n#[macro_export]\nmacro_rules! reviewed { () => { 1 }; }\n",
    );
    write(
        &root,
        "app/src/lib.rs",
        "//! Consumer.\npub fn run() { let _ = vendor_macros::reviewed!(); }\n",
    );
    contract(&root, &[allowance("vendor_macros::reviewed")]);
    lock(&root);
    let report = check(&root);
    assert_eq!(report.status, ReportStatus::Pass, "{:#?}", report.findings);

    write(
        &root,
        "vendor/macros/src/lib.rs",
        "//! Changed repository path macro.\n#[macro_export]\nmacro_rules! reviewed { () => { 2 }; }\n",
    );
    assert_finding(&check(&root).findings, "LOCK-023", "vendor-macros");
    reset(&root);
}

#[test]
fn external_package_named_local_needs_external_source_authority() {
    let root = root("external-local");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nlocal = \"1\"\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! External macro.\npub fn run() { local::reviewed!(); }\n",
    );
    contract(&root, &[allowance("local::reviewed")]);

    assert_finding(&check(&root).findings, "RUST-MACRO-006", "local::reviewed");
    reset(&root);
}

fn root(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("zrail-macro-origin-{name}-{}", std::process::id()));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    root
}

fn package(root: &Path, name: &str) {
    write(
        root,
        "Cargo.toml",
        &format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n"),
    );
}

fn workspace(root: &Path, members: &[&str], exclude: &[&str]) {
    write(
        root,
        "Cargo.toml",
        &format!("[workspace]\nmembers = {members:?}\nexclude = {exclude:?}\nresolver = \"3\"\n"),
    );
}

fn member(root: &Path, directory: &str, name: &str, dependencies: &str) {
    fs::create_dir_all(root.join(directory).join("src")).expect("create member");
    write(
        root,
        &format!("{directory}/Cargo.toml"),
        &format!(
            "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\n{dependencies}\n"
        ),
    );
}

fn allowance(name: &str) -> String {
    format!(
        "[[source.rust.macros.allow]]\nname = \"{name}\"\nreason = \"Reviewed repository macro package.\"\n"
    )
}

fn allowance_with_definition(name: &str, definition: &str) -> String {
    format!(
        "[[source.rust.macros.allow]]\nname = \"{name}\"\ndefinition = \"{definition}\"\nreason = \"Reviewed repository macro definition.\"\n"
    )
}

fn contract(root: &Path, allowances: &[String]) {
    write(
        root,
        "zrail.toml",
        &format!("{CONTRACT}\n{}", allowances.join("\n")),
    );
}

fn lock(root: &Path) {
    build_lock(root, "zrail.toml".as_ref())
        .expect("build lock")
        .write(&root.join("zrail.lock"))
        .expect("write lock");
}

fn check(root: &Path) -> zrail_core::Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check fixture")
        .report
}

fn assert_finding(findings: &[Finding], id: &str, text: &str) {
    assert!(
        findings
            .iter()
            .any(|finding| finding.id == id && finding.message.contains(text)),
        "missing {id} for {text}: {findings:#?}"
    );
}

fn write(root: &Path, path: &str, contents: &str) {
    let path = root.join(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, contents).expect("write fixture");
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
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;
