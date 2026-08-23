//! Small deterministic helpers shared by include binding lookup.

use zrail_core::AnalysisQuality;

use super::include_bindings::{BindingSite, ResolvedPath};

pub(super) fn select_site(
    selected: &mut Vec<BindingSite>,
    selected_depth: &mut Option<usize>,
    depth: usize,
    site: BindingSite,
) {
    if selected_depth.is_none_or(|current| depth > current) {
        selected.clear();
        *selected_depth = Some(depth);
    }
    if *selected_depth == Some(depth) {
        selected.push(site);
    }
}

pub(super) fn normalize(mut paths: Vec<ResolvedPath>) -> Vec<ResolvedPath> {
    paths.sort();
    paths.dedup();
    paths
}

pub(super) fn unresolved(name: &str) -> ResolvedPath {
    ResolvedPath {
        name: name.into(),
        quality: AnalysisQuality::Unresolved,
        crossed_include: true,
    }
}

pub(super) fn split_root(path: &str) -> (&str, &str) {
    path.find("::").map_or((path, ""), |separator| {
        (&path[..separator], &path[separator..])
    })
}

pub(super) fn join(prefix: &str, suffix: &str) -> String {
    format!("{prefix}{suffix}")
}
