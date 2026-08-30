//! Repository implementation claims require exact external dependency selection.

use super::{build_lock, check_repository, fixture, reset, write};

#[test]
fn only_external_implementation_closures_require_a_lock() {
    let root = fixture("local-resolution", "");
    assert!(!root.join("Cargo.lock").exists());
    build_lock(&root, "zrail.toml".as_ref()).unwrap();
    external_manifest(&root, "external = \"1\"");
    let error = build_lock(&root, "zrail.toml".as_ref())
        .unwrap_err()
        .to_string();
    assert!(error.contains("requires Cargo.lock"), "{error}");
    reset(&root);
}

#[test]
fn stale_missing_and_ambiguous_outgoing_edges_are_rejected() {
    for (name, requirement, edges, extra) in [
        ("stale", "2", "\"external\"", String::new()),
        ("missing", "1", "", String::new()),
        (
            "ambiguous",
            "1",
            "\"external 1.2.3\", \"external 1.9.0\"",
            registry("1.9.0", 'b'),
        ),
    ] {
        let root = fixture(&format!("resolution-{name}"), "");
        external_manifest(&root, &format!("external = {requirement:?}"));
        write(
            &root,
            "Cargo.lock",
            &format!("{}{}{}", graph(edges), registry("1.2.3", 'a'), extra),
        );
        let error = build_lock(&root, "zrail.toml".as_ref())
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("outgoing Cargo.lock") || error.contains("maps ambiguously"),
            "{error}"
        );
        reset(&root);
    }
}

#[test]
fn registry_versions_checksums_and_transitive_graph_changes_invalidate_authority() {
    for (name, changed) in [
        ("version", registry("1.3.0", 'a')),
        ("checksum", registry("1.2.3", 'b')),
        (
            "transitive",
            format!(
                "{}dependencies = [\"indirect\"]\n{}",
                registry("1.2.3", 'a'),
                registry("2.0.0", 'c').replace("name = \"external\"", "name = \"indirect\"")
            ),
        ),
    ] {
        let root = fixture(&format!("resolution-{name}"), "");
        external_manifest(&root, "external = \"1\"");
        write(
            &root,
            "Cargo.lock",
            &format!("{}{}", graph("\"external\""), registry("1.2.3", 'a')),
        );
        let lock = build_lock(&root, "zrail.toml".as_ref()).unwrap();
        lock.write(&root.join("zrail.lock")).unwrap();
        write(
            &root,
            "Cargo.lock",
            &format!("{}{changed}", graph("\"external\"")),
        );
        let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
            .unwrap()
            .report;
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.id == "LOCK-023"),
            "{}",
            report.human()
        );
        reset(&root);
    }
}

#[test]
fn git_resolution_binds_exact_revisions_and_rejects_wrong_manifest_references() {
    let root = fixture("git-resolution", "");
    external_manifest(
        &root,
        "external = { git = \"https://example.test/macros.git\", branch = \"main\" }",
    );
    let git = |branch: &str, revision: char| {
        format!(
            "[[package]]\nname = \"external\"\nversion = \"1.2.3\"\nsource = \"git+https://example.test/macros.git?branch={branch}#{}\"\n",
            revision.to_string().repeat(40),
        )
    };
    write(
        &root,
        "Cargo.lock",
        &format!("{}{}", graph("\"external\""), git("main", 'a')),
    );
    let before = build_lock(&root, "zrail.toml".as_ref()).unwrap();
    before.write(&root.join("zrail.lock")).unwrap();
    write(
        &root,
        "Cargo.lock",
        &format!("{}{}", graph("\"external\""), git("main", 'b')),
    );
    let after = build_lock(&root, "zrail.toml".as_ref()).unwrap();
    assert_ne!(before.macro_implementations, after.macro_implementations);
    write(
        &root,
        "Cargo.lock",
        &format!("{}{}", graph("\"external\""), git("other", 'a')),
    );
    let error = build_lock(&root, "zrail.toml".as_ref())
        .unwrap_err()
        .to_string();
    assert!(error.contains("outgoing Cargo.lock"), "{error}");
    reset(&root);
}

fn external_manifest(root: &std::path::Path, dependency: &str) {
    write(
        root,
        "helper/Cargo.toml",
        &format!(
            "[package]\nname = \"helper\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\n{dependency}\n",
        ),
    );
}

fn graph(helper_edges: &str) -> String {
    format!(
        "version = 4\n[[package]]\nname = \"app\"\nversion = \"0.0.0\"\ndependencies = [\"reviewed\"]\n\
         [[package]]\nname = \"reviewed\"\nversion = \"0.0.0\"\ndependencies = [\"helper\"]\n\
         [[package]]\nname = \"helper\"\nversion = \"0.0.0\"\ndependencies = [{helper_edges}]\n",
    )
}

fn registry(version: &str, checksum: char) -> String {
    format!(
        "[[package]]\nname = \"external\"\nversion = {version:?}\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = {:?}\n",
        checksum.to_string().repeat(64),
    )
}
