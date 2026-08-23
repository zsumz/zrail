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
        context: SyntaxGuard,
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
        let mut expanded_sites = 0;
        let mut resolved = Vec::new();
        for site in sites {
            let expansion = ResolveRequest {
                instance: request.instance,
                written: request.written,
                scope: request.scope,
                depth: request.depth + 1,
                mode: request.mode.clone(),
                usage: request.usage,
            };
            let expanded = self.expand_binding(&site, &expansion, suffix, trail, budget)?;
            if expanded.is_empty() {
                continue;
            }
            expanded_sites += 1;
            resolved.extend(expanded);
            if resolved.len() > MAX_BINDING_CANDIDATES {
                if owns_key {
                    trail.remove(&key);
                }
                return Ok(Some(vec![unresolved(request.written)]));
            }
        }
        if expanded_sites > 1 {
            for candidate in &mut resolved {
                candidate.quality = AnalysisQuality::Unresolved;
                candidate.requires_projection = true;
            }
        }
        if owns_key {
            trail.remove(&key);
        }
        Ok(Some(normalize(resolved)))
    }
}
