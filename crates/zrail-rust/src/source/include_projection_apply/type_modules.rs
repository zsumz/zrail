//! Leafness belongs to a logical module occurrence, including all item splices.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, SourceSpan};

use crate::source::{
    GuardAvailability, IncludeContext, RustFileFacts, SourceEntry, SourceInstanceId,
    include_binding_helpers::canonical_name,
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    type_policy_model::TypeModuleOccurrence,
};

pub(super) fn project(
    bindings: &IncludeBindings,
    file: &RustFileFacts,
    budget: &mut ProjectionBudget,
    remaining: &mut usize,
) -> Result<Vec<Vec<TypeModuleOccurrence>>, ProjectionLimit> {
    let mut cache: BTreeMap<(SourceInstanceId, Vec<SourceSpan>), Result<bool, String>> =
        BTreeMap::new();
    let names = file
        .paths
        .iter()
        .filter_map(|fact| Some((fact.span?, fact.written.as_deref()?)))
        .collect::<BTreeMap<_, _>>();
    let mut declarations = Vec::new();
    for declaration in &file.type_policy.declarations {
        let mut occurrences = Vec::new();
        for id in bindings.active_instances(&file.relative, file.syntax, &declaration.guard) {
            let Some(source) = bindings.instances.get(id) else {
                continue;
            };
            budget.retain_fact(remaining)?;
            let scope = &declaration.lexical_scope;
            let floor = bindings.lexical_floor(id, scope, budget)?;
            let module_scope = &scope[..floor];
            let module = bindings.effective_module(id, module_scope, budget)?;
            let identity = module.as_ref().and_then(|module| {
                canonical_name(&module.names, names.get(&declaration.identity_span)?)
            });
            let key = (id, module_scope.to_vec());
            let leaf = if let Some(value) = cache.get(&key) {
                value.clone()
            } else {
                let value = if module.is_none() {
                    Err("logical module identity is unresolved".into())
                } else if bindings
                    .namespace_opacity(id, module_scope, true, budget)?
                    .is_opaque()
                {
                    Err("logical module namespace is opaque; duplication authority is not namespace authority".into())
                } else {
                    leaf(bindings, id, module_scope, &mut BTreeSet::new(), budget)?
                };
                cache.insert(key, value.clone());
                value
            };
            occurrences.push(TypeModuleOccurrence {
                instance: id,
                domain: source.domain.clone(),
                identity,
                leaf,
            });
        }
        declarations.push(occurrences);
    }
    Ok(declarations)
}

fn leaf(
    bindings: &IncludeBindings,
    id: SourceInstanceId,
    scope: &[SourceSpan],
    seen: &mut BTreeSet<SourceInstanceId>,
    budget: &mut ProjectionBudget,
) -> Result<Result<bool, String>, ProjectionLimit> {
    budget.consume_work()?;
    if !seen.insert(id) {
        return Ok(Ok(true));
    }
    let Some(source) = bindings.instances.get(id) else {
        return Ok(Err("logical module occurrence is unresolved".into()));
    };
    let mut result = Ok(true);
    for binding in bindings
        .files
        .get(&id)
        .and_then(|file| file.modules.get(scope))
        .into_iter()
        .flatten()
    {
        budget.consume_work()?;
        let availability = binding
            .guard
            .combine(&source.guard)
            .availability_in_domain(&source.domain);
        let child = match availability {
            GuardAvailability::Absent => Ok(true),
            GuardAvailability::Exact if binding.quality == AnalysisQuality::Exact => Ok(false),
            _ => Err("child-module availability is unresolved".into()),
        };
        result = combine(result, child);
    }
    for (edge, child) in bindings.instances.includes_from(id) {
        budget.consume_work()?;
        if edge.context == IncludeContext::Items && edge.parent_scope == scope {
            result = combine(result, leaf(bindings, *child, &[], seen, budget)?);
        }
    }
    if scope.is_empty()
        && let (Some(parent), SourceEntry::Include(edge)) = (source.parent, &source.entered_from)
    {
        result = combine(
            result,
            leaf(bindings, parent, &edge.parent_scope, seen, budget)?,
        );
    }
    seen.remove(&id);
    Ok(result)
}

fn combine(left: Result<bool, String>, right: Result<bool, String>) -> Result<bool, String> {
    left.and_then(|left| right.map(|right| left && right))
}
