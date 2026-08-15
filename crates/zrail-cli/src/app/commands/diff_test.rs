//! Diff rejects locks that do not belong to their loaded architecture state.

use std::{fs, path::PathBuf};

use zrail_core::{LockFile, load_contract};

use crate::app::{
    args::{DiffMode, DiffOptions},
    output::OutputFormat,
};

use super::diff;

#[test]
fn deny_grants_rejects_stale_contract_digest() {
    let (before, after) = fixture("stale");
    write_lock(&before, |lock| lock.contract_sha256 = "0".repeat(64));
    write_lock(&after, |_| {});

    let output = diff(&options(&before, &after)).expect("compare stale lock");

    assert_eq!(output.exit_code, 1);
    assert!(
        output
            .text
            .contains("UNKNOWN lock.authority before:contract")
    );
    reset(&before);
    reset(&after);
}

#[test]
fn deny_grants_rejects_incompatible_semantics() {
    let (before, after) = fixture("engine");
    write_lock(&before, |_| {});
    write_lock(&after, |lock| lock.semantics = 999);

    let output = diff(&options(&before, &after)).expect("compare incompatible lock");

    assert_eq!(output.exit_code, 1);
    assert!(
        output
            .text
            .contains("UNKNOWN lock.authority after:semantics")
    );
    reset(&before);
    reset(&after);
}

fn fixture(name: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir().join(format!("zrail-diff-{name}-{}", std::process::id()));
    let before = base.with_extension("before");
    let after = base.with_extension("after");
    for root in [&before, &after] {
        reset(root);
        fs::create_dir_all(root).expect("create fixture");
        fs::write(root.join("zrail.toml"), CONTRACT).expect("write contract");
    }
    (before, after)
}

fn write_lock(root: &std::path::Path, mutate: impl FnOnce(&mut LockFile)) {
    let digest = load_contract(root, std::path::Path::new("zrail.toml"))
        .expect("load contract")
        .sha256;
    let mut lock = LockFile::new(digest);
    mutate(&mut lock);
    lock.write(&root.join("zrail.lock")).expect("write lock");
}

fn options(before: &std::path::Path, after: &std::path::Path) -> DiffOptions {
    DiffOptions {
        mode: DiffMode::Explicit {
            before: before.to_path_buf(),
            after: after.to_path_buf(),
        },
        config: PathBuf::from("zrail.toml"),
        lock: PathBuf::from("zrail.lock"),
        format: OutputFormat::Human,
        deny_grants: true,
    }
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const CONTRACT: &str = concat!(
    "schema = 1\nadapters = [\"rust\"]\n\n",
    "[repository]\nroots = [\".\"]\nexclude = []\n",
    "workspace_members = \"exact\"\nnested_git = \"deny\"\n",
    "submodules = \"deny\"\nsymlinks = \"inside\"\n\n",
    "[dependencies]\nmode = \"locked\"\nunassigned_packages = \"allow\"\n",
    "cycles = \"deny\"\n\n",
    "[source.rust]\nmodule_docs = \"allow\"\nfacades = \"allow\"\n",
    "tests = \"allow\"\n\n",
    "[source.rust.hygiene]\nunsafe = \"allow\"\nlint_suppressions = \"allow\"\n",
    "deny_methods = []\ndeny_macros = []\n\n",
    "[source.rust.size.facade]\ntarget = 300\nhard = 300\n\n",
    "[source.rust.size.implementation]\ntarget = 300\nhard = 300\n\n",
    "[source.rust.size.test]\ntarget = 300\nhard = 300\n\n",
    "[source.rust.size.auxiliary]\ntarget = 300\nhard = 300\n",
);
