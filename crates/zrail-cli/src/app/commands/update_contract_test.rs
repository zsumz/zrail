//! Contract-only grants cannot authorize replacement of their own lock.

use std::{fs, path::PathBuf};

use zrail_rust::build_lock;

use crate::app::{
    args::{CommonOptions, UpdateOptions},
    commands::git_base::{commit_all, git_available},
    output::OutputFormat,
};

use super::update;

#[test]
fn policy_weakenings_require_explicit_acceptance() {
    if !git_available() {
        return;
    }
    for (name, before, after) in WEAKENINGS {
        let root = fixture_root(name);
        reset(&root);
        write_fixture(&root);
        let lock_path = root.join("zrail.lock");
        build_lock(&root, std::path::Path::new("zrail.toml"))
            .expect("build initial lock")
            .write(&lock_path)
            .expect("write initial lock");
        commit_all(&root);
        let initial = fs::read_to_string(&lock_path).expect("read initial lock");
        let contract = fs::read_to_string(root.join("zrail.toml")).expect("read contract");
        assert_eq!(contract.matches(before).count(), 1, "mutation {name}");
        fs::write(root.join("zrail.toml"), contract.replace(before, after))
            .expect("weaken contract");
        if *name == "owner" {
            write_second_owner(&root);
        }

        let mut options = options(&root);
        let refused = update(&options).expect("evaluate contract update");
        assert_eq!(refused.exit_code, 1, "mutation {name}");
        assert!(
            refused.text.contains("GRANT"),
            "mutation {name}: {}",
            refused.text
        );
        assert_eq!(
            fs::read_to_string(&lock_path).expect("read refused lock"),
            initial,
            "mutation {name}"
        );

        options.accept_grants = true;
        let accepted = update(&options).expect("accept contract update");
        assert_eq!(accepted.exit_code, 0, "mutation {name}");
        reset(&root);
    }
}

const WEAKENINGS: &[(&str, &str, &str)] = &[
    ("unsafe", "unsafe = \"deny\"", "unsafe = \"allow\""),
    (
        "module-docs",
        "module_docs = \"required\"",
        "module_docs = \"allow\"",
    ),
    (
        "facades",
        "facades = \"declarative\"",
        "facades = \"allow\"",
    ),
    (
        "entrypoints",
        "entrypoints = \"declarative\"",
        "entrypoints = \"allow\"",
    ),
    ("tests", "tests = \"sibling\"", "tests = \"allow\""),
    ("scope", SCOPE, ""),
    (
        "denied-symbol",
        "deny = [\"std::process\", \"std::net\"]",
        "deny = [\"std::net\"]",
    ),
    (
        "owner",
        "allow = [\"crates/fixture/src/io.rs\"]",
        "allow = [\"crates/fixture/src/io.rs\", \"crates/fixture/src/other.rs\"]",
    ),
    (
        "layer",
        "packages = [\"fixture\"]\nmay_depend_on = []",
        "packages = [\"fixture\"]\nmay_depend_on = [\"base\"]",
    ),
    (
        "effect",
        "deny = [\"network\", \"process\"]",
        "deny = [\"process\"]",
    ),
];

const SCOPE: &str = concat!(
    "[[scope]]\n",
    "name = \"no-process\"\n",
    "include = [\"crates/fixture/src/**\"]\n",
    "reason = \"processes belong elsewhere\"\n\n",
    "[scope.symbols]\n",
    "deny = [\"std::process\", \"std::net\"]\n\n",
);

fn options(root: &std::path::Path) -> UpdateOptions {
    UpdateOptions {
        common: CommonOptions {
            root: root.to_path_buf(),
            config: PathBuf::from("zrail.toml"),
            lock: PathBuf::from("zrail.lock"),
            format: OutputFormat::Human,
            ..CommonOptions::default()
        },
        base: "HEAD".into(),
        accept_grants: false,
        accept_migration: None,
    }
}

fn write_fixture(root: &std::path::Path) {
    fs::create_dir_all(root.join("crates/fixture/src")).expect("create fixture package");
    fs::create_dir_all(root.join("crates/base/src")).expect("create base package");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/base\", \"crates/fixture\"]\nresolver = \"3\"\n",
    )
    .expect("write workspace");
    write_package(root, "base");
    write_package(root, "fixture");
    fs::write(root.join("crates/base/src/lib.rs"), "//! base\n").expect("write base source");
    fs::write(
        root.join("crates/fixture/src/lib.rs"),
        "//! fixture\nmod io;\nmod other;\n",
    )
    .expect("write fixture facade");
    fs::write(
        root.join("crates/fixture/src/io.rs"),
        "//! fixture\nuse std::fs;\npub fn metadata() { let _ = fs::metadata(\".\"); }\n",
    )
    .expect("write owned source");
    fs::write(root.join("crates/fixture/src/other.rs"), "//! other\n").expect("write owner target");
    fs::write(root.join("zrail.toml"), contract()).expect("write contract");
}

fn write_second_owner(root: &std::path::Path) {
    fs::write(
        root.join("crates/fixture/src/other.rs"),
        "//! other\nuse std::fs;\npub fn exists() -> bool { fs::exists(\".\").unwrap_or(false) }\n",
    )
    .expect("write second owner use");
}

fn write_package(root: &std::path::Path, name: &str) {
    fs::write(
        root.join(format!("crates/{name}/Cargo.toml")),
        format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n"),
    )
    .expect("write package");
}

fn contract() -> String {
    format!(
        concat!(
            "schema = 1\nadapters = [\"rust\"]\n\n",
            "[repository]\nroots = [\"crates\"]\nexclude = []\n",
            "workspace_members = \"exact\"\nnested_git = \"deny\"\n",
            "submodules = \"deny\"\nsymlinks = \"inside\"\n\n",
            "[dependencies]\nmode = \"locked\"\nunassigned_packages = \"deny\"\n",
            "cycles = \"deny\"\n\n",
            "[source.rust]\nmodule_docs = \"required\"\nfacades = \"declarative\"\n",
            "entrypoints = \"declarative\"\ntests = \"sibling\"\n\n",
            "[source.rust.hygiene]\nunsafe = \"deny\"\n",
            "lint_suppressions = \"deny\"\ndeny_methods = []\ndeny_macros = []\n\n",
            "[source.rust.size.facade]\ntarget = 300\nhard = 300\n\n",
            "[source.rust.size.implementation]\ntarget = 300\nhard = 300\n\n",
            "[source.rust.size.test]\ntarget = 300\nhard = 300\n\n",
            "[source.rust.size.auxiliary]\ntarget = 300\nhard = 300\n\n",
            "[profiles.restricted.effects]\ndeny = [\"network\", \"process\"]\n\n",
            "[[layer]]\nname = \"base\"\npackages = [\"base\"]\n",
            "may_depend_on = []\nreason = \"foundation\"\n\n",
            "[layer.dependencies]\nexternal = \"none\"\n\n",
            "[[layer]]\nname = \"app\"\npackages = [\"fixture\"]\n",
            "may_depend_on = []\nprofiles = [\"restricted\"]\nreason = \"application\"\n\n",
            "[layer.dependencies]\nexternal = \"locked\"\n\n",
            "{SCOPE}",
            "[[owner]]\nname = \"filesystem\"\nkind = \"capability\"\n",
            "within = [\"crates/fixture/src/**\"]\nmatch = \"std::fs\"\n",
            "allow = [\"crates/fixture/src/io.rs\"]\nreason = \"one filesystem owner\"\n",
        ),
        SCOPE = SCOPE
    )
}

fn fixture_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zrail-update-contract-{name}-{}",
        std::process::id()
    ))
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
