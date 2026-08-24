//! Resolved dependency denials remain exact, deterministic, and fail closed.

use std::{fs, path::PathBuf};

use zrail_core::Finding;
use zrail_rust::check_repository;

#[test]
fn transitive_policy_reports_the_shortest_exact_locked_path() {
    let root = repository("shortest", MANIFEST, GRAPH_LOCK);
    write_contract(&root, "transitive", "['normal']");

    let finding = dependency_finding(&root, "DEP-008");

    assert!(finding.message.contains("normal dependency path"));
    assert!(finding.message.contains("bridge 1.2.3"));
    assert!(finding.message.contains("blocked 3.0.0"));
    assert!(finding.message.contains("checksum=bbbb"));
    assert!(!finding.message.contains("relay"), "{}", finding.message);

    write_contract(&root, "direct", "['normal']");
    assert_no_dependency_finding(&root, "DEP-008");
    reset(&root);
}

#[test]
fn dependency_kind_selects_the_honest_first_manifest_edge() {
    let root = repository("kind", MANIFEST, GRAPH_LOCK);
    write_contract(&root, "transitive", "['development']");
    assert_no_dependency_finding(&root, "DEP-008");

    write_contract(&root, "transitive", "['build']");
    let finding = dependency_finding(&root, "DEP-008");

    assert!(finding.message.contains("build dependency path"));
    assert!(finding.message.contains("detour 1.0.0"));
    assert!(finding.message.contains("relay 2.0.0"));
    reset(&root);
}

#[test]
fn ambiguous_manifest_to_lock_mapping_fails_closed() {
    let root = repository("ambiguous", AMBIGUOUS_MANIFEST, AMBIGUOUS_LOCK);
    write_contract(&root, "transitive", "['normal']");

    let finding = dependency_finding(&root, "DEP-011");

    assert!(finding.message.contains("maps ambiguously"));
    assert!(finding.message.contains("bridge 1.4.0"));
    assert!(finding.message.contains("bridge 2.1.0"));
    assert_no_dependency_finding(&root, "DEP-008");
    reset(&root);
}

#[test]
fn transitive_policy_requires_a_checked_in_cargo_lock() {
    let root = repository("missing-lock", MANIFEST, "");
    fs::remove_file(root.join("Cargo.lock")).expect("remove fixture lock");
    write_contract(&root, "transitive", "['normal']");

    let finding = dependency_finding(&root, "DEP-011");

    assert!(finding.message.contains("requires Cargo.lock"));
    reset(&root);
}

fn dependency_finding(root: &std::path::Path, id: &str) -> Finding {
    findings(root)
        .into_iter()
        .find(|finding| finding.id == id && finding.rule == "blocked-path")
        .unwrap_or_else(|| panic!("missing {id} dependency finding"))
}

fn assert_no_dependency_finding(root: &std::path::Path, id: &str) {
    let findings = findings(root);
    assert!(
        !findings
            .iter()
            .any(|finding| finding.id == id && finding.rule == "blocked-path"),
        "unexpected {id}: {findings:#?}"
    );
}

fn findings(root: &std::path::Path) -> Vec<Finding> {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("analyze dependency fixture")
        .report
        .findings
}

fn repository(name: &str, manifest: &str, lock: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-transitive-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture source");
    fs::write(root.join("Cargo.toml"), manifest).expect("write manifest");
    fs::write(root.join("Cargo.lock"), lock).expect("write Cargo.lock");
    fs::write(root.join("src/lib.rs"), "//! Generic dependency fixture.\n").expect("write source");
    root
}

fn write_contract(root: &std::path::Path, reachability: &str, kinds: &str) {
    fs::write(
        root.join("zrail.toml"),
        CONTRACT
            .replace("REACHABILITY", reachability)
            .replace("KINDS", kinds),
    )
    .expect("write contract");
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

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
lint_suppressions = "allow"
deny_methods = []
deny_macros = []

[[dependency]]
name = "blocked-path"
from = "app"
deny = ["blocked"]
reachability = "REACHABILITY"
kinds = KINDS
reason = "The selected dependency path is prohibited."
"#;

const MANIFEST: &str = r#"[package]
name = "app"
version = "0.1.0"
edition = "2024"

[dependencies]
bridge = "1"

[build-dependencies]
detour = "1"
"#;

const AMBIGUOUS_MANIFEST: &str = r#"[package]
name = "app"
version = "0.1.0"
edition = "2024"

[dependencies]
bridge = "*"
"#;

const GRAPH_LOCK: &str = r#"version = 4

[[package]]
name = "app"
version = "0.1.0"
dependencies = ["bridge", "detour"]

[[package]]
name = "bridge"
version = "1.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
dependencies = ["blocked"]

[[package]]
name = "blocked"
version = "3.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"

[[package]]
name = "detour"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
dependencies = ["relay"]

[[package]]
name = "relay"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
dependencies = ["blocked"]
"#;

const AMBIGUOUS_LOCK: &str = r#"version = 4

[[package]]
name = "app"
version = "0.1.0"
dependencies = [
 "bridge 1.4.0 (registry+https://github.com/rust-lang/crates.io-index)",
 "bridge 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
]

[[package]]
name = "bridge"
version = "1.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1111111111111111111111111111111111111111111111111111111111111111"

[[package]]
name = "bridge"
version = "2.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2222222222222222222222222222222222222222222222222222222222222222"
"#;
