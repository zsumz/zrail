//! Exact frozen array precondition, isolated before element filtering and required counts.

pub(super) fn package_count(lock: &toml::Value) -> usize {
    let packages = lock["package"]
        .as_array()
        .unwrap_or_else(|| panic!("Cargo.lock must contain package entries"));
    packages.len()
}
