//! Lock updates require explicit acceptance before recording gated architecture changes.

use std::{fs, path::PathBuf};

use zrail_core::sha256_hex;
use zrail_rust::build_lock;

use crate::app::{
    args::{CommonOptions, UpdateOptions},
    commands::git_base::{commit_all, git_available},
    output::OutputFormat,
};

use super::update;

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
mode = "locked"
unassigned_packages = "allow"
cycles = "deny"

[source.rust]
module_docs = "required"
facades = "declarative"
tests = "sibling"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "deny"
deny_methods = []
deny_macros = []

[source.rust.size.facade]
target = 80
hard = 120

[source.rust.size.implementation]
target = 240
hard = 300

[source.rust.size.test]
target = 300
hard = 300

[source.rust.size.auxiliary]
target = 300
hard = 300
"#;

#[test]
fn update_refuses_new_dependency_without_acceptance() {
    if !git_available() {
        return;
    }
    let root = fixture_root("dependency");
    reset(&root);
    write_fixture(&root);
    let lock_path = root.join("zrail.lock");
    build_lock(&root, std::path::Path::new("zrail.toml"))
        .expect("build initial lock")
        .write(&lock_path)
        .expect("write initial lock");
    commit_all(&root);
    let initial = fs::read_to_string(&lock_path).expect("read initial lock");
    fs::write(
        root.join("crates/fixture/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"fixture\"\n",
            "version = \"0.0.0\"\n",
            "edition = \"2024\"\n\n",
            "[dependencies]\n",
            "serde = \"1\"\n",
        ),
    )
    .expect("add dependency");

    let mut options = options(&root);
    let refused = update(&options).expect("evaluate refused update");
    assert_eq!(refused.exit_code, 1);
    assert!(refused.text.contains("refused gated architecture changes"));
    assert_eq!(fs::read_to_string(&lock_path).expect("read lock"), initial);

    options.accept_grants = true;
    let accepted = update(&options).expect("accept update");
    assert_eq!(accepted.exit_code, 0);
    assert!(
        fs::read_to_string(&lock_path)
            .expect("read accepted lock")
            .contains("name = \"serde\"")
    );
    reset(&root);
}

#[test]
fn update_refuses_changed_generated_provenance_without_acceptance() {
    if !git_available() {
        return;
    }
    let root = fixture_root("generated");
    reset(&root);
    write_generated_fixture(&root, "one");
    let lock_path = root.join("zrail.lock");
    build_lock(&root, std::path::Path::new("zrail.toml"))
        .expect("build initial lock")
        .write(&lock_path)
        .expect("write initial lock");
    commit_all(&root);
    let initial = fs::read_to_string(&lock_path).expect("read initial lock");
    write_manifest(&root, "two");

    let mut options = options(&root);
    let refused = update(&options).expect("evaluate provenance update");

    assert_eq!(refused.exit_code, 1);
    assert!(refused.text.contains("UNKNOWN rust.generated-provenance"));
    assert_eq!(fs::read_to_string(&lock_path).expect("read lock"), initial);

    options.accept_grants = true;
    let accepted = update(&options).expect("accept provenance update");
    assert_eq!(accepted.exit_code, 0);
    assert_ne!(fs::read_to_string(&lock_path).expect("read lock"), initial);
    reset(&root);
}

#[test]
fn update_refuses_changed_gate_bytes_without_acceptance() {
    if !git_available() {
        return;
    }
    let root = fixture_root("gate");
    reset(&root);
    write_gate_fixture(&root);
    let lock_path = root.join("zrail.lock");
    build_lock(&root, std::path::Path::new("zrail.toml"))
        .expect("build initial lock")
        .write(&lock_path)
        .expect("write initial lock");
    commit_all(&root);
    let initial = fs::read_to_string(&lock_path).expect("read initial lock");
    fs::write(
        root.join("scripts/check"),
        "cargo test --workspace\ncargo fmt --check\n",
    )
    .expect("change gate");

    let mut options = options(&root);
    let refused = update(&options).expect("evaluate gate update");

    assert_eq!(refused.exit_code, 1);
    assert!(refused.text.contains("UNKNOWN qualification.gate-lock"));
    assert_eq!(fs::read_to_string(&lock_path).expect("read lock"), initial);

    options.accept_grants = true;
    let accepted = update(&options).expect("accept gate update");
    assert_eq!(accepted.exit_code, 0);
    assert!(accepted.text.contains("gates: 1"));
    reset(&root);
}

fn options(root: &std::path::Path) -> UpdateOptions {
    UpdateOptions {
        common: CommonOptions {
            root: root.to_path_buf(),
            config: PathBuf::from("zrail.toml"),
            lock: PathBuf::from("zrail.lock"),
            format: OutputFormat::Human,
        },
        base: "HEAD".into(),
        accept_grants: false,
    }
}

fn write_fixture(root: &std::path::Path) {
    fs::create_dir_all(root.join("crates/fixture/src")).expect("create fixture");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/fixture\"]\nresolver = \"3\"\n",
    )
    .expect("write workspace");
    fs::write(
        root.join("crates/fixture/Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write package");
    fs::write(root.join("crates/fixture/src/lib.rs"), "//! fixture\n").expect("write source");
    fs::write(root.join("zrail.toml"), CONTRACT).expect("write contract");
}

fn write_generated_fixture(root: &std::path::Path, upstream: &str) {
    write_fixture(root);
    fs::create_dir_all(root.join("crates/fixture/src/generated")).expect("create generated root");
    fs::create_dir_all(root.join("schemas")).expect("create schema root");
    fs::write(root.join("schemas/wire.schema"), "message Wire {}\n").expect("write schema");
    fs::write(
        root.join("crates/fixture/src/generated/wire.rs"),
        "//! @generated by fixture\npub const WIRE: u8 = 1;\n",
    )
    .expect("write generated source");
    fs::write(
        root.join("zrail.toml"),
        format!(
            concat!(
                "{CONTRACT}\n",
                "[[source.rust.generated]]\n",
                "root = \"crates/fixture/src/generated\"\n",
                "manifest = \"crates/fixture/src/generated/MANIFEST.json\"\n",
                "inputs = [\"schemas/**\"]\n",
                "target = 300\n",
                "hard = 300\n",
                "reason = \"fixture provenance\"\n",
                "auxiliary = [\"wire.rs\"]\n",
            ),
            CONTRACT = CONTRACT
        ),
    )
    .expect("write generated contract");
    write_manifest(root, upstream);
}

fn write_gate_fixture(root: &std::path::Path) {
    write_fixture(root);
    fs::create_dir_all(root.join("docs")).expect("create docs");
    fs::create_dir_all(root.join("scripts")).expect("create scripts");
    fs::write(
        root.join("crates/fixture/src/lib.rs"),
        "//! fixture\n#[cfg(test)]\n#[path = \"architecture_test.rs\"]\nmod architecture_test;\n",
    )
    .expect("write test owner");
    fs::write(
        root.join("crates/fixture/src/architecture_test.rs"),
        "//! evidence\n#[test]\nfn gate_is_reviewed() {}\n",
    )
    .expect("write evidence");
    fs::write(root.join("docs/architecture.md"), "# Architecture\n").expect("write document");
    fs::write(root.join("scripts/check"), "cargo test --workspace\n").expect("write gate");
    fs::write(
        root.join("zrail.toml"),
        format!(
            concat!(
                "{CONTRACT}\n",
                "[[gate]]\nname = \"check\"\nkind = \"local\"\npath = \"scripts/check\"\n",
                "reason = \"fixture gate\"\n\n",
                "[[invariant]]\nid = \"ARCH-01\"\ntitle = \"Gate is reviewed\"\n",
                "status = \"enforced\"\ndocument = \"docs/architecture.md#architecture\"\n",
                "evidence = [\"rust-test:crates/fixture/src/architecture_test.rs::gate_is_reviewed\", \"gate:check\"]\n",
            ),
            CONTRACT = CONTRACT
        ),
    )
    .expect("write gate contract");
}

fn write_manifest(root: &std::path::Path, upstream: &str) {
    let schema = "message Wire {}\n";
    let source = "//! @generated by fixture\npub const WIRE: u8 = 1;\n";
    let manifest = format!(
        concat!(
            "{{\"schema\":1,\"generator\":\"fixture\",\"upstream\":\"{}\",",
            "\"inputs\":[{{\"path\":\"schemas/wire.schema\",\"sha256\":\"{}\"}}],",
            "\"files\":[{{\"path\":\"wire.rs\",\"sha256\":\"{}\"}}]}}"
        ),
        upstream,
        sha256_hex(schema.as_bytes()),
        sha256_hex(source.as_bytes())
    );
    fs::write(
        root.join("crates/fixture/src/generated/MANIFEST.json"),
        manifest,
    )
    .expect("write generated manifest");
}

fn fixture_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("zrail-update-{name}-{}", std::process::id()))
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
