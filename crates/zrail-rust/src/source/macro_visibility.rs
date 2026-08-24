//! Repository glob candidates bind through a bounded, exact module visibility graph.

use std::collections::BTreeSet;

use zrail_core::AnalysisQuality;

use super::{
    MacroCandidate, MacroDerivation, MacroExpansionFact, MacroImportFact, ObservedFact,
    Reachability, SyntaxGuard,
};

pub(super) use super::macro_visibility_graph::MacroVisibility;
use super::macro_visibility_graph::VisibilityLookup;

const MAX_MACRO_CANDIDATES: usize = 64;

impl MacroVisibility {
    pub(super) fn resolve(
        &self,
        invocation: &mut MacroExpansionFact,
        file: &str,
        file_reachability: Reachability,
        local_macros: Option<&BTreeSet<&str>>,
    ) {
        let mut candidates = Vec::new();
        let mut resolved_leaves = BTreeSet::new();
        for candidate in invocation.candidates.drain(..) {
            if candidate.derivation != MacroDerivation::GlobImport
                || !self.repository_candidate(file, &candidate.observation.name)
            {
                candidates.push(candidate);
                continue;
            }
            let reachability = match candidate.observation.guard {
                SyntaxGuard::Ordinary
                | SyntaxGuard::ProductionOnly
                | SyntaxGuard::Conditional
                | SyntaxGuard::ConditionalProductionOnly => file_reachability,
                SyntaxGuard::TestOnly | SyntaxGuard::ConditionalTestOnly => Reachability::test(),
                SyntaxGuard::Never => Reachability::UNREACHABLE,
            };
            match self.imports_for(file, &candidate.observation.name, reachability) {
                VisibilityLookup::Known(imports)
                    if imports
                        .iter()
                        .any(|import| import.guard.available_in(candidate.observation.guard)) =>
                {
                    resolved_leaves.insert(leaf(&candidate.observation.name).to_owned());
                    candidates.extend(
                        imports
                            .into_iter()
                            .filter(|import| import.guard.available_in(candidate.observation.guard))
                            .map(|import| {
                                imported_candidate(&candidate.observation, import, local_macros)
                            }),
                    );
                }
                VisibilityLookup::Known(_) | VisibilityLookup::Unknown
                    if contains_local(local_macros, &candidate.observation.name) =>
                {
                    resolved_leaves.insert(leaf(&candidate.observation.name).to_owned());
                    candidates.push(candidate);
                }
                VisibilityLookup::Known(_) => {}
                VisibilityLookup::Unknown => candidates.push(MacroCandidate::unresolved(
                    unresolved(candidate.observation),
                    MacroDerivation::GlobImport,
                )),
            }
        }
        candidates.retain(|candidate| {
            candidate.derivation != MacroDerivation::Written
                || !resolved_leaves.contains(leaf(&candidate.observation.name))
        });
        candidates.sort_by(candidate_order);
        candidates.dedup_by(|left, right| same_candidate(left, right));
        invocation.candidates = if candidates.len() > MAX_MACRO_CANDIDATES {
            vec![MacroCandidate::unresolved(
                unresolved(invocation.observation.clone()),
                MacroDerivation::GlobImport,
            )]
        } else {
            candidates
        };
        invocation.refresh_quality();
    }
}

fn imported_candidate(
    original: &ObservedFact,
    import: &MacroImportFact,
    local_macros: Option<&BTreeSet<&str>>,
) -> MacroCandidate {
    let local = !import.target.contains("::") && contains_local(local_macros, &import.target);
    let mut observation = ObservedFact {
        name: if local {
            original.name.clone()
        } else {
            import.target.clone()
        },
        written: original.written.clone(),
        canonical: Vec::new(),
        span: original.span,
        quality: if local {
            AnalysisQuality::Conservative
        } else {
            import.quality
        },
        guard: original.guard,
        lexical_scope: original.lexical_scope.clone(),
        namespace: original.namespace,
    };
    if local {
        observation.canonical.push(original.name.clone());
    }
    let mut candidate = MacroCandidate::pending(
        observation,
        local || repository_path(&import.target),
        if import.re_export {
            MacroDerivation::ReExport
        } else {
            MacroDerivation::ExactImport
        },
    );
    candidate.written_alias = false;
    candidate
}

fn unresolved(mut observation: ObservedFact) -> ObservedFact {
    observation.canonical.clear();
    observation.quality = AnalysisQuality::Unresolved;
    observation
}

fn contains_local(local: Option<&BTreeSet<&str>>, path: &str) -> bool {
    local.is_some_and(|names| names.contains(leaf(path)))
}

fn candidate_order(left: &MacroCandidate, right: &MacroCandidate) -> std::cmp::Ordering {
    left.observation
        .name
        .cmp(&right.observation.name)
        .then(left.observation.canonical.cmp(&right.observation.canonical))
        .then(left.observation.quality.cmp(&right.observation.quality))
        .then(left.derivation.cmp(&right.derivation))
        .then(left.written_alias.cmp(&right.written_alias))
        .then(left.definition.cmp(&right.definition))
        .then(left.definition_name.cmp(&right.definition_name))
        .then(left.definition_sha256.cmp(&right.definition_sha256))
        .then(left.origins.cmp(&right.origins))
}

fn same_candidate(left: &MacroCandidate, right: &MacroCandidate) -> bool {
    left.observation.name == right.observation.name
        && left.observation.canonical == right.observation.canonical
        && left.observation.quality == right.observation.quality
        && left.derivation == right.derivation
        && left.written_alias == right.written_alias
        && left.definition == right.definition
        && left.definition_name == right.definition_name
        && left.definition_sha256 == right.definition_sha256
        && left.origins == right.origins
}

pub(super) fn repository_path(path: &str) -> bool {
    path.split("::")
        .next()
        .is_some_and(|root| matches!(root, "crate" | "self" | "super"))
}

fn leaf(path: &str) -> &str {
    path.rsplit("::").next().unwrap_or(path)
}

#[cfg(test)]
#[path = "macro_visibility_test.rs"]
mod macro_visibility_test;
