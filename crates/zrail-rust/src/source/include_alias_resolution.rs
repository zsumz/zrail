//! Alias recursion is cycle-safe, candidate-bounded, and declaration-scoped.

use zrail_core::AnalysisQuality;

use super::{
    BindingKind, SyntaxGuard,
    include_binding_helpers::{normalize, split_root, unresolved},
    include_binding_resolution::MAX_BINDING_CANDIDATES,
    include_bindings::{IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{EffectiveModule, ResolutionKey, ResolutionTrail, ResolveRequest},
};

impl IncludeBindings {
    pub(super) fn resolve_aliases(
        &self,
        request: &ResolveRequest<'_>,
        context: &SyntaxGuard,
        module: &EffectiveModule,
        trail: &mut ResolutionTrail,
        budget: &mut ProjectionBudget,
    ) -> Result<Option<Vec<ResolvedPath>>, ProjectionLimit> {
        budget.consume_work()?;
        let (root, suffix) = split_root(request.written);
        let mut sites = self.alias_sites(request, root, suffix, context, module, budget)?;
        if sites.is_empty() {
            return Ok(None);
        }
        if sites.len() > MAX_BINDING_CANDIDATES {
            return Ok(Some(vec![unresolved(request.written)]));
        }
        let key = ResolutionKey::Alias {
            instance: sites[0].instance,
            name: root.into(),
            scope: sites[0].binding.lexical_scope.clone(),
        };
        let owns_key = trail.insert(key.clone());
        if !owns_key {
            sites.retain(|site| {
                !matches!(
                    site.binding.kind,
                    BindingKind::Import | BindingKind::TypeAlias
                ) || split_root(&site.binding.target).0 != root
            });
            if sites.is_empty() {
                return Ok(None);
            }
        }
        let site_guards = sites
            .iter()
            .map(|site| (site.instance, site.binding.guard.clone()))
            .collect::<Vec<_>>();
        if guards_cover(self, context, &site_guards) && !guards_overlap(self, context, &site_guards)
        {
            for site in &mut sites {
                let base = if site.binding.replacement_macros.is_empty() {
                    site.binding.quality_without_macros
                } else {
                    AnalysisQuality::Unresolved
                };
                site.binding.quality = base.max(self.visibility_quality(
                    &site.binding.visibility,
                    &site.module,
                    &request.mode.consumer,
                ));
            }
        }
        let mut expanded_guards = Vec::new();
        let mut resolved = Vec::new();
        for site in sites {
            let expansion = ResolveRequest {
                instance: request.instance,
                written: request.written,
                scope: request.scope,
                depth: request.depth + 1,
                mode: request.mode.clone(),
                usage: request.usage,
                guard: request.guard.clone(),
            };
            let expanded = self.expand_binding(&site, &expansion, suffix, trail, budget)?;
            if expanded.is_empty() {
                continue;
            }
            expanded_guards.push((site.instance, site.binding.guard.clone()));
            resolved.extend(expanded);
            if resolved.len() > MAX_BINDING_CANDIDATES {
                if owns_key {
                    trail.remove(&key);
                }
                return Ok(Some(vec![unresolved(request.written)]));
            }
        }
        if guards_overlap(self, context, &expanded_guards) {
            for candidate in &mut resolved {
                candidate.quality = AnalysisQuality::Unresolved;
                candidate.requires_projection = true;
                candidate.blocks_completeness = true;
            }
        }
        if owns_key {
            trail.remove(&key);
        }
        Ok(Some(normalize(resolved)))
    }
}

fn guards_overlap(
    bindings: &IncludeBindings,
    context: &SyntaxGuard,
    sites: &[(super::SourceInstanceId, SyntaxGuard)],
) -> bool {
    sites.iter().enumerate().any(|(index, (left_id, left))| {
        sites[index + 1..].iter().any(|(right_id, right)| {
            let (Some(left_source), Some(right_source)) = (
                bindings.instances.get(*left_id),
                bindings.instances.get(*right_id),
            ) else {
                return true;
            };
            if left_source.domain != right_source.domain {
                return true;
            }
            let combined = left.combine(right).combine(context);
            combined.predicate().is_satisfiable() != Some(false)
                && combined
                    .availability_in_domain(&left_source.domain)
                    .is_available()
        })
    })
}

fn guards_cover(
    bindings: &IncludeBindings,
    context: &SyntaxGuard,
    sites: &[(super::SourceInstanceId, SyntaxGuard)],
) -> bool {
    let Some((first, _)) = sites.first() else {
        return false;
    };
    let Some(source) = bindings.instances.get(*first) else {
        return false;
    };
    if sites.iter().any(|(id, _)| {
        bindings
            .instances
            .get(*id)
            .is_none_or(|candidate| candidate.domain != source.domain)
    }) {
        return false;
    }
    let union = SyntaxGuard::from_predicate(super::CfgPredicate::any(
        sites.iter().map(|(_, guard)| guard.predicate()).collect(),
    ));
    union.availability_for_domain(context, &source.domain) == super::GuardAvailability::Exact
}
