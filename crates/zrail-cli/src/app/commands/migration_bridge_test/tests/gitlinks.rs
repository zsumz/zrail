//! Raw gitlinks cannot evade submodule policy through lossy snapshot materialization.

use std::{ffi::OsStr, fs, path::Path};

use super::{
    bridge_fixture_root, commit_all, commit_index, git_available, head_revision, migrate_lock,
    migration_options, reset, run_git,
};
use crate::app::commands::git_base::GitSnapshot;

#[test]
fn deny_rejects_raw_gitlinks_even_without_gitmodules_or_governed_contents() {
    if !git_available() {
        return;
    }
    for governed in [".", "src"] {
        let (root, base) =
            bridge_fixture_root(&format!("gitlink-denied-{governed}"), false, governed);
        add_gitlink(&root, &base);
        assert!(!root.join(".gitmodules").exists());
        let error = migrate_lock(&migration_options(&root, &base, Some("HEAD")))
            .expect_err("deny the actual target Git tree");
        assert!(
            error.message.contains("repository.submodules"),
            "{}",
            error.message
        );
        assert!(error.message.contains("vendor/assets"));
        assert!(error.message.contains("160000"));
        assert!(!root.join("migration.json").exists());
        let error = migrate_lock(&migration_options(&root, "HEAD", None))
            .expect_err("same-revision migration must also preserve Git object kinds");
        assert!(error.message.contains("repository.submodules"));
        reset(&root);
    }
}

#[test]
fn allowed_gitlinks_are_opaque_and_bind_the_exact_object_identity() {
    if !git_available() {
        return;
    }
    let (root, base) = bridge_fixture_root("gitlink-allowed", false, "src");
    let contract = fs::read_to_string(root.join("zrail.toml")).unwrap();
    fs::write(
        root.join("zrail.toml"),
        contract.replace("submodules = \"deny\"", "submodules = \"allow\""),
    )
    .unwrap();
    commit_all(&root);
    let other = head_revision(&root);
    for object in [&base, &other] {
        add_gitlink(&root, object);
        let snapshot = GitSnapshot::create_repository(&root, OsStr::new("HEAD")).unwrap();
        assert!(snapshot.tree["vendor/assets"].is_gitlink());
        assert_eq!(&snapshot.tree["vendor/assets"].object, object);
        assert!(!snapshot.root().join("vendor/assets").exists());
        migrate_lock(&migration_options(&root, &base, Some("HEAD"))).unwrap();
        let report = fs::read_to_string(root.join("migration.json")).unwrap();
        assert!(report.contains("\"mode\": \"160000\""));
        assert!(report.contains(&zrail_core::sha256_hex(
            format!("gitlink\0{object}").as_bytes()
        )));
        fs::remove_file(root.join("migration.json")).unwrap();
    }
    reset(&root);
}

fn add_gitlink(root: &Path, object: &str) {
    run_git(
        root,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            object,
            "vendor/assets",
        ],
    );
    commit_index(root);
}
