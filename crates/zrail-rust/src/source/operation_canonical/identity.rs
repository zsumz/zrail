//! Construction identities retain canonical candidates from every source instance.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    super::{SourceOperationFact, SourceOperationKind},
    resolution,
};
use crate::source::{
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
};

pub(super) fn canonicalize(
    operations: &mut [SourceOperationFact],
    bindings: &IncludeBindings,
    file: &str,
    budget: &mut ProjectionBudget,
    unresolved: &mut BTreeSet<(String, Option<SourceSpan>)>,
) -> Result<(), ProjectionLimit> {
    for operation in operations
        .iter_mut()
        .filter(|operation| operation.kind == SourceOperationKind::TypeConstruction)
    {
        let written = operation
            .identity
            .written
            .as_deref()
            .unwrap_or(&operation.identity.name);
        let result = resolution::resolve(
            bindings,
            file,
            &operation.identity,
            operation.file_local,
            written,
            budget,
        )?;
        if result.expected == 0 {
            continue;
        }
        let mut candidates = BTreeMap::<String, AnalysisQuality>::new();
        for route in result.routes {
            candidates
                .entry(route.name)
                .and_modify(|quality| *quality = (*quality).max(route.quality))
                .or_insert(route.quality);
        }
        let mut quality = candidates
            .values()
            .copied()
            .max()
            .unwrap_or(AnalysisQuality::Unresolved);
        if result.unresolved {
            quality = AnalysisQuality::Unresolved;
        } else if candidates.len() > 1 {
            quality = quality.max(AnalysisQuality::Conservative);
        }
        if !operation.exact_construction_syntax {
            quality = quality.max(operation.identity.quality);
        }
        match candidates.len() {
            0 => operation.identity.quality = AnalysisQuality::Unresolved,
            1 => {
                if let Some((name, _)) = candidates.into_iter().next() {
                    operation.identity.name = name;
                    operation.identity.canonical.clear();
                    operation.identity.quality = quality;
                } else {
                    operation.identity.quality = AnalysisQuality::Unresolved;
                }
            }
            _ => {
                operation.identity.canonical = candidates.into_keys().collect();
                operation.identity.quality = quality;
            }
        }
        operation.file_local = false;
        if operation.exact_construction_syntax && result.blocks_completeness {
            unresolved.insert((file.into(), operation.identity.span));
        }
    }
    Ok(())
}
