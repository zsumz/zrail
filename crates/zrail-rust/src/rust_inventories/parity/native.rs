//! Trusted syntax fixtures invoke the stock physical selection and parsed-fact pipeline.

use std::{collections::BTreeMap, fs, path::Path};

use crate::inventory::{RepositoryInventory, RustSourceFile, classify_path};

pub(super) fn observed(root: &Path, source: &str) -> BTreeMap<String, usize> {
    assert!(!root.exists(), "never overwrite a qualification directory");
    fs::create_dir_all(root.join("src")).expect("fixture directory");
    fs::write(root.join("src/sample.rs"), source).expect("fixture source");
    let policy = toml::from_str("name='methods'\nreason='Frozen quantity parity.'\ninclude=['src/**/*.rs']\nworld='authored'\nsubject={kind='written-methods',names=['turn_component','poll_io','wake_handle','pulse_handle']}\nassertion={kind='count',maximum=0}").expect("native policy");
    let sources = BTreeMap::from([("src/sample.rs".into(), source.into())]);
    let analyzed = analyze(root, &sources, &policy);
    fs::remove_dir_all(root).expect("remove test-owned source");
    analyzed
        .expect("complete syntax inventory")
        .counts
        .iter()
        .map(|count| (format!("{}:{}", count.path, count.name), count.count))
        .collect()
}

pub(super) fn analyze(
    root: &Path,
    sources: &BTreeMap<String, String>,
    policy: &zrail_core::RustInventoryRule,
) -> Result<crate::GovernedRustInventory, String> {
    let inventory = RepositoryInventory {
        root: root.to_path_buf(),
        entries: vec![],
        manifest_paths: vec![],
        rust_files: sources
            .iter()
            .map(|(path, source)| RustSourceFile {
                relative: path.clone(),
                class: classify_path(path, &[]),
                source: source.clone(),
                lines: source.lines().count(),
            })
            .collect(),
    };
    let mut rust: zrail_core::RustSourceContract = toml::from_str("module_docs='required'\nfacades='declarative'\ntests='sibling'\n[hygiene]\nunsafe='deny'\nlint_suppressions='deny'\n").expect("source contract");
    rust.inventories = vec![policy.clone()];
    let facts = crate::source::index_rust_source(&inventory, &rust);
    crate::rust_inventories::analyze(&inventory, &facts, &rust.inventories)
        .map(|mut analysis| analysis.policies.remove(0))
}
