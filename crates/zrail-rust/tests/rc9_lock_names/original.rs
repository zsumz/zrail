//! Exact frozen four-line whole-array selection; count returned only for inspection.

pub(super) fn selected_count(packages: &[toml::Value], name: &str) -> usize {
    let matches = packages
        .iter()
        .filter(|package| package["name"].as_str() == Some(name))
        .collect::<Vec<_>>();
    matches.len()
}
