//! Regular-file reads and atomic replacements reject architecture aliases.

use std::{fs, path::PathBuf};

use super::{create_text, read_bytes_with_limit, read_text, read_text_with_limit, replace_text};

#[test]
fn bounded_byte_reads_accept_non_utf8_inputs() {
    let root = fixture_root("bytes");
    reset(&root);
    let path = root.join("schema.bin");
    fs::write(&path, [0xff, 0x00]).expect("write binary schema");

    assert_eq!(
        read_bytes_with_limit(&path, 2).expect("read bytes"),
        [0xff, 0x00]
    );
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn creation_is_complete_and_never_replaces_existing_state() {
    let root = fixture_root("create");
    reset(&root);
    let path = root.join("zrail.toml");

    create_text(&path, "new\n").expect("create contract");
    let error = create_text(&path, "replacement\n").expect_err("creation must not replace");

    assert!(error.contains("already exists"));
    assert_eq!(read_text(&path).expect("read contract"), "new\n");
    assert_eq!(fs::read_dir(&root).expect("read fixture").count(), 1);
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn replacement_is_complete_and_leaves_no_temporary_file() {
    let root = fixture_root("replace");
    reset(&root);
    let path = root.join("zrail.lock");
    fs::write(&path, "old").expect("write old lock");

    replace_text(&path, "new\n").expect("replace lock");

    assert_eq!(read_text(&path).expect("read lock"), "new\n");
    assert_eq!(fs::read_dir(&root).expect("read fixture").count(), 1);
    fs::remove_dir_all(root).expect("remove fixture");
}

#[cfg(unix)]
#[test]
fn reads_and_replacements_reject_symlinks() {
    use std::os::unix::fs::symlink;

    let root = fixture_root("symlink");
    reset(&root);
    let target = root.join("target");
    let alias = root.join("zrail.lock");
    fs::write(&target, "secret").expect("write target");
    symlink(&target, &alias).expect("create alias");

    assert!(
        read_text(&alias)
            .expect_err("read must fail")
            .contains("symlink")
    );
    assert!(
        replace_text(&alias, "replacement")
            .expect_err("replace must fail")
            .contains("symlink")
    );
    assert!(
        create_text(&alias, "replacement")
            .expect_err("creation must fail")
            .contains("symlink")
    );
    assert_eq!(fs::read_to_string(&target).expect("read target"), "secret");
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn reads_stop_at_the_configured_byte_limit() {
    let root = fixture_root("oversized");
    reset(&root);
    let path = root.join("large.rs");
    fs::write(&path, "12345").expect("write input");

    let error = read_text_with_limit(&path, 4).expect_err("oversized input must fail");

    assert!(error.contains("4-byte safety limit"));
    fs::remove_dir_all(root).expect("remove fixture");
}

fn fixture_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("zrail-input-{name}-{}", std::process::id()))
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
    fs::create_dir_all(root).expect("create fixture");
}
