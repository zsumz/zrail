//! Shared normalization for selector-aware tightening ratchets.

/// Returns the canonical Rust-path spelling used in ratchet identities.
///
/// Raw identifier prefixes do not change which denied method or macro a
/// selector governs, so they are removed before selectors enter lock state.
pub fn normalize_ratchet_selector(selector: &str) -> String {
    selector
        .split("::")
        .map(|segment| segment.strip_prefix("r#").unwrap_or(segment))
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
#[path = "ratchet_test.rs"]
mod ratchet_test;
