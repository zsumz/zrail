//! Direct function and associated-function calls with conservative glob candidates.

use std::borrow::Cow;

use syn::{Expr, ExprCall, ExprPath, Path, Type, spanned::Spanned as _};
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
    generic_types: &[String],
    lexical_scope: &[zrail_core::SourceSpan],
) -> Vec<ObservedFact> {
    let Expr::Path(callee) = call.func.as_ref() else {
        return Vec::new();
    };
    let Some((path, minimum_quality)) = effective_path(callee, generic_types) else {
        let written = path_text(&callee.path);
        return vec![written_fact(
            written.clone(),
            written,
            callee.span(),
            AnalysisQuality::Unresolved,
            lexical_scope,
        )];
    };
    let (resolved, quality) = imports.resolve(&path, guard);
    if resolved.is_empty() {
        return Vec::new();
    }
    let written = path_text(&path);
    let mut observed = vec![written_fact(
        resolved.clone(),
        written,
        callee.span(),
        quality.max(minimum_quality),
        lexical_scope,
    )];
    observed.extend(candidates(&path, imports, &resolved, guard));
    observed
}

fn effective_path<'a>(
    callee: &'a ExprPath,
    generic_types: &[String],
) -> Option<(Cow<'a, Path>, AnalysisQuality)> {
    let Some(qself) = &callee.qself else {
        return Some((Cow::Borrowed(&callee.path), AnalysisQuality::Exact));
    };
    if qself.position > 0 {
        return Some((Cow::Borrowed(&callee.path), AnalysisQuality::Exact));
    }
    let Type::Path(self_type) = qself.ty.as_ref() else {
        return None;
    };
    if self_type.qself.is_some() {
        return None;
    }
    let mut path = self_type.path.clone();
    let generic = path.segments.first().is_some_and(|segment| {
        segment.ident == "Self"
            || generic_types
                .iter()
                .any(|generic| segment.ident == generic.as_str())
    });
    path.segments.extend(callee.path.segments.iter().cloned());
    Some((
        Cow::Owned(path),
        if generic {
            AnalysisQuality::Unresolved
        } else {
            AnalysisQuality::Exact
        },
    ))
}

fn path_text(path: &Path) -> String {
    let mut written = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    if path.leading_colon.is_some() {
        written.insert_str(0, "::");
    }
    written
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
