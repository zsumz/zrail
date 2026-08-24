//! Direct function and associated-function calls with conservative glob candidates.

use std::borrow::Cow;
use std::collections::BTreeSet;

use syn::{Expr, ExprCall, ExprPath, Path, Type, spanned::Spanned as _};
use zrail_core::{AnalysisQuality, Finding};

use super::{
    CompilationDomain, MacroDerivation, ObservedFact, SyntaxGuard,
    fact::{fact, source_span, written_fact},
    imports::{ImportCandidateKind, ImportMap},
    model::CallResolutionFact,
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
    if projection_text(callee).is_some() {
        return Vec::new();
    }
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

pub(super) fn unresolved_projection(
    call: &ExprCall,
    guard: SyntaxGuard,
) -> Option<CallResolutionFact> {
    let Expr::Path(callee) = call.func.as_ref() else {
        return None;
    };
    Some(CallResolutionFact {
        written: projection_text(callee)?,
        span: source_span(callee.span()),
        guard,
    })
}

pub(super) fn resolution_findings(
    path: &str,
    calls: &[CallResolutionFact],
    domains: Option<&BTreeSet<CompilationDomain>>,
) -> Vec<Finding> {
    let Some(domains) = domains else {
        return Vec::new();
    };
    calls
        .iter()
        .filter(|call| {
            domains.iter().any(|domain| {
                call.guard.available_in(SyntaxGuard::for_test_only(
                    domain.mode.enables_cfg_test(),
                ))
            })
        })
        .map(|call| {
            Finding::error(
                "RUST-CALL-001",
                "rust.source.call-resolution",
                "source",
                format!(
                    "qualified call crosses an associated-type projection that zrail cannot resolve exactly: {}",
                    call.written
                ),
            )
            .at(path, Some(call.span))
            .with_analysis(AnalysisQuality::Unresolved)
            .with_help(
                "call one concrete type path before trusting direct-call authority at this site",
            )
        })
        .collect()
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

fn projection_text(callee: &ExprPath) -> Option<String> {
    let qself = callee.qself.as_ref()?;
    if qself.position == 0 {
        return None;
    }
    let associated = callee.path.segments.len().checked_sub(qself.position);
    if associated == Some(1) {
        return None;
    }
    let self_type = match qself.ty.as_ref() {
        Type::Path(path) if path.qself.is_none() => path_text(&path.path),
        _ => "unresolved self type".into(),
    };
    let mut trait_path = segment_text(callee.path.segments.iter().take(qself.position));
    if callee.path.leading_colon.is_some() {
        trait_path.insert_str(0, "::");
    }
    let associated_path = segment_text(callee.path.segments.iter().skip(qself.position));
    let associated_path = if associated_path.is_empty() {
        "unresolved associated call"
    } else {
        &associated_path
    };
    Some(format!("<{self_type} as {trait_path}>::{associated_path}"))
}

fn segment_text<'a>(segments: impl Iterator<Item = &'a syn::PathSegment>) -> String {
    segments
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
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
