//! Ownership filters violations and staleness by target and fact reachability.

use std::{fs, path::PathBuf};

use zrail_core::{Finding, Report};
use zrail_rust::{check_repository, explain_path};

#[test]
fn production_owners_ignore_nonproduction_uses_without_hiding_runtime_debt() {
    let root = repository();
    let report = check(&root);

    assert_finding(&report, "runtime-process", "OWN-003", "src/trespasser.rs");
    for path in [
        "src/guarded.rs",
        "tests/integration.rs",
        "benches/throughput.rs",
    ] {
        assert_no_finding(&report, "runtime-process", path);
    }
    assert_no_id(&report, "runtime-process", "OWN-004");

    let stale = finding(&report, "runtime-metadata", "OWN-004", "src/stale.rs");
    assert!(
        stale.message.contains("no production-reachable use"),
        "{}",
        stale.message
    );

    assert_finding(
        &report,
        "default-environment",
        "OWN-003",
        "src/all_outside.rs",
    );
    assert_no_id(&report, "default-environment", "OWN-004");
    assert_no_id(&report, "runtime-network", "OWN-003");
    assert_no_id(&report, "runtime-network", "OWN-004");

    let explanation = explain_path(&root, "zrail.toml".as_ref(), "src/executor.rs".as_ref())
        .expect("explain production owner");
    let owner = explanation
        .call_owners
        .iter()
        .find(|owner| owner.name == "runtime-process")
        .expect("process owner explanation");
    assert_eq!(owner.reachability, "production");
    assert!(explanation.human().contains("reachability production"));
    reset(&root);
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-owner-reachability-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    for directory in ["src", "tests", "benches"] {
        fs::create_dir_all(root.join(directory)).expect("create fixture directory");
    }
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", CONTRACT);
    write(&root, "src/lib.rs", LIBRARY);
    write(&root, "src/executor.rs", process("Runtime process owner."));
    write(&root, "src/trespasser.rs", process("Runtime trespasser."));
    write(
        &root,
        "src/guarded.rs",
        guarded_process("Guarded process use."),
    );
    write(&root, "src/stale.rs", guarded_metadata());
    write(
        &root,
        "src/all_owner.rs",
        guarded_environment("Default owner."),
    );
    write(
        &root,
        "src/all_outside.rs",
        guarded_environment("Default trespasser."),
    );
    write(&root, "src/net_owner.rs", network("Runtime network owner."));
    write(&root, "src/net_guarded.rs", guarded_network());
    write(
        &root,
        "tests/integration.rs",
        process("Integration process use."),
    );
    write(
        &root,
        "benches/throughput.rs",
        process("Benchmark process use."),
    );
    root
}

fn check(root: &std::path::Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check owner fixture")
        .report
}

fn finding<'a>(report: &'a Report, rule: &str, id: &str, path: &str) -> &'a Finding {
    report
        .findings
        .iter()
        .find(|finding| {
            finding.rule == rule && finding.id == id && finding.path.as_deref() == Some(path)
        })
        .unwrap_or_else(|| panic!("missing {rule} {id} at {path}: {}", report.human()))
}

fn assert_finding(report: &Report, rule: &str, id: &str, path: &str) {
    let _ = finding(report, rule, id, path);
}

fn assert_no_finding(report: &Report, rule: &str, path: &str) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| { finding.rule == rule && finding.path.as_deref() == Some(path) }),
        "unexpected {rule} at {path}: {}",
        report.human()
    );
}

fn assert_no_id(report: &Report, rule: &str, id: &str) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule == rule && finding.id == id),
        "unexpected {rule} {id}: {}",
        report.human()
    );
}

fn process(doc: &str) -> String {
    format!("//! {doc}\nfn run() {{ let _ = std::process::Command::new(\"true\"); }}\n")
}

fn guarded_process(doc: &str) -> String {
    format!(
        "//! {doc}\n#[cfg(test)]\nfn run() {{ let _ = std::process::Command::new(\"true\"); }}\n"
    )
}

fn guarded_metadata() -> &'static str {
    "//! Stale runtime owner.\n#[cfg(test)]\nfn run() { let _ = std::fs::metadata(\"file\"); }\n"
}

fn guarded_environment(doc: &str) -> String {
    format!("//! {doc}\n#[cfg(test)]\nfn run() {{ let _ = std::env::var(\"KEY\"); }}\n")
}

fn network(doc: &str) -> String {
    format!("//! {doc}\nfn run() {{ let _ = std::net::TcpStream::connect(\"localhost:1\"); }}\n")
}

fn guarded_network() -> &'static str {
    "//! Guarded network use.\n#[cfg(test)]\nfn run() { let _ = std::net::TcpStream::connect(\"localhost:1\"); }\n"
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

[[bench]]
name = "throughput"
path = "benches/throughput.rs"
harness = false
"#;

const LIBRARY: &str = concat!(
    "//! Library.\n",
    "mod executor;\n",
    "mod trespasser;\n",
    "mod guarded;\n",
    "mod stale;\n",
    "mod all_owner;\n",
    "mod all_outside;\n",
    "mod net_owner;\n",
    "mod net_guarded;\n",
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
[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"

[[owner]]
name = "runtime-process"
kind = "call"
reachability = "production"
within = ["**/*.rs"]
match = "std::process::Command::new"
allow = ["src/executor.rs"]
reason = "One runtime process owner."

[[owner]]
name = "runtime-metadata"
kind = "call"
reachability = "production"
within = ["src/**"]
match = "std::fs::metadata"
allow = ["src/stale.rs"]
reason = "One runtime metadata owner."

[[owner]]
name = "default-environment"
kind = "call"
within = ["src/**"]
match = "std::env::var"
allow = ["src/all_owner.rs"]
reason = "Default reachability sees guarded facts."

[[owner]]
name = "runtime-network"
kind = "capability"
reachability = "production"
within = ["src/**"]
match = "std::net"
allow = ["src/net_owner.rs"]
reason = "One runtime network owner."
"#;
