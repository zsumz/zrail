//! Exact associated `Self` returns repair inferred local field-place bases.

use std::collections::BTreeSet;

use zrail_core::AnalysisQuality;

use super::{associated, resolution};
use crate::source::{
    SourceOperationFact, SourceSyntax,
    include_bindings::{IncludeBindings, ResolvedTerminal},
    include_projection_budget::ProjectionLimit,
    include_resolution_state::ResolutionUsage,
    operation_model::{AssociatedReturnInference, OperationSubjectOrigin},
};

pub(super) fn canonicalize(
    operations: &mut [SourceOperationFact],
    bindings: &IncludeBindings,
    file: &str,
    syntax: SourceSyntax,
    associated: &associated::Catalog,
    resolver: &mut resolution::Resolver<'_>,
) -> Result<(), ProjectionLimit> {
    for operation in operations {
        let Some(inference) = operation
            .place
            .as_ref()
            .and_then(|place| place.base_inference.clone())
        else {
            continue;
        };
        let Some((base_fact, item)) = split(&inference) else {
            continue;
        };
        let written = base_fact.name.clone();
        let mut resolved = resolver.resolve(resolution::Request {
            bindings,
            file,
            syntax,
            fact: &base_fact,
            file_local: false,
            subject_origin: inference.subject_origin,
            written: &written,
            usage: ResolutionUsage::Type,
            construction: None,
            root_lookup: Some(inference.root_lookup),
            generic_shadow: inference.generic_shadow,
        })?;
        if resolved.expected == 0
            || resolved.suppressed != 0
            || resolved.routes.len() != resolved.expected
        {
            continue;
        }
        let mut bases = BTreeSet::new();
        let mut exact = true;
        for route in &mut resolved.routes {
            if route.terminal != ResolvedTerminal::Type {
                exact = false;
                break;
            }
            route.name.push_str("::");
            route.name.push_str(&item);
            route.terminal = ResolvedTerminal::Unknown;
            associated.classify_value(
                route,
                &inference.fact.guard,
                super::qualification::TraitSelection::Ordinary,
            );
            let Some((base, _)) = route.name.rsplit_once("::") else {
                exact = false;
                break;
            };
            if route.quality != AnalysisQuality::Exact
                || !associated.returns_self(route, &inference.fact.guard, inference.try_depth)
            {
                exact = false;
                break;
            }
            bases.insert(base.to_owned());
        }
        if !exact {
            continue;
        }
        let bases = bases.into_iter().collect::<Vec<_>>();
        let [base] = bases.as_slice() else {
            continue;
        };
        let Some(place) = &mut operation.place else {
            continue;
        };
        place.base_name.clone_from(base);
        place.base_quality = AnalysisQuality::Exact;
        place.base_file_local = false;
        place.base_origin = OperationSubjectOrigin::WrittenPath;
        place.base_span = None;
        place.base_inference = None;
    }
    Ok(())
}

fn split(inference: &AssociatedReturnInference) -> Option<(crate::source::ObservedFact, String)> {
    let mut fact = inference.fact.clone();
    let (base, item) = fact.name.rsplit_once("::")?;
    let base = base.to_owned();
    let item = item.to_owned();
    fact.name = base;
    fact.written = fact
        .written
        .and_then(|written| written.rsplit_once("::").map(|(base, _)| base.to_owned()));
    Some((fact, item))
}
