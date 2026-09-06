//! Trusted syntax fixtures invoke the stock physical selection and parsed-fact pipeline.

use std::{collections::BTreeMap, fs, path::Path};

use crate::inventory::{FileClass, RepositoryInventory, RustSourceFile};

pub(super) fn observed(root: &Path, source: &str) -> BTreeMap<String, usize> {
    assert!(!root.exists(), "never overwrite a qualification directory");
    fs::create_dir_all(root.join("src")).expect("fixture directory");
    fs::write(root.join("src/sample.rs"), source).expect("fixture source");
    let inventory = RepositoryInventory {
        root: root.to_path_buf(),
        entries: vec![],
        manifest_paths: vec![],
        rust_files: vec![RustSourceFile {
            relative: "src/sample.rs".into(),
            class: FileClass::Implementation,
            source: source.into(),
            lines: source.lines().count(),
        }],
    };
    let mut rust: zrail_core::RustSourceContract = toml::from_str("module_docs='required'\nfacades='declarative'\ntests='sibling'\n[hygiene]\nunsafe='deny'\nlint_suppressions='deny'\n").expect("source contract");
    let policy = toml::from_str("name='methods'\nreason='Frozen quantity parity.'\ninclude=['src/**/*.rs']\nworld='authored'\nsubject={kind='written-methods',names=['turn_component','poll_io','wake_handle','pulse_handle']}\nassertion={kind='count',maximum=0}").expect("native policy");
    rust.inventories = vec![policy];
    let facts = crate::source::index_rust_source(&inventory, &rust);
    let analyzed = crate::rust_inventories::analyze(&inventory, &facts, &rust.inventories);
    fs::remove_dir_all(root).expect("remove test-owned source");
    analyzed.expect("complete syntax inventory").policies[0]
        .counts
        .iter()
        .map(|count| (format!("{}:{}", count.path, count.name), count.count))
        .collect()
}
