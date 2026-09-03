//! Macro lookup evaluates each physical fragment in its logical include scope.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn included_fragment_borrows_the_parent_dependency_scope() {
    let root = fixture("inherited-scope", &["macro_alpha"]);
    write(
        &root,
        "crates/consumer/src/lib.rs",
        "//! Consumer.\nuse macro_alpha::reviewed;\ninclude!(\"fragment.rs\");\n",
    );
    write(
        &root,
        "crates/consumer/src/fragment.rs",
        "//! Included behavior.\npub fn run() { reviewed!(); }\n",
    );
    write_contract(&root, &["macro_alpha"]);

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn included_import_contributes_to_the_parent_macro_scope() {
    let root = fixture("included-import", &["macro_alpha"]);
    write(
        &root,
        "crates/consumer/src/lib.rs",
        "//! Consumer.\ninclude!(\"imports.rs\");\npub fn run() { reviewed!(); }\n",
    );
    write(
        &root,
        "crates/consumer/src/imports.rs",
        "//! Included imports.\nuse macro_alpha::reviewed;\n",
    );
    write_contract(&root, &["macro_alpha"]);

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn mounted_invocation_diagnostics_keep_the_physical_fragment_path() {
    let root = fixture("physical-diagnostic", &["macro_alpha"]);
    write(
        &root,
        "crates/consumer/src/lib.rs",
        "//! Consumer.\nuse macro_alpha::reviewed;\ninclude!(\"fragment.rs\");\n",
    );
    write(
        &root,
        "crates/consumer/src/fragment.rs",
        "//! Included behavior.\npub fn run() { reviewed!(); }\n",
    );
    write_contract(&root, &[]);

    let report = check(&root);

    assert!(
        report.findings.iter().any(|finding| {
            finding.id.starts_with("RUST-MACRO-")
                && finding.path.as_deref() == Some("crates/consumer/src/fragment.rs")
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

#[test]
fn one_fragment_resolves_independently_in_two_logical_mounts() {
    let root = fixture("distinct-mounts", &["macro_alpha", "macro_beta"]);
    write(
        &root,
        "crates/consumer/src/lib.rs",
        r#"//! Consumer.
mod alpha {
    use macro_alpha::reviewed;
    include!("fragment.rs");
}
mod beta {
    use macro_beta::reviewed;
    include!("fragment.rs");
}
"#,
    );
    write(
        &root,
        "crates/consumer/src/fragment.rs",
        "//! Included behavior.\npub fn run() { reviewed!(); }\n",
    );
    write_contract(&root, &["macro_alpha", "macro_beta"]);

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

fn fixture(name: &str, dependencies: &[&str]) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-mount-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("crates/consumer/src")).expect("create consumer fixture");
    write(&root, "Cargo.toml", WORKSPACE_MANIFEST);
    let dependencies = dependencies
        .iter()
        .map(|package| format!("{package} = {{ path = \"../{package}\" }}"))
        .collect::<Vec<_>>()
        .join("\n");
    write(
        &root,
        "crates/consumer/Cargo.toml",
        &format!(
            "{}[dependencies]\n{dependencies}\n",
            package_manifest("consumer")
        ),
    );
    for package in dependencies.lines().filter_map(dependency_name) {
        fs::create_dir_all(root.join(format!("crates/{package}/src")))
            .expect("create macro package fixture");
        write(
            &root,
            &format!("crates/{package}/Cargo.toml"),
            &package_manifest(package),
        );
        write(
            &root,
            &format!("crates/{package}/src/lib.rs"),
            "//! Macro package.\n#[macro_export]\nmacro_rules! reviewed { () => {}; }\n",
        );
    }
    root
}

fn dependency_name(line: &str) -> Option<&str> {
    line.split_once(" = ").map(|(name, _)| name)
}

fn package_manifest(name: &str) -> String {
    format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n")
}

fn write_contract(root: &Path, packages: &[&str]) {
    let mut allowances = String::new();
    for package in packages {
        allowances.push_str("[[source.rust.macros.allow]]\nname = \"");
        allowances.push_str(package);
        allowances.push_str("::reviewed\"\ndefinition = \"crates/");
        allowances.push_str(package);
        allowances
            .push_str("/src/lib.rs\"\nreason = \"Reviewed repository dependency expansion.\"\n");
    }
    write(root, "zrail.toml", &format!("{CONTRACT}{allowances}"));
}

fn check(root: &Path) -> zrail_core::Report {
    build_lock(root, "zrail.toml".as_ref())
        .expect("build logical mount lock")
        .write(&root.join("zrail.lock"))
        .expect("write logical mount lock");
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check logical mount fixture")
        .report
}

fn write(root: &Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const WORKSPACE_MANIFEST: &str = "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"3\"\n";

const CONTRACT: &str = r#"schema = 1
adapters = ["rust"]
[repository]
roots = ["crates"]
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
