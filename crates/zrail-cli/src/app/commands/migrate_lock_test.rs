//! Immutable-base migrations require a digest-bound review before lock updates.

use std::fs;

use zrail_core::{LOCK_SCHEMA, LOCK_SEMANTICS};

use crate::app::{
    args::{CommonOptions, MigrateLockOptions, UpdateOptions},
    output::OutputFormat,
};

use super::{
    super::git_base::{
        CONTRACT_PREFIX, CONTRACT_SUFFIX, commit_all, fixture_root, git_available, reset,
    },
    super::update::update,
    migrate_lock,
};

#[test]
fn immutable_base_migration_writes_a_digest_bound_scoped_report() {
    if !git_available() {
        return;
    }
    let root = fixture_root("lock-migration");
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.0.0'\nedition='2024'\n",
    )
    .expect("write manifest");
    fs::write(root.join("src/lib.rs"), "//! Fixture.\n").expect("write source");
    fs::write(
        root.join("zrail.toml"),
        format!("{CONTRACT_PREFIX}roots = [\".\"]\n{CONTRACT_SUFFIX}"),
    )
    .expect("write contract");
    let mut old = zrail_rust::build_lock(&root, "zrail.toml".as_ref()).expect("build old lock");
    old.schema = LOCK_SCHEMA - 1;
    old.semantics = LOCK_SEMANTICS - 1;
    old.analysis = None;
    old.write(&root.join("zrail.lock")).expect("write old lock");
    commit_all(&root);

    let output = migrate_lock(&MigrateLockOptions {
        root: root.clone(),
        config: "zrail.toml".into(),
        lock: "zrail.lock".into(),
        base: "HEAD".into(),
        output: "migration.json".into(),
    })
    .expect("migrate immutable base");

    let artifact = fs::read_to_string(root.join("migration.json")).expect("read report");
    assert!(artifact.contains("\"from_semantics\": 2"));
    assert!(artifact.contains("\"to_semantics\": 3"));
    assert!(artifact.contains("\"newly-observable\""));
    assert!(output.text.contains("--accept-migration sha256:"));
    let acceptance = output
        .text
        .lines()
        .find_map(|line| line.strip_prefix("accept with: --accept-migration "))
        .expect("acceptance identity")
        .to_owned();
    let mut update_options = UpdateOptions {
        common: CommonOptions {
            root: root.clone(),
            config: "zrail.toml".into(),
            lock: "zrail.lock".into(),
            format: OutputFormat::Human,
            ..CommonOptions::default()
        },
        base: "HEAD".into(),
        accept_grants: false,
        accept_migration: None,
    };
    let refused = update(&update_options).expect("refuse unaccepted migration");
    assert_eq!(refused.exit_code, 1);
    assert!(refused.text.contains("--accept-migration sha256:"));

    update_options.accept_migration = Some(acceptance);
    let accepted = update(&update_options).expect("accept reviewed migration");
    assert_eq!(accepted.exit_code, 0);
    let lock = zrail_core::LockFile::read(&root.join("zrail.lock")).expect("read migrated lock");
    assert_eq!(lock.semantics, LOCK_SEMANTICS);
    reset(&root);
}
