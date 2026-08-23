//! Small deterministic helpers shared by include binding lookup.

use zrail_core::AnalysisQuality;

use super::include_bindings::ResolvedPath;

pub(super) const MAX_RESOLVED_PATH_BYTES: usize = 1_024;

pub(super) fn normalize(paths: Vec<ResolvedPath>) -> Vec<ResolvedPath> {
    let mut normalized = std::collections::BTreeMap::<String, ResolvedPath>::new();
    for path in paths {
        let entry = normalized
            .entry(path.name.clone())
            .or_insert_with(|| path.clone());
        entry.quality = entry.quality.max(path.quality);
        entry.crossed_include |= path.crossed_include;
        entry.requires_projection |= path.requires_projection;
    }
    normalized.into_values().collect()
}

pub(super) fn unresolved(name: &str) -> ResolvedPath {
    ResolvedPath {
        name: name.into(),
        quality: AnalysisQuality::Unresolved,
        crossed_include: true,
        requires_projection: true,
    }
}

pub(super) fn split_root(path: &str) -> (&str, &str) {
    path.find("::").map_or((path, ""), |separator| {
        (&path[..separator], &path[separator..])
    })
}

pub(super) fn join(prefix: &str, suffix: &str) -> Option<String> {
    (prefix.len().saturating_add(suffix.len()) <= MAX_RESOLVED_PATH_BYTES)
        .then(|| format!("{prefix}{suffix}"))
}

pub(super) fn canonical_name(prefix: &[String], written: &str) -> Option<String> {
    let prefix_bytes = prefix.iter().map(String::len).sum::<usize>();
    let separators = prefix.len();
    (prefix_bytes
        .saturating_add(separators.saturating_mul(2))
        .saturating_add(written.len())
        <= MAX_RESOLVED_PATH_BYTES)
        .then(|| {
            if prefix.is_empty() {
                written.into()
            } else {
                format!("{}::{written}", prefix.join("::"))
            }
        })
}
