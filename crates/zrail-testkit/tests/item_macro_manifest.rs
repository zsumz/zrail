//! Exact item-macro manifests produce complete content-bound namespace authority.

use std::{fs, path::PathBuf};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn exact_manifest_binds_generated_names_invocation_and_lock_state() {
    let root = fixture();
    let lock = build_lock(&root, "zrail.toml".as_ref()).expect("build exact manifest lock");
    assert_eq!(lock.item_macro_manifests.len(), 1);
    let authority = &lock.item_macro_manifests[0];
    assert_eq!(authority.bindings, 1);
    assert_eq!(authority.definition, "repository:src/lib.rs::declare");
    assert_eq!(authority.guard, "ordinary");
    assert_eq!(authority.domains.len(), 2);
    assert!(
        authority
            .domains
            .iter()
            .any(|domain| domain.ends_with("mode=library"))
    );
    assert!(
        authority
            .domains
            .iter()
            .any(|domain| domain.ends_with("mode=library-test"))
    );
    lock.write(&root.join("zrail.lock")).expect("write lock");

    let checked = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check exact manifested namespace");
    assert_eq!(checked.report.status, ReportStatus::Pass);
    assert!(checked.analysis.is_complete());

    fs::write(root.join("src/lib.rs"), source_with_field("u64", ""))
        .expect("change macro definition");
    let changed = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("definition changes remain reviewable lock drift");
    assert!(changed.report.findings.iter().any(|finding| {
        finding.id == "LOCK-035" && finding.message.contains("definition or invocation")
    }));

    fs::write(root.join("src/lib.rs"), source("unexpected")).expect("change invocation");
    let error = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect_err("changed invocation tokens must fail before lock construction");
    assert!(error.to_string().contains("invocation tokens differ"));
    reset(&root);
}

#[test]
fn external_manifest_binds_one_exact_cargo_lock_package() {
    let root = external_fixture();

    let lock = build_lock(&root, "zrail.toml".as_ref()).expect("build external manifest lock");
    let authority = &lock.item_macro_manifests[0];
    assert_eq!(
        authority.definition,
        "cargo-lock:provider:1.0.0:registry+https://github.com/rust-lang/crates.io-index"
    );
    assert_eq!(authority.definition_sha256, "1".repeat(64));

    fs::write(
        root.join("zrail.toml"),
        EXTERNAL_CONTRACT.replace("version = \"1.0.0\"", "version = \"2.0.0\""),
    )
    .expect("select wrong same-name version");
    let error = build_lock(&root, "zrail.toml".as_ref())
        .expect_err("wrong same-name package version must fail closed");
    assert!(
        error
            .to_string()
            .contains("does not match Cargo.lock authority")
    );
    reset(&root);
}

fn fixture() -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("zrail-item-macro-manifest-{}", std::process::id()));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source");
    fs::create_dir_all(root.join("zrail/macros")).expect("create manifest directory");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.0.0'\nedition='2024'\n",
    )
    .expect("write Cargo manifest");
    fs::write(root.join("src/lib.rs"), source("")).expect("write source");
    fs::write(root.join("zrail.toml"), contract()).expect("write contract");
    fs::write(root.join("zrail/macros/declare.toml"), manifest()).expect("write exact manifest");
    root
}

fn external_fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-external-item-macro-manifest-{}",
        std::process::id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source");
    fs::create_dir_all(root.join("zrail/macros")).expect("create manifest directory");
    fs::write(root.join("Cargo.toml"), EXTERNAL_CARGO).expect("write Cargo manifest");
    fs::write(root.join("Cargo.lock"), EXTERNAL_LOCK).expect("write Cargo lock");
    fs::write(root.join("src/lib.rs"), EXTERNAL_SOURCE).expect("write source");
    fs::write(root.join("zrail.toml"), EXTERNAL_CONTRACT).expect("write contract");
    fs::write(root.join("zrail/macros/declare.toml"), external_manifest())
        .expect("write exact manifest");
    root
}

fn source(input: &str) -> String {
    source_with_field("usize", input)
}

fn source_with_field(field: &str, input: &str) -> String {
    format!(
        "//! Exact manifested namespace.\n\
         macro_rules! declare {{ ($($token:tt)*) => {{ struct Generated {{ value: {field} }} }} }}\n\
         declare!({input});\n\
         fn accepts_generated(_: Generated) {{}}\n"
    )
}

fn manifest() -> &'static str {
    r#"schema = 1
macro_name = "declare"
invocation_sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

[[binding]]
name = "Generated"
kind = "type"
public = false
"#
}

fn external_manifest() -> &'static str {
    r#"schema = 1
macro_name = "provider::declare"
invocation_sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

[[binding]]
name = "Generated"
kind = "type"
public = false
"#
}

fn contract() -> &'static str {
    r#"schema = 2
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
module_docs = "allow"
facades = "allow"
tests = "allow"

[[source.rust.item_macros]]
name = "declare"
path = "src/lib.rs"
resolution = "exact"
manifest = "zrail/macros/declare.toml"
reason = "The checked-in manifest declares the complete generated namespace."

[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
"#
}

const EXTERNAL_SOURCE: &str =
    "//! Exact external namespace.\nprovider::declare!();\nfn accepts_generated(_: Generated) {}\n";

const EXTERNAL_CARGO: &str = r#"[package]
name = "fixture"
version = "0.0.0"
edition = "2024"

[dependencies]
provider = { package = "provider", version = "1" }
"#;

const EXTERNAL_LOCK: &str = r#"version = 4

[[package]]
name = "fixture"
version = "0.0.0"
dependencies = ["provider 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)"]

[[package]]
name = "provider"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1111111111111111111111111111111111111111111111111111111111111111"

[[package]]
name = "provider"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2222222222222222222222222222222222222222222222222222222222222222"
"#;

const EXTERNAL_CONTRACT: &str = r#"schema = 2
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
module_docs = "allow"
facades = "allow"
tests = "allow"

[[source.rust.item_macros]]
name = "provider::declare"
path = "src/lib.rs"
resolution = "exact"
source = { kind = "cargo-lock", package = "provider", version = "1.0.0" }
manifest = "zrail/macros/declare.toml"
reason = "The exact locked provider owns this manifested namespace."

[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
"#;

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
