//! Small deterministic helpers shared by include binding lookup.

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    include_bindings::{ResolvedOrigin, ResolvedPath, ResolvedTerminal},
    include_resolution_state::ModuleBoundary,
};

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
        entry.blocks_completeness |= path.blocks_completeness;
        if entry.origin != path.origin {
            entry.origin = ResolvedOrigin::Unknown;
            entry.quality = AnalysisQuality::Unresolved;
        }
        if entry.terminal != path.terminal {
            entry.terminal = ResolvedTerminal::Unknown;
            entry.quality = AnalysisQuality::Unresolved;
        }
    }
    normalized.into_values().collect()
}

pub(super) fn unresolved(name: &str) -> ResolvedPath {
    ResolvedPath {
        name: name.into(),
        quality: AnalysisQuality::Unresolved,
        crossed_include: true,
        requires_projection: true,
        blocks_completeness: true,
        origin: ResolvedOrigin::Unknown,
        terminal: ResolvedTerminal::Unknown,
    }
}

pub(super) fn opaque(name: &str, blocks_completeness: bool) -> ResolvedPath {
    ResolvedPath {
        name: name.into(),
        quality: AnalysisQuality::Unresolved,
        crossed_include: false,
        requires_projection: true,
        blocks_completeness,
        origin: ResolvedOrigin::Unknown,
        terminal: ResolvedTerminal::Unknown,
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

pub(super) fn block_local_name(
    boundaries: &[ModuleBoundary],
    lexical_scope: &[SourceSpan],
    file: &str,
    written: &str,
) -> String {
    let anonymous = lexical_scope.iter().rev().find(|span| {
        !boundaries.iter().any(
            |boundary| matches!(boundary, ModuleBoundary::Inline(_, module) if module == *span),
        )
    });
    let Some(scope) = anonymous else {
        return written.into();
    };
    format!("<block@{file}:{}:{}>::{written}", scope.line, scope.column)
}

pub(super) fn canonical_local_name(prefix: &[String], written: &str) -> Option<String> {
    let segments = written.split("::").collect::<Vec<_>>();
    let overlap = (0..=prefix.len().min(segments.len()))
        .rev()
        .find(|count| {
            prefix[prefix.len() - count..]
                .iter()
                .map(String::as_str)
                .eq(segments[..*count].iter().copied())
        })
        .unwrap_or(0);
    let mut names = prefix[..prefix.len() - overlap].to_vec();
    names.extend(segments.into_iter().map(str::to_owned));
    canonical_name(&[], &names.join("::"))
}
