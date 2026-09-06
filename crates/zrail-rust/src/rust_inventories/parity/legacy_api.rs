//! Trusted access to the unchanged frozen collector quantities.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    AuthorityInventory, counts, expected_selector_methods, repository_inventory, source_inventory,
};

pub(in super::super) fn observed(path: &str, source: &str) -> BTreeMap<String, usize> {
    source_inventory(path, source).selector_methods
}

pub(in super::super) fn owners(path: &str, source: &str) -> BTreeSet<String> {
    source_inventory(path, source).connection_set_files
}

pub(in super::super) fn check_owners(connection_set_files: BTreeSet<String>) {
    let actual = AuthorityInventory {
        connection_set_files,
        ..AuthorityInventory::default()
    };
    assert_eq!(
        actual.connection_set_files,
        BTreeSet::from([SET_OWNER.into()])
    );
}

pub(in super::super) fn expected_owner_files() -> BTreeSet<String> {
    BTreeSet::from([SET_OWNER.into()])
}

pub(in super::super) fn repository_owner_files(
    root: &std::path::Path,
    roots: &[String],
) -> BTreeSet<String> {
    repository_inventory(root, roots).connection_set_files
}

pub(in super::super) fn check_detector_owners(connection_set_files: BTreeSet<String>) {
    let actual = AuthorityInventory {
        connection_set_files,
        ..AuthorityInventory::default()
    };
    assert_eq!(
        actual.connection_set_files,
        BTreeSet::from(["src/reactor/rogue.rs".into()])
    );
}

pub(in super::super) fn expected() -> BTreeMap<String, usize> {
    expected_selector_methods()
}

pub(in super::super) fn associated(path: &str, source: &str) -> BTreeMap<String, usize> {
    source_inventory(path, source).associated_calls
}

const SET_OWNER: &str = "src/reactor/direct_plaintext/set_owner.rs";
const RUSTLS_ADAPTER: &str = "src/reactor/direct_plaintext/rustls_transport.rs";

fn expected_associated_calls() -> BTreeMap<String, usize> {
    counts(&[
        (&format!("{SET_OWNER}:ConnectionSet::new"), 1),
        (&format!("{SET_OWNER}:ConnectionSet::turn_component"), 1),
        (&format!("{SET_OWNER}:ConnectionSet::poll_io"), 1),
        (&format!("{SET_OWNER}:ConnectionSet::wake_handle"), 1),
        (&format!("{SET_OWNER}:ConnectionSet::pulse_handle"), 1),
        (&format!("{RUSTLS_ADAPTER}:Source::register"), 1),
        (&format!("{RUSTLS_ADAPTER}:Source::reregister"), 1),
        (&format!("{RUSTLS_ADAPTER}:Source::deregister"), 1),
    ])
}

pub(in super::super) fn expected_associated() -> BTreeMap<String, usize> {
    expected_associated_calls()
}

pub(in super::super) fn repository_associated(
    root: &std::path::Path,
    roots: &[String],
) -> BTreeMap<String, usize> {
    repository_inventory(root, roots).associated_calls
}

pub(in super::super) fn check_associated(associated_calls: BTreeMap<String, usize>) {
    let actual = AuthorityInventory {
        associated_calls,
        ..AuthorityInventory::default()
    };
    assert_eq!(actual.associated_calls, expected_associated_calls());
}

pub(in super::super) fn check_detector_associated(associated_calls: BTreeMap<String, usize>) {
    let actual = AuthorityInventory {
        associated_calls,
        ..AuthorityInventory::default()
    };
    assert_eq!(
        actual.associated_calls,
        counts(&[
            ("src/reactor/rogue.rs:ConnectionSet::new", 1),
            ("src/reactor/rogue.rs:DirectSet::new", 1),
            ("src/reactor/rogue.rs:DirectSet::poll_io", 1),
            ("src/reactor/rogue.rs:DirectSet::turn_component", 1),
            ("src/reactor/rogue.rs:DirectSet::wake_handle", 1),
            ("src/reactor/rogue.rs:DirectSet::pulse_handle", 1),
        ])
    );
}
