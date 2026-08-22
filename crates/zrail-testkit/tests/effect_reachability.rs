//! Effect profiles distinguish production target reachability and test-guarded facts.

use std::{fs, path::PathBuf};

use zrail_core::Report;
use zrail_rust::{check_repository, explain_path};

#[test]
fn production_profiles_ignore_nonproduction_targets_and_guarded_facts() {
    let root = repository("production", true);

    let report = check(&root);
    let effects = report
        .findings
        .iter()
        .filter(|finding| finding.id == "EFFECT-001")
        .collect::<Vec<_>>();

    assert_eq!(effects.len(), 2, "{}", report.human());
    assert!(
        effects
            .iter()
            .any(|finding| finding.path.as_deref() == Some("src/lib.rs"))
    );
    assert!(
        effects
            .iter()
            .any(|finding| finding.path.as_deref() == Some("src/shared.rs"))
    );
    assert!(!effects.iter().any(|finding| {
        matches!(
            finding.path.as_deref(),
            Some(
                "tests/integration.rs" | "benches/throughput.rs" | "examples/demo.rs" | "build.rs"
            )
        )
    }));
    reset(&root);
}

#[test]
fn omitted_profile_reachability_preserves_all_fact_behavior() {
    let root = repository("all", false);

    let report = check(&root);

    for path in [
        "tests/integration.rs",
        "benches/throughput.rs",
        "examples/demo.rs",
        "build.rs",
    ] {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.id == "EFFECT-001" && finding.path.as_deref() == Some(path)),
            "{path}: {}",
            report.human()
        );
    }
    let guarded_library_effects = report
        .findings
        .iter()
        .filter(|finding| {
            finding.id == "EFFECT-001" && finding.path.as_deref() == Some("src/lib.rs")
        })
        .count();
    assert_eq!(guarded_library_effects, 8, "{}", report.human());
    reset(&root);
}

#[test]
fn explain_reports_target_and_profile_fact_reachability() {
    let root = repository("explain", true);

    let benchmark = explain_path(
        &root,
        "zrail.toml".as_ref(),
        "benches/throughput.rs".as_ref(),
    )
    .expect("explain benchmark");
    let shared = explain_path(&root, "zrail.toml".as_ref(), "src/shared.rs".as_ref())
        .expect("explain shared source");

    assert_eq!(benchmark.reachability, "benchmark");
    assert_eq!(shared.reachability, "both");
    assert_eq!(
        shared.profile_reachability,
        ["restricted: production files and ordinary facts"]
    );
    assert!(
        shared
            .human()
            .contains("profile reachability: restricted: production")
    );
    reset(&root);
}

fn repository(name: &str, production_only: bool) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-effect-reachability-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    for directory in ["src", "tests", "benches", "examples"] {
        fs::create_dir_all(root.join(directory)).expect("create fixture directory");
    }
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "src/lib.rs", LIBRARY);
    write(&root, "src/shared.rs", SHARED);
    write(
        &root,
        "tests/integration.rs",
        process_source("Integration test."),
    );
    write(&root, "benches/throughput.rs", process_source("Benchmark."));
    write(&root, "examples/demo.rs", process_source("Example."));
    write(
        &root,
        "build.rs",
        "//! Build script.\nfn main() { let _ = std::fs::read(\"input\"); }\n",
    );
    let reachability = if production_only {
        "reachability = \"production\"\n"
    } else {
        ""
    };
    write(
        &root,
        "zrail.toml",
        CONTRACT.replace("{reachability}", reachability),
    );
    root
}

fn check(root: &std::path::Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check reachability fixture")
        .report
}

fn process_source(doc: &str) -> String {
    format!("//! {doc}\nfn run() {{ let _ = std::process::Command::new(\"true\"); }}\n")
}

fn write(root: &std::path::Path, path: &str, contents: impl AsRef<[u8]>) {
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
build = "build.rs"
"#;

const LIBRARY: &str = concat!(
    "//! Library.\n",
    "#[cfg(test)] use std::process::Command;\n",
    "#[cfg(test)] use std::process::*;\n",
    "#[path = \"shared.rs\"]\n",
    "mod shared;\n",
    "#[cfg(test)]\n",
    "#[path = \"shared.rs\"]\n",
    "mod shared_test;\n",
    "pub fn production() { let _ = std::process::Command::new(\"true\"); }\n",
    "#[cfg(test)]\n",
    "mod tests {\n",
    "    fn guarded() { let _ = std::process::Command::new(\"true\"); }\n",
    "}\n",
);

const SHARED: &str = r#"//! Shared source.
pub fn production() { let _ = std::process::Command::new("true"); }
#[cfg(test)]
fn guarded() { let _ = std::process::Command::new("true"); }
"#;

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
mode = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
[profiles.restricted]
{reachability}[profiles.restricted.effects]
deny = ["process", "filesystem"]
[[layer]]
name = "app"
packages = ["fixture"]
profiles = ["restricted"]
reason = "Fixture policy."
"#;
