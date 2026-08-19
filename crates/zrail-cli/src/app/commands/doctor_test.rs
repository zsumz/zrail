//! Doctor command status and exit behavior agree on unsupported locks.

use std::{fs, path::PathBuf};

use zrail_core::LOCK_SCHEMA;
use zrail_rust::build_lock;

use crate::app::{args::CommonOptions, output::OutputFormat};

use super::doctor;

#[test]
fn unsupported_lock_schema_exits_nonzero() {
    let root = fixture_root();
    if root.exists() {
        fs::remove_dir_all(&root).expect("reset doctor fixture");
    }
    fs::create_dir_all(root.join("src")).expect("create doctor source");
    fs::write(root.join("Cargo.toml"), MANIFEST).expect("write Cargo manifest");
    fs::write(root.join("src/lib.rs"), "//! fixture\n").expect("write Rust source");
    fs::write(root.join("zrail.toml"), CONTRACT).expect("write contract");
    let mut lock =
        build_lock(&root, std::path::Path::new("zrail.toml")).expect("build supported lock");
    lock.schema = LOCK_SCHEMA + 1;
    lock.write(&root.join("zrail.lock"))
        .expect("write unsupported lock");

    let result = doctor(&CommonOptions {
        root: root.clone(),
        config: "zrail.toml".into(),
        lock: "zrail.lock".into(),
        format: OutputFormat::Human,
    })
    .expect("run doctor");

    assert_eq!(result.exit_code, 1);
    assert!(result.text.contains("status: lock-schema-mismatch"));
    fs::remove_dir_all(root).expect("remove doctor fixture");
}

fn fixture_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "zrail-doctor-schema-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ))
}

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";

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
mode = "locked"
unassigned_packages = "allow"
cycles = "deny"

[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#;
