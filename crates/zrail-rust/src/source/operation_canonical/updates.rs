//! Deferred functional updates enumerate fields only after alias identity is exact.

mod fields;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    super::{
        FactNamespace, ObservedFact, SourceOperationFact, SourceOperationKind,
        operation_model::OperationSubjectOrigin, operation_place_canonical::catalog::Catalog,
    },
    resolution,
};
use crate::source::{
    SyntaxGuard,
    include_binding_helpers::join,
    include_bindings::{IncludeBindings, ResolvedOrigin},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::ResolutionUsage,
};

use fields::{FieldGroup, omitted_guard};

pub(super) fn expand(
    operations: &mut Vec<SourceOperationFact>,
    bindings: &IncludeBindings,
    catalog: &Catalog,
    file: &str,
    budget: &mut ProjectionBudget,
    remaining: &mut usize,
    unresolved_findings: &mut BTreeSet<(String, Option<SourceSpan>)>,
) -> Result<(), ProjectionLimit> {
    let mut expanded = Vec::with_capacity(operations.len());
    for operation in operations.drain(..) {
        let Some(update) = operation.struct_update.as_ref() else {
            expanded.push(operation);
            continue;
        };
        let Some(place) = operation.place.as_ref() else {
            unresolved_findings.insert((file.into(), Some(update.rest_span)));
            expanded.push(field_operation(
                &operation,
                "<unresolved>::*".into(),
                AnalysisQuality::Unresolved,
                operation.identity.guard.clone(),
            ));
            continue;
        };
        let subject = ObservedFact {
            name: place.base_name.clone(),
            written: Some(update.written.clone()),
            canonical: Vec::new(),
            span: place.base_span,
            quality: place.base_quality,
            guard: operation.identity.guard.clone(),
            lexical_scope: operation.identity.lexical_scope.clone(),
            namespace: FactNamespace::Type,
        };
        let result = resolution::resolve(
            resolution::Request {
                bindings,
                file,
                fact: &subject,
                file_local: place.base_file_local,
                subject_origin: place.base_origin,
                written: &update.written,
                usage: ResolutionUsage::OperationType,
                construction: None,
            },
            budget,
        )?;
        if result.expected == 0 {
            expanded.push(operation);
            continue;
        }
        let resolution_failed = result.unresolved || result.blocks_completeness;
        let mut groups = BTreeMap::<String, FieldGroup>::new();
        let mut missing = result.unresolved;
        for route in &result.routes {
            if route.origin != ResolvedOrigin::CrateLocal {
                missing = true;
                continue;
            }
            let Some(fields) = catalog.named_fields(&route.name, &route.domain) else {
                missing = true;
                continue;
            };
            let group = groups.entry(route.name.clone()).or_default();
            group.quality = group.quality.max(route.quality);
            for field in fields {
                group.add(field);
            }
        }
        let base_ambiguity = if groups.len() > 1 {
            AnalysisQuality::Conservative
        } else {
            AnalysisQuality::Exact
        };
        for (base, group) in &groups {
            for field in group.fields.values() {
                let guard =
                    omitted_guard(&operation.identity.guard, &field.guard, &field.name, update);
                if guard.predicate().is_satisfiable() == Some(false) {
                    continue;
                }
                let Some(name) = join(base, &format!("::{}", field.name)) else {
                    missing = true;
                    continue;
                };
                let quality =
                    group
                        .quality
                        .max(field.quality)
                        .max(base_ambiguity)
                        .max(if missing {
                            AnalysisQuality::Unresolved
                        } else {
                            AnalysisQuality::Exact
                        });
                budget.retain_fact(remaining)?;
                push_unique(
                    &mut expanded,
                    field_operation(&operation, name, quality, guard),
                );
            }
        }
        if missing || result.blocks_completeness || groups.is_empty() {
            if resolution_failed {
                unresolved_findings.insert((file.into(), Some(update.rest_span)));
            }
            let bases = if groups.is_empty() {
                vec![place.base_name.clone()]
            } else {
                groups.into_keys().collect()
            };
            for base in bases {
                budget.retain_fact(remaining)?;
                let name = join(&base, "::*").unwrap_or_else(|| "<unresolved>::*".into());
                push_unique(
                    &mut expanded,
                    field_operation(
                        &operation,
                        name,
                        AnalysisQuality::Unresolved,
                        operation.identity.guard.clone(),
                    ),
                );
            }
        }
    }
    *operations = expanded;
    Ok(())
}

fn field_operation(
    source: &SourceOperationFact,
    name: String,
    quality: AnalysisQuality,
    guard: SyntaxGuard,
) -> SourceOperationFact {
    SourceOperationFact {
        kind: SourceOperationKind::FieldRead,
        identity: ObservedFact {
            name,
            written: None,
            canonical: Vec::new(),
            span: source.struct_update.as_ref().map(|update| update.rest_span),
            quality,
            guard,
            lexical_scope: source.identity.lexical_scope.clone(),
            namespace: FactNamespace::Type,
        },
        file_local: false,
        subject_origin: OperationSubjectOrigin::WrittenPath,
        construction: None,
        construction_proven: false,
        method: None,
        place: None,
        struct_update: None,
    }
}

fn push_unique(operations: &mut Vec<SourceOperationFact>, operation: SourceOperationFact) {
    if let Some(existing) = operations.iter_mut().find(|existing| {
        existing.kind == operation.kind
            && existing.identity.name == operation.identity.name
            && existing.identity.span == operation.identity.span
            && existing.identity.guard == operation.identity.guard
    }) {
        existing.identity.quality = existing.identity.quality.max(operation.identity.quality);
    } else {
        operations.push(operation);
    }
}
