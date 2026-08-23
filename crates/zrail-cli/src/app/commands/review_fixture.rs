//! Shared protected-review fixtures keep sibling test modules focused on assertions.

use std::{fs, path::PathBuf};

use zrail_rust::build_lock;

use crate::app::{
    args::{CommonOptions, ReviewOptions},
    output::OutputFormat,
};

use super::git_base::commit_all;

pub(super) fn fixture(name: &str) -> Fixture {
    fixture_with_contract(name, CONTRACT)
}

pub(super) fn fixture_with_contract(name: &str, contract: &str) -> Fixture {
    let base = std::env::temp_dir().join(format!(
        "zrail-review-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let fixture = Fixture {
        authority: base.join("authority"),
        proposal: base.join("proposal"),
        base,
    };
    reset(&fixture);
    write_repository(&fixture.authority, contract);
    build_lock(&fixture.authority, std::path::Path::new("zrail.toml"))
        .expect("build authority lock")
        .write(&fixture.authority.join("zrail.lock"))
        .expect("write authority lock");
    commit_all(&fixture.authority);
    copy_repository(&fixture.authority, &fixture.proposal);
    fixture
}

fn write_repository(root: &std::path::Path, contract: &str) {
    fs::create_dir_all(root.join("src")).expect("create source directory");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write Cargo manifest");
    fs::write(root.join("src/lib.rs"), "//! fixture\n").expect("write Rust source");
    fs::write(root.join("zrail.toml"), contract).expect("write contract");
}

fn copy_repository(source: &std::path::Path, destination: &std::path::Path) {
    fs::create_dir_all(destination.join("src")).expect("create proposal source directory");
    for path in ["Cargo.toml", "zrail.toml", "zrail.lock", "src/lib.rs"] {
        let target = destination.join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).expect("create proposal parent");
        }
        fs::copy(source.join(path), target).expect("copy proposal input");
    }
}

pub(super) fn options(fixture: &Fixture) -> ReviewOptions {
    ReviewOptions {
        common: CommonOptions {
            root: fixture.proposal.clone(),
            config: "zrail.toml".into(),
            lock: "zrail.lock".into(),
            format: OutputFormat::Human,
            ..CommonOptions::default()
        },
        authority_root: fixture.authority.clone(),
        base: "HEAD".into(),
        allow_grants: false,
    }
}

pub(super) fn reset(fixture: &Fixture) {
    if fixture.base.exists() {
        fs::remove_dir_all(&fixture.base).expect("reset fixture");
    }
}

pub(super) struct Fixture {
    base: PathBuf,
    authority: PathBuf,
    pub(super) proposal: PathBuf,
}

pub(super) const CONTRACT: &str = r#"schema = 1
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
