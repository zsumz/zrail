//! Direct function and associated-function calls with conservative glob candidates.

use std::borrow::Cow;

use syn::{Expr, ExprCall, ExprPath, Path, Type, spanned::Spanned as _};
use zrail_core::AnalysisQuality;

use super::{
    ObservedFact, SyntaxGuard,
    fact::{source_span, written_fact, written_path},
    imports::ImportMap,
    model::{CallResolutionFact, CallResolutionKind},
    operation_model::subject::WrittenOperationSubject,
};

mod candidates;
#[path = "calls_contextual_projection.rs"]
mod contextual_projection;
#[path = "calls_resolution.rs"]
mod resolution;

#[cfg(test)]
pub(crate) use candidates::MAX_MACRO_CANDIDATES;
pub(super) use candidates::{candidates, macro_candidates};
pub(crate) use resolution::resolution_finding;
pub(super) use resolution::{
    generic_resolution_boundaries, normalize_resolutions, resolution_findings,
};

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
    let (written, kind) = match unresolved_call_text(path, generic_types) {
        Some(written) => (written, CallResolutionKind::AssociatedTypeProjection),
        None => (
            contextual_projection::text(path)?,
            CallResolutionKind::ContextualAssociatedTypeProjection,
        ),
    };
    Some(CallResolutionFact {
        written,
        span: source_span(path.span()),
        guard,
        kind,
        associated_candidates: Vec::new(),
        occurrence: None,
    })
}

pub(super) fn callee_path(expression: &Expr) -> Option<&ExprPath> {
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
    let subject = WrittenOperationSubject::from_expression(callee);
    let Some(qself) = &callee.qself else {
        return Some((subject.call_path()?, AnalysisQuality::Exact));
    };
    if qself.position > 0 {
        return Some((subject.call_path()?, AnalysisQuality::Exact));
    }
    let Type::Path(self_type) = qself.ty.as_ref() else {
        return None;
    };
    if self_type.qself.is_some() {
        return None;
    }
    let generic = self_type.path.segments.first().is_some_and(|segment| {
        segment.ident == "Self"
            || generic_types
                .iter()
                .any(|generic| segment.ident == generic.as_str())
    });
    Some((
        subject.call_path()?,
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
    Some(WrittenOperationSubject::from_expression(callee).written())
}

fn unresolved_call_text(callee: &ExprPath, generic_types: &[String]) -> Option<String> {
    if callee.qself.is_none()
        && callee.path.segments.len() > 2
        && callee.path.segments.first().is_some_and(|segment| {
            generic_types
                .iter()
                .any(|generic| segment.ident == generic.as_str())
        })
    {
        return Some(WrittenOperationSubject::from_expression(callee).written());
    }
    if let Some(projection) = projection_text(callee) {
        return Some(projection);
    }
    let qself = callee.qself.as_ref()?;
    if qself.position > 0 {
        return None;
    }
    let Type::Path(self_type) = qself.ty.as_ref() else {
        return Some(WrittenOperationSubject::from_expression(callee).written());
    };
    let generic = self_type.qself.is_some()
        || self_type.path.segments.first().is_some_and(|segment| {
            generic_types
                .iter()
                .any(|generic| segment.ident == generic.as_str())
        });
    generic.then(|| WrittenOperationSubject::from_expression(callee).written())
}

#[cfg(test)]
#[path = "calls_test.rs"]
mod calls_test;

#[cfg(test)]
#[path = "calls/tests/projection.rs"]
mod calls_projection_test;
