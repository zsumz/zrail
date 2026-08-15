//! Protected review recomputes source-aware dependency identity from proposal data.

use std::{fs, path::PathBuf};

use zrail_rust::build_lock;

use crate::app::{
    args::{CommonOptions, ReviewOptions},
    output::OutputFormat,
};

use super::{
    git_base::{commit_all, git_available},
    review,
};

#[test]
fn same_name_git_dependency_cannot_reuse_an_internal_lock() {
    if !git_available() {
        return;
    }
    let fixture = fixture();
    let manifest = fixture.proposal.join("crates/app/Cargo.toml");
    fs::write(
        manifest,
        package(
            "app",
            "zrail-core = { git = \"https://example.test/zrail-core\" }",
        ),
    )
    .expect("replace internal source with same-name Git source");

    let result = review(&options(&fixture)).expect("review source substitution");

    assert_eq!(result.exit_code, 1);
    assert!(result.text.contains("error[REVIEW-002]"), "{}", result.text);
    assert!(result.text.contains("error[LOCK-005]"), "{}", result.text);
    reset(&fixture.base);
}

fn fixture() -> Fixture {
    let base = std::env::temp_dir().join(format!(
        "zrail-review-dependency-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&base);
    let fixture = Fixture {
        authority: base.join("authority"),
        proposal: base.join("proposal"),
        base,
    };
    write_repository(&fixture.authority);
    build_lock(&fixture.authority, std::path::Path::new("zrail.toml"))
        .expect("build authority lock")
        .write(&fixture.authority.join("zrail.lock"))
        .expect("write authority lock");
    commit_all(&fixture.authority);
    write_repository(&fixture.proposal);
    fs::copy(
        fixture.authority.join("zrail.lock"),
        fixture.proposal.join("zrail.lock"),
    )
    .expect("copy reviewed lock");
    fixture
}

fn write_repository(root: &std::path::Path) {
    for member in ["app", "zrail-core"] {
        fs::create_dir_all(root.join(format!("crates/{member}/src"))).expect("create package");
        fs::write(
            root.join(format!("crates/{member}/src/lib.rs")),
            format!("//! {member}\n"),
        )
        .expect("write source");
    }
    fs::write(root.join("Cargo.toml"), WORKSPACE).expect("write workspace");
    fs::write(
        root.join("crates/app/Cargo.toml"),
        package("app", "zrail-core = { path = \"../zrail-core\" }"),
    )
    .expect("write app manifest");
    fs::write(
        root.join("crates/zrail-core/Cargo.toml"),
        package("zrail-core", ""),
    )
    .expect("write core manifest");
    fs::write(root.join("zrail.toml"), CONTRACT).expect("write contract");
}

fn package(name: &str, dependency: &str) -> String {
    let dependencies = if dependency.is_empty() {
        String::new()
    } else {
        format!("\n[dependencies]\n{dependency}\n")
    };
    format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n{dependencies}")
}

fn options(fixture: &Fixture) -> ReviewOptions {
    ReviewOptions {
        common: CommonOptions {
            root: fixture.proposal.clone(),
            config: "zrail.toml".into(),
            lock: "zrail.lock".into(),
            format: OutputFormat::Human,
        },
        authority_root: fixture.authority.clone(),
        base: "HEAD".into(),
        allow_grants: false,
    }
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

struct Fixture {
    base: PathBuf,
    authority: PathBuf,
    proposal: PathBuf,
}

const WORKSPACE: &str = r#"[workspace]
members = ["crates/app", "crates/zrail-core"]
resolver = "3"
"#;

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
facades = "allow"
tests = "allow"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#;
