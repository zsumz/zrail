//! Trusted access to the unchanged frozen collector quantities.

use std::collections::BTreeMap;

use super::{expected_selector_methods, source_inventory};

pub(in super::super) fn observed(path: &str, source: &str) -> BTreeMap<String, usize> {
    source_inventory(path, source).selector_methods
}

pub(in super::super) fn expected() -> BTreeMap<String, usize> {
    expected_selector_methods()
}

pub(in super::super) fn associated(path: &str, source: &str) -> BTreeMap<String, usize> {
    source_inventory(path, source).associated_calls
}
