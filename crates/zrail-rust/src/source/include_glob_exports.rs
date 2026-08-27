//! Included item files export root-scope glob bindings through bounded traversal.

use std::collections::BTreeSet;

use zrail_core::AnalysisQuality;

use super::super::{
    GuardAvailability, IncludeContext, SourceInstanceId, SyntaxGuard,
    include_binding_resolution::{MAX_BINDING_CANDIDATES, MAX_BINDING_STEPS},
    include_bindings::{BindingSite, IncludeBindings},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
};

pub(super) fn collect(
    bindings: &IncludeBindings,
    instance: SourceInstanceId,
    context: &SyntaxGuard,
    budget: &mut ProjectionBudget,
) -> Result<Vec<BindingSite>, ProjectionLimit> {
    collect_inner(bindings, instance, context, &mut BTreeSet::new(), budget)
}

fn collect_inner(
    bindings: &IncludeBindings,
    instance: SourceInstanceId,
    context: &SyntaxGuard,
    seen: &mut BTreeSet<SourceInstanceId>,
    budget: &mut ProjectionBudget,
) -> Result<Vec<BindingSite>, ProjectionLimit> {
    budget.consume_work()?;
    if !seen.insert(instance) || seen.len() > MAX_BINDING_STEPS {
        return Ok(Vec::new());
    }
    let Some(source) = bindings.instances.get(instance) else {
        return Ok(Vec::new());
    };
    let mut sites = Vec::new();
    for binding in bindings
        .files
        .get(&source.file)
        .into_iter()
        .flat_map(|bindings| &bindings.globs)
    {
        budget.consume_work()?;
        let availability = binding
            .guard
            .availability_for_domain(context, &source.domain);
        if binding.lexical_scope.is_empty() && availability.is_available() {
            let Some(module) = bindings.effective_module(instance, &[], budget)? else {
                continue;
            };
            let mut binding = binding.clone();
            if availability == GuardAvailability::Possible {
                binding.quality = AnalysisQuality::Unresolved;
            }
            sites.push(BindingSite {
                binding,
                instance,
                module,
                crossed_include: true,
            });
            if sites.len() > MAX_BINDING_CANDIDATES {
                break;
            }
        }
    }
    for (edge, child) in bindings.instances.includes_from(instance) {
        budget.consume_work()?;
        if edge.context == IncludeContext::Items && edge.parent_scope.is_empty() {
            for site in collect_inner(bindings, *child, context, seen, budget)? {
                sites.push(site);
                if sites.len() > MAX_BINDING_CANDIDATES {
                    break;
                }
            }
        }
        if sites.len() > MAX_BINDING_CANDIDATES {
            break;
        }
    }
    seen.remove(&instance);
    Ok(sites)
}
