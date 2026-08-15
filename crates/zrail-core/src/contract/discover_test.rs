//! Wildcard discovery traverses only safe fixed prefixes and actual root metadata.

use std::{fs, path::PathBuf};

use super::{fixed_prefix, normalize_import, toml_files};

#[test]
fn patterns_have_one_normalized_fixed_traversal_root() {
    let narrow = normalize_import("./architecture/crates/*.toml").expect("normalize pattern");
    assert_eq!(fixed_prefix(&narrow), "architecture/crates");
    assert_eq!(fixed_prefix("**/*.toml"), "");
    assert!(normalize_import("../architecture/*.toml").is_err());
    assert!(normalize_import("architecture\\*.toml").is_err());
}

#[test]
fn narrow_prefix_ignores_unrelated_deep_trees() {
    let root = fixture_root("narrow");
    reset(&root);
    fs::create_dir_all(root.join("architecture/crates")).expect("create contract directory");
    fs::write(root.join("architecture/crates/core.toml"), "schema = 1\n").expect("write contract");
    fs::create_dir_all(root.join("node_modules")).expect("create unrelated tree");
    for index in 0..128 {
        fs::write(
            root.join(format!("node_modules/{index}.toml")),
            "unrelated = true\n",
        )
        .expect("write unrelated file");
    }

    let mut inspected = 0;
    let files =
        toml_files(&root, "architecture/crates", &mut inspected).expect("discover narrow prefix");

    assert_eq!(files, [root.join("architecture/crates/core.toml")]);
    assert_eq!(inspected, 1);
    reset(&root);
}

#[test]
fn nested_target_directories_are_visible_but_root_build_output_is_not() {
    let root = fixture_root("target");
    reset(&root);
    fs::create_dir_all(root.join("architecture/target")).expect("create nested target");
    fs::create_dir_all(root.join("target")).expect("create root target");
    fs::write(root.join("architecture/target/rules.toml"), "schema = 1\n")
        .expect("write nested contract");
    fs::write(root.join("target/ignored.toml"), "schema = 1\n").expect("write build output");

    let mut inspected = 0;
    let files = toml_files(&root, "", &mut inspected).expect("discover repository");

    assert!(files.contains(&root.join("architecture/target/rules.toml")));
    assert!(!files.contains(&root.join("target/ignored.toml")));
    reset(&root);
}

fn fixture_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zrail-contract-discovery-{name}-{}",
        std::process::id()
    ))
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
    fs::create_dir_all(root).expect("create fixture");
}
