//! Direct function and associated-function calls with conservative glob candidates.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use syn::{Expr, ExprCall, ExprPath, Path, Type, spanned::Spanned as _};
use zrail_core::{AnalysisQuality, Finding};

use super::{
    AssociatedOccurrenceKind, CompilationDomain, GenericAssociatedCandidate, GenericRootShadow,
    ObservedFact, RustFileFacts, SyntaxGuard,
    fact::{source_span, written_fact, written_path},
    imports::ImportMap,
    model::{CallResolutionFact, CallResolutionKind},
    operation_model::subject::WrittenOperationSubject,
};

mod candidates;
#[path = "calls_contextual_projection.rs"]
mod contextual_projection;

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
                call.guard
                    .available_in(SyntaxGuard::for_test_only(domain.mode.enables_cfg_test()))
            })
        })
        .map(|call| resolution_finding(path, call))
        .collect()
}

pub(crate) fn resolution_finding(path: &str, call: &CallResolutionFact) -> Finding {
    let (message, help) = match call.kind {
        CallResolutionKind::AssociatedTypeProjection
        | CallResolutionKind::ContextualAssociatedTypeProjection => (
            format!(
                "qualified expression path crosses an associated-type projection that zrail cannot resolve exactly: {}",
                call.written
            ),
            "name one concrete type before trusting path or direct-call authority at this site",
        ),
        CallResolutionKind::ExplicitTrait => (
            format!(
                "explicit trait in qualified associated-item path cannot be resolved exactly: {}",
                call.written
            ),
            "import or qualify the exact trait before trusting associated-item or direct-call authority at this site",
        ),
        CallResolutionKind::GenericAssociatedItem => (
            format!(
                "generic-root associated item cannot be resolved to one exact policy identity: {}",
                call.written
            ),
            "use an explicit trait-qualified path before trusting associated-item, capability, or direct-call authority at this site",
        ),
    };
    Finding::error(
        "RUST-CALL-001",
        "rust.source.call-resolution",
        "source",
        message,
    )
    .at(path, Some(call.span))
    .with_analysis(AnalysisQuality::Unresolved)
    .with_help(help)
}

pub(super) fn generic_resolution_boundaries(file: &RustFileFacts) -> Vec<CallResolutionFact> {
    let call_sites = file
        .calls
        .iter()
        .filter_map(boundary_key)
        .collect::<BTreeSet<_>>();
    let mut boundaries = BTreeMap::<
        (String, zrail_core::SourceSpan, SyntaxGuard),
        (AssociatedOccurrenceKind, Vec<GenericAssociatedCandidate>),
    >::new();
    for fact in file.paths.iter().chain(&file.calls).filter(|fact| {
        fact.generic_shadow == Some(GenericRootShadow::TypeParameter)
            && fact
                .written
                .as_deref()
                .is_some_and(|written| written.contains("::"))
    }) {
        let (Some(written), Some(span)) = (fact.written.as_ref(), fact.span) else {
            continue;
        };
        let key = (written.clone(), span, fact.guard.clone());
        let occurrence = if call_sites.contains(&key) {
            AssociatedOccurrenceKind::DirectCall
        } else if fact.namespace == super::FactNamespace::Type {
            AssociatedOccurrenceKind::TypeReference
        } else {
            AssociatedOccurrenceKind::ValueReference
        };
        let entry = boundaries
            .entry(key)
            .or_insert_with(|| (occurrence, Vec::new()));
        if occurrence == AssociatedOccurrenceKind::DirectCall {
            entry.0 = occurrence;
        }
        merge_candidates(&mut entry.1, &fact.associated_candidates);
    }
    boundaries
        .into_iter()
        .map(
            |((written, span, guard), (occurrence, associated_candidates))| CallResolutionFact {
                written,
                span,
                guard,
                kind: CallResolutionKind::GenericAssociatedItem,
                associated_candidates,
                occurrence: Some(occurrence),
            },
        )
        .collect()
}

fn boundary_key(fact: &ObservedFact) -> Option<(String, zrail_core::SourceSpan, SyntaxGuard)> {
    (fact.generic_shadow == Some(GenericRootShadow::TypeParameter))
        .then(|| Some((fact.written.clone()?, fact.span?, fact.guard.clone())))
        .flatten()
}

fn merge_candidates(
    target: &mut Vec<GenericAssociatedCandidate>,
    candidates: &[GenericAssociatedCandidate],
) {
    for candidate in candidates {
        if let Some(existing) = target.iter_mut().find(|existing| {
            existing.name == candidate.name && existing.canonical == candidate.canonical
        }) {
            existing.quality = existing.quality.max(candidate.quality);
        } else {
            target.push(candidate.clone());
        }
    }
    target.sort();
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
