//! Focused examples for repository paths and glob semantics.

use std::{fs, path::Path};

use super::{
    MAX_GLOB_PATTERN_BYTES, MAX_GLOB_PATTERN_SEGMENTS, glob_matches, normalize_relative,
    repository_file, repository_relative,
};

#[test]
fn recursive_globs_cross_directory_boundaries() {
    assert!(glob_matches("crates/**/src/*.rs", "crates/a/b/src/lib.rs"));
    assert!(!glob_matches("crates/*/src/*.rs", "crates/a/b/src/lib.rs"));
}

#[test]
fn component_wildcards_do_not_cross_slashes() {
    assert!(glob_matches("crates/zrail-*", "crates/zrail-core"));
    assert!(!glob_matches("crates/zrail-*", "crates/zrail-core/src"));
}

#[test]
fn repeated_recursive_globs_are_bounded_and_collapsed() {
    let repeated = std::iter::repeat_n("**", MAX_GLOB_PATTERN_SEGMENTS).collect::<Vec<_>>();
    assert!(glob_matches(&repeated.join("/"), "a/b/c"));

    let adversarial = std::iter::repeat_n("**/x", MAX_GLOB_PATTERN_SEGMENTS)
        .collect::<Vec<_>>()
        .join("/");
    assert!(!glob_matches(&adversarial, "a/b/c"));
    assert!(!glob_matches(&"x".repeat(MAX_GLOB_PATTERN_BYTES + 1), "x"));
}

#[test]
fn escaping_paths_are_refused() {
    assert!(normalize_relative(Path::new("../secret")).is_err());
    assert_eq!(
        normalize_relative(Path::new("./crates/core")).as_deref(),
        Ok("crates/core")
    );
    assert!(normalize_relative(Path::new("crates\\core")).is_err());
}

#[test]
fn filesystem_paths_have_one_portable_repository_spelling() {
    let root = Path::new("/repository");
    assert_eq!(
        repository_relative(root, Path::new("/repository/crates/core")).as_deref(),
        Ok("crates/core")
    );
    assert!(repository_relative(root, Path::new("/outside/core")).is_err());
}

#[cfg(unix)]
#[test]
fn repository_paths_reject_non_utf8_and_backslash_components() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    let root = Path::new("/repository");
    assert!(repository_relative(root, Path::new("/repository/a\\b")).is_err());
    let invalid = Path::new("/repository").join(OsStr::from_bytes(b"bad-\xff"));
    assert!(repository_relative(root, &invalid).is_err());
}

#[test]
fn repository_files_must_be_relative_and_parented_inside_the_root() {
    let root = std::env::temp_dir().join(format!("zrail-path-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("reset fixture");
    }
    fs::create_dir_all(root.join("architecture")).expect("create fixture");

    assert_eq!(
        repository_file(&root, Path::new("architecture/zrail.lock")).expect("resolve lock"),
        fs::canonicalize(&root)
            .expect("canonical root")
            .join("architecture/zrail.lock")
    );
    assert!(repository_file(&root, &root.join("zrail.lock")).is_err());
    assert!(repository_file(&root, Path::new("../zrail.lock")).is_err());
    fs::remove_dir_all(root).expect("remove fixture");
}

#[cfg(unix)]
#[test]
fn repository_files_reject_symlinked_parent_escapes() {
    use std::os::unix::fs::symlink;

    let root = std::env::temp_dir().join(format!("zrail-parent-{}", std::process::id()));
    let outside = std::env::temp_dir().join(format!("zrail-outside-{}", std::process::id()));
    for path in [&root, &outside] {
        if path.exists() {
            fs::remove_dir_all(path).expect("reset fixture");
        }
        fs::create_dir_all(path).expect("create fixture");
    }
    symlink(&outside, root.join("architecture")).expect("create parent alias");

    assert!(repository_file(&root, Path::new("architecture/zrail.lock")).is_err());
    fs::remove_dir_all(root).expect("remove root");
    fs::remove_dir_all(outside).expect("remove outside");
}
