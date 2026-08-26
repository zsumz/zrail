//! Shared mirror inputs count once while retaining per-mirror digest authority.

use std::{collections::BTreeMap, fs};

use zrail_core::{TestExecutionIdentity, TestMirrorContract};

use crate::inventory::{RepositoryEntry, RepositoryEntryKind};

use super::MirrorInputCache;

#[test]
fn one_thousand_two_hundred_forty_four_mirrors_share_input_bytes() {
    let root =
        std::env::temp_dir().join(format!("zrail-mirror-input-cache-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale fixture");
    }
    fs::create_dir(&root).expect("create fixture");
    let paths = ["production.rs", "test.rs", "Cargo.lock"];
    for path in paths {
        fs::write(root.join(path), path).expect("write input");
    }
    let owned = paths
        .into_iter()
        .map(|relative| RepositoryEntry {
            relative: relative.into(),
            absolute: root.join(relative),
            kind: RepositoryEntryKind::File,
        })
        .collect::<Vec<_>>();
    let entries = owned
        .iter()
        .map(|entry| (entry.relative.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut cache = MirrorInputCache::new(&entries);
    let mut mirror = mirror();

    let first = cache.digest(&mirror).expect("first digest");
    let charged = cache.aggregate_bytes();
    let mut last = first.clone();
    for index in 0..1_244 {
        mirror.name = format!("exact_test_{index:04}");
        last = cache.digest(&mirror).expect("bulk digest");
    }

    assert_ne!(first, last);
    assert_eq!(cache.aggregate_bytes(), charged);
    assert_eq!(cache.loaded_inputs(), 3);
    fs::remove_dir_all(root).expect("remove fixture");
}

fn mirror() -> TestMirrorContract {
    TestMirrorContract {
        production: "production.rs".into(),
        test: "test.rs".into(),
        name: "exact_test".into(),
        receipt: "receipt.json".into(),
        inputs: vec!["Cargo.lock".into()],
        execution: TestExecutionIdentity {
            command: "cargo test exact_test -- --exact".into(),
            package: "fixture".into(),
            default_features: true,
            features: Vec::new(),
            target: "x86_64-unknown-linux-gnu".into(),
            toolchain: "rustc 1.96.0".into(),
        },
        reason: "Exact behavior".into(),
    }
}
