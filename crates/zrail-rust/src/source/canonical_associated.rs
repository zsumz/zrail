//! Cargo dependency roots canonicalize candidate trait-associated identities.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::super::{GenericAssociatedCandidate, ProviderAuthority};

pub(super) fn apply(
    candidate: &mut GenericAssociatedCandidate,
    roots: &BTreeMap<String, BTreeSet<String>>,
    overflowed: &BTreeSet<String>,
) {
    let Some((root, suffix)) = split_root(&candidate.name) else {
        return;
    };
    let visible = visible_root(root);
    if overflowed.contains(visible) {
        candidate.canonical.clear();
        candidate.quality = AnalysisQuality::Unresolved;
        candidate
            .provider_authorities
            .insert(ProviderAuthority::Unknown);
        return;
    }
    let Some(canonical_roots) = roots.get(visible) else {
        return;
    };
    candidate.canonical = canonical_roots
        .iter()
        .map(|canonical| format!("{canonical}{suffix}"))
        .filter(|canonical| canonical != &candidate.name)
        .collect();
    candidate.provider_authorities.extend(
        canonical_roots
            .iter()
            .filter_map(|canonical| canonical.trim_start_matches("::").split("::").next())
            .map(|root| ProviderAuthority::ExternalRoot(visible_root(root).into())),
    );
    if canonical_roots.len() > 1 {
        candidate.quality = candidate.quality.max(AnalysisQuality::Conservative);
    }
}

fn split_root(path: &str) -> Option<(&str, &str)> {
    (!path.is_empty()).then(|| {
        path.find("::").map_or((path, ""), |separator| {
            (&path[..separator], &path[separator..])
        })
    })
}

fn visible_root(root: &str) -> &str {
    root.strip_prefix("r#").unwrap_or(root)
}
