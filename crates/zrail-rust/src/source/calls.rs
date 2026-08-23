//! Direct function and associated-function calls with conservative glob candidates.

use syn::{Expr, ExprCall, spanned::Spanned as _};
use zrail_core::AnalysisQuality;

use super::{
    MacroDerivation, ObservedFact, SyntaxGuard,
    fact::{fact, written_fact},
    imports::{ImportCandidateKind, ImportMap},
};

const MAX_MACRO_CANDIDATES: usize = 64;

pub(super) fn facts(
    call: &ExprCall,
    imports: &ImportMap,
    guard: SyntaxGuard,
    lexical_scope: &[zrail_core::SourceSpan],
) -> Vec<ObservedFact> {
    let Expr::Path(callee) = call.func.as_ref() else {
        return Vec::new();
    };
    let (resolved, quality) = imports.resolve(&callee.path, guard);
    if resolved.is_empty() {
        return Vec::new();
    }
    let span = callee.path.span();
    let mut written = callee
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    if callee.path.leading_colon.is_some() {
        written.insert_str(0, "::");
    }
    let mut observed = vec![written_fact(
        resolved.clone(),
        written,
        span,
        quality,
        lexical_scope,
    )];
    if callee.qself.is_none() {
        observed.extend(candidates(&callee.path, imports, &resolved, guard));
    }
    observed
}

pub(super) fn candidates(
    path: &syn::Path,
    imports: &ImportMap,
    resolved: &str,
    guard: SyntaxGuard,
) -> Vec<ObservedFact> {
    imports
        .call_candidates(path, guard)
        .into_iter()
        .filter(|candidate| candidate.path != resolved)
        .map(|candidate| fact(candidate.path, path.span(), AnalysisQuality::Conservative))
        .collect()
}

pub(super) fn macro_candidates(
    path: &syn::Path,
    imports: &ImportMap,
    resolved: &str,
    guard: SyntaxGuard,
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

#[cfg(test)]
#[path = "calls_test.rs"]
mod calls_test;
