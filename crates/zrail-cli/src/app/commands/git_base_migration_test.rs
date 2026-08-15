//! Git-base diffs keep generated-provenance migrations reviewable.

use std::fs;

use zrail_core::{LockFile, LockedGeneratedSource, load_contract};

use crate::app::{
    args::{DiffMode, DiffOptions},
    output::OutputFormat,
};

use super::git_base::{
    CONTRACT_PREFIX, CONTRACT_SUFFIX, commit_all, fixture_root, git_available, reset,
};

#[test]
fn base_diff_accepts_a_legacy_generated_provenance_state() {
    if !git_available() {
        return;
    }
    let root = fixture_root("generated-migration");
    reset(&root);
    fs::create_dir_all(&root).expect("create fixture");
    fs::write(root.join("zrail.toml"), generated_contract("")).expect("write legacy contract");
    write_lock(&root, false);
    commit_all(&root);
    fs::write(
        root.join("zrail.toml"),
        generated_contract("inputs = [\"schema/**\"]\n"),
    )
    .expect("write migrated contract");
    write_lock(&root, true);

    let output = crate::app::commands::diff(&DiffOptions {
        mode: DiffMode::Base {
            root: root.clone(),
            revision: "HEAD".into(),
        },
        config: "zrail.toml".into(),
        lock: "zrail.lock".into(),
        format: OutputFormat::Human,
        deny_grants: true,
    })
    .expect("compare provenance migration");

    assert_eq!(output.exit_code, 0);
    assert!(
        output
            .text
            .contains("REVOKE rust.generated-source.input src/generated:schema/**")
    );
    assert!(
        output
            .text
            .contains("REVOKE rust.generated-provenance src/generated")
    );
    assert!(
        output
            .text
            .contains("Changes: 0 grants, 2 revokes, 0 debt, 0 cleanup, 0 unknown")
    );
    reset(&root);
}

fn generated_contract(inputs: &str) -> String {
    format!(
        concat!(
            "{}roots = [\".\"]\n{}\n",
            "[[source.rust.generated]]\n",
            "root = \"src/generated\"\n",
            "manifest = \"src/generated/MANIFEST.json\"\n",
            "{}",
            "target = 1000\n",
            "hard = 2000\n",
            "reason = \"fixture compiler owns this tree\"\n",
        ),
        CONTRACT_PREFIX, CONTRACT_SUFFIX, inputs
    )
}

fn write_lock(root: &std::path::Path, protected: bool) {
    let digest = load_contract(root, std::path::Path::new("zrail.toml"))
        .expect("load fixture contract")
        .sha256;
    let mut lock = LockFile::new(digest);
    if protected {
        lock.generated.push(LockedGeneratedSource {
            root: "src/generated".into(),
            manifest_sha256: "1".repeat(64),
        });
    }
    lock.write(&root.join("zrail.lock")).expect("write lock");
}
