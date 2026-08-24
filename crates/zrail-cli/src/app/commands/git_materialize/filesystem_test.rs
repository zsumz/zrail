//! Snapshot filesystem writes remain create-only and temporary.

use std::fs;

use super::{TemporaryRoot, write_new};

#[test]
fn temporary_root_removes_create_only_nested_files_on_drop() {
    let temporary = TemporaryRoot::create().expect("create temporary root");
    let root = temporary.path().to_owned();
    let destination = root.join("nested/input.txt");

    write_new(&destination, b"first").expect("write new file");
    assert_eq!(fs::read(&destination).expect("read file"), b"first");
    assert!(write_new(&destination, b"second").is_err());

    drop(temporary);
    assert!(!root.exists());
}
