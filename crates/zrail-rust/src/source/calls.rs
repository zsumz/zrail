//! Direct function and associated-function calls with conservative glob candidates.

use std::borrow::Cow;
use std::collections::BTreeSet;

use syn::{Expr, ExprCall, ExprPath, Path, Type, spanned::Spanned as _};
use zrail_core::{AnalysisQuality, Finding};

use super::{
    CompilationDomain, ObservedFact, SyntaxGuard,
    fact::{source_span, written_fact, written_path},
    imports::ImportMap,
    model::CallResolutionFact,
};

mod candidates;

#[cfg(test)]
pub(crate) use candidates::MAX_MACRO_CANDIDATES;
pub(super) use candidates::{candidates, macro_candidates};

pub(super) fn facts(
    call: &ExprCall,
    imports: &ImportMap,
    guard: &SyntaxGuard,
    generic_types: &[String],
    lexical_scope: &[zrail_core::SourceSpan],
) -> Vec<ObservedFact> {
    let Some(callee) = callee_path(call.func.as_ref()) else {
        return Vec::new();
    };
    if unresolved_call_text(callee, generic_types).is_some() {
        return Vec::new();
    }
    let Some((path, minimum_quality)) = effective_path(callee, generic_types) else {
        let written = written_path(&callee.path);
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
    let written = written_path(&path);
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

pub(super) fn unresolved_path_projection(
    path: &ExprPath,
    guard: SyntaxGuard,
    generic_types: &[String],
) -> Option<CallResolutionFact> {
    Some(CallResolutionFact {
        written: unresolved_call_text(path, generic_types)?,
        span: source_span(path.span()),
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
                    "qualified expression path crosses an associated-type projection that zrail cannot resolve exactly: {}",
                    call.written
                ),
            )
            .at(path, Some(call.span))
            .with_analysis(AnalysisQuality::Unresolved)
            .with_help(
                "name one concrete type before trusting path or direct-call authority at this site",
            )
        })
        .collect()
}

fn callee_path(expression: &Expr) -> Option<&ExprPath> {
    match expression {
        Expr::Path(path) => Some(path),
        Expr::Paren(paren) => callee_path(&paren.expr),
        Expr::Group(group) => callee_path(&group.expr),
        _ => None,
    }
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
    Some(qualified_self_text(callee, qself))
}

fn unresolved_call_text(callee: &ExprPath, generic_types: &[String]) -> Option<String> {
    if let Some(projection) = projection_text(callee) {
        return Some(projection);
    }
    let qself = callee.qself.as_ref()?;
    if qself.position > 0 {
        return None;
    }
    let Type::Path(self_type) = qself.ty.as_ref() else {
        return Some(qualified_self_text(callee, qself));
    };
    let generic = self_type.qself.is_some()
        || self_type.path.segments.first().is_some_and(|segment| {
            generic_types
                .iter()
                .any(|generic| segment.ident == generic.as_str())
        });
    generic.then(|| qualified_self_text(callee, qself))
}

fn qualified_self_text(callee: &ExprPath, qself: &syn::QSelf) -> String {
    let self_type = match qself.ty.as_ref() {
        Type::Path(path) if path.qself.is_none() => written_path(&path.path),
        _ => "unresolved self type".into(),
    };
    if qself.position == 0 {
        return format!(
            "<{self_type}>::{}",
            segment_text(callee.path.segments.iter())
        );
    }
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
    format!("<{self_type} as {trait_path}>::{associated_path}")
}

fn segment_text<'a>(segments: impl Iterator<Item = &'a syn::PathSegment>) -> String {
    segments
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
#[path = "calls_test.rs"]
mod calls_test;

#[cfg(test)]
#[path = "calls/tests/projection.rs"]
mod calls_projection_test;
