//! Generated lock state covers every byte of the provenance manifest.

use std::{fs, path::PathBuf};

use zrail_core::GeneratedSourceContract;

use super::locked_sources;

#[test]
fn project_specific_provenance_changes_the_lock_digest() {
    let root = fixture_root();
    fs::create_dir_all(root.join("generated")).expect("create fixture");
    let manifest = root.join("generated/MANIFEST.json");
    fs::write(&manifest, r#"{"schema":1,"upstream_commit":"one"}"#).expect("write first manifest");
    let first = locked_sources(&root, &[contract()]);
    fs::write(&manifest, r#"{"schema":1,"upstream_commit":"two"}"#).expect("write second manifest");
    let second = locked_sources(&root, &[contract()]);

    assert_ne!(first, second);
    fs::remove_dir_all(root).expect("remove fixture");
}

fn contract() -> GeneratedSourceContract {
    GeneratedSourceContract {
        root: "generated".into(),
        manifest: "generated/MANIFEST.json".into(),
        inputs: vec!["schema/**".into()],
        target: 300,
        hard: 300,
        reason: "fixture".into(),
        auxiliary: Vec::new(),
    }
}

fn fixture_root() -> PathBuf {
    std::env::temp_dir().join(format!("zrail-generated-lock-{}", std::process::id()))
}
