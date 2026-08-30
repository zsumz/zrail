//! Provider capture cannot be derailed by unrelated or reserved excluded subtrees.

use std::fs;

use super::{build_lock, fixture, reset, write};

#[test]
fn unrelated_and_reserved_trees_are_pruned_before_traversal_limits() {
    let root = fixture("targeted-scan", "");
    let contract = fs::read_to_string(root.join("zrail.toml")).unwrap().replace(
        "exclude = [\"macros/templates/**\"]",
        "exclude = [\"macros/templates/**\", \"node_modules/**\", \"macros/target/**\", \"helper/.zrail/**\"]",
    );
    write(&root, "zrail.toml", &contract);
    let before = build_lock(&root, "zrail.toml".as_ref()).unwrap();
    for prefix in ["node_modules", "macros/target", "helper/.zrail"] {
        let mut path = root.join(prefix);
        for _ in 0..140 {
            path.push("n");
        }
        fs::create_dir_all(path).unwrap();
    }
    let after = build_lock(&root, "zrail.toml".as_ref()).unwrap();
    assert_eq!(before.macro_implementations, after.macro_implementations);
    reset(&root);
}

#[test]
fn explicitly_selected_source_excluded_assets_remain_bound() {
    let root = fixture("excluded-explicit-scan", "inputs = [\"schemas/**\"]");
    let contract = fs::read_to_string(root.join("zrail.toml"))
        .unwrap()
        .replace(
            "exclude = [\"macros/templates/**\"]",
            "exclude = [\"macros/templates/**\", \"schemas/**\"]",
        );
    write(&root, "zrail.toml", &contract);
    let before = build_lock(&root, "zrail.toml".as_ref()).unwrap();
    write(&root, "schemas/api.json", "changed selected data");
    let after = build_lock(&root, "zrail.toml".as_ref()).unwrap();
    assert_ne!(before.macro_implementations, after.macro_implementations);
    reset(&root);
}

#[cfg(unix)]
#[test]
fn fixed_prefixes_cannot_traverse_internal_symlink_aliases() {
    let root = fixture("symlink-prefix-scan", "inputs = [\"alias/api.json\"]");
    std::os::unix::fs::symlink("schemas", root.join("alias")).unwrap();
    let error = build_lock(&root, "zrail.toml".as_ref())
        .unwrap_err()
        .to_string();
    assert!(error.contains("symlink"), "{error}");
    reset(&root);
}
