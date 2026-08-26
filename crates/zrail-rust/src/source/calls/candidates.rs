//! Conservative call and macro candidates inferred from imported paths.

use syn::spanned::Spanned as _;
use zrail_core::AnalysisQuality;

use super::super::{
    MacroDerivation, ObservedFact, SyntaxGuard,
    fact::fact,
    imports::{ImportCandidateKind, ImportMap},
};

pub(crate) const MAX_MACRO_CANDIDATES: usize = 64;

pub(crate) fn candidates(
    path: &syn::Path,
    imports: &ImportMap,
    resolved: &str,
    guard: &SyntaxGuard,
) -> Vec<ObservedFact> {
    imports
        .call_candidates(path, guard)
        .into_iter()
        .filter(|candidate| candidate.path != resolved)
        .map(|candidate| fact(candidate.path, path.span(), AnalysisQuality::Conservative))
        .collect()
}

pub(crate) fn macro_candidates(
    path: &syn::Path,
    imports: &ImportMap,
    resolved: &str,
    guard: &SyntaxGuard,
) -> (Vec<(ObservedFact, MacroDerivation)>, bool) {
    let (candidates, overflowed) =
        imports.bounded_macro_candidates(path, MAX_MACRO_CANDIDATES - 1, guard);
    let candidates = candidates
        .into_iter()
        .filter(|candidate| candidate.path != resolved)
        .map(|candidate| {
            let derivation = match candidate.kind {
                ImportCandidateKind::Exact => MacroDerivation::ExactImport,
                ImportCandidateKind::Glob => MacroDerivation::GlobImport,
                ImportCandidateKind::ReExport => MacroDerivation::ReExport,
            };
            (
                fact(candidate.path, path.span(), AnalysisQuality::Conservative),
                derivation,
            )
        })
        .collect();
    (candidates, overflowed)
}
