//! Exact frozen prefix operation; the returned text is not normalized or parsed.

pub(super) fn expected_version(name: &str, version: &str) -> String {
    let version = version
        .strip_prefix('=')
        .unwrap_or_else(|| panic!("{name} guardrail version must be exact"));
    version.to_owned()
}
