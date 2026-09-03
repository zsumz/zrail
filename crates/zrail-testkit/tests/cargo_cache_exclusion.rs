//! Exact Cargo cache exclusions accept missing tags and reject forged tags.

use std::{fs, path::Path};

use zrail_rust::check_repository;

const CACHE_TAG: &str = "Signature: 8a477f597d28d172789f06886806bc55\n";

#[test]
fn exact_package_target_exclusion_is_valid_before_the_cache_exists() {
    let root = repository("prospective", "crates/member/target/**");
    assert_no_rep_006(&root);
    reset(&root);

    let workspace = repository("workspace", "target/**");
    assert_no_rep_006(&workspace);
    reset(&workspace);
}

#[test]
fn existing_package_target_accepts_a_missing_or_standard_cache_tag() {
    let root = repository("tagged", "crates/member/target/**");
    let target = root.join("crates/member/target");
    fs::create_dir_all(&target).expect("create target cache");
    fs::write(target.join("artifact"), "opaque").expect("write artifact");
    assert_no_rep_006(&root);

    fs::write(target.join("CACHEDIR.TAG"), CACHE_TAG).expect("write cache tag");
    assert_no_rep_006(&root);

    fs::write(target.join("CACHEDIR.TAG"), "not a cache tag\n").expect("forge cache tag");
    assert_rep_006(&root);

    fs::remove_file(target.join("CACHEDIR.TAG")).expect("remove forged tag");
    fs::create_dir(target.join("CACHEDIR.TAG")).expect("create non-file tag");
    assert_rep_006(&root);
    reset(&root);
}

#[test]
fn wildcard_target_patterns_remain_rejected() {
    let root = repository("untagged", "crates/member/target/**");
    let target = root.join("crates/member/target");
    fs::create_dir_all(&target).expect("create target cache");
    fs::write(target.join("artifact"), "opaque").expect("write artifact");
    assert_no_rep_006(&root);
    reset(&root);

    let wildcard = repository("wildcard", "crates/*/target/**");
    assert_rep_006(&wildcard);
    reset(&wildcard);
}

fn repository(name: &str, exclusion: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-cargo-cache-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("crates/member/src")).expect("create fixture");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\"]\nresolver = \"3\"\n",
    )
    .expect("write workspace");
    fs::write(
        root.join("crates/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write manifest");
    fs::write(root.join("crates/member/src/lib.rs"), "//! Member.\n").expect("write source");
    fs::write(root.join("zrail.toml"), contract(exclusion)).expect("write contract");
    root
}

fn contract(exclusion: &str) -> String {
    format!(
        r#"schema = 1
adapters = ["rust"]
[repository]
roots = ["crates"]
exclude = ["{exclusion}"]
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
[source.rust.macros]
mode = "allow"
[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
"#
    )
}

fn assert_no_rep_006(root: &Path) {
    let report = check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check fixture")
        .report;
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.id != "REP-006"),
        "{}",
        report.human()
    );
}

fn assert_rep_006(root: &Path) {
    let report = check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check fixture")
        .report;
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "REP-006"),
        "{}",
        report.human()
    );
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
