//! Glob imports reach a bounded Rust fixed point without escaping module floors.

#[path = "include_glob_exports.rs"]
mod exports;

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    IncludeContext, SourceEntry, SourceInstanceId, SyntaxGuard,
    include_binding_helpers::{join, unresolved},
    include_binding_resolution::MAX_BINDING_CANDIDATES,
    include_bindings::{BindingSite, IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{
        EffectiveModule, LookupMode, ResolutionKey, ResolutionTrail, ResolutionUsage,
        ResolveRequest,
    },
};

impl IncludeBindings {
    pub(super) fn expand_glob(
        &self,
        site: &BindingSite,
        written: &str,
        trail: &mut ResolutionTrail,
        depth: usize,
        budget: &mut ProjectionBudget,
        usage: ResolutionUsage,
        guard: &SyntaxGuard,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        budget.consume_work()?;
        let key = ResolutionKey::Glob {
            instance: site.instance,
            target: site.binding.target.clone(),
            scope: site.binding.lexical_scope.clone(),
        };
        if !trail.insert(key.clone()) {
            return Ok(Vec::new());
        }
        let Some(target) = join(&site.binding.target, &format!("::{written}")) else {
            trail.remove(&key);
            return Ok(vec![unresolved(written)]);
        };
        let Some(target) = self.anchor_target(site, target) else {
            trail.remove(&key);
            return Ok(vec![unresolved(written)]);
        };
        let resolved = self.resolve_in(
            ResolveRequest {
                instance: site.instance,
                written: &target,
                scope: &site.binding.lexical_scope,
                depth,
                mode: LookupMode::glob_target(site.module.clone()),
                usage,
                guard: guard.clone(),
                allow_implicit_prelude: false,
            },
            trail,
            budget,
        )?;
        trail.remove(&key);
        Ok(resolved
            .into_iter()
            .map(|candidate| ResolvedPath {
                name: candidate.name,
                quality: candidate
                    .quality
                    .max(site.binding.quality)
                    .max(AnalysisQuality::Conservative),
                crossed_include: candidate.crossed_include || site.crossed_include,
                requires_projection: true,
                blocks_completeness: candidate.blocks_completeness,
                origin: candidate.origin,
                terminal: candidate.terminal,
            })
            .collect())
    }

    pub(super) fn glob_sites(
        &self,
        instance: SourceInstanceId,
        scope: &[SourceSpan],
        context: &SyntaxGuard,
        budget: &mut ProjectionBudget,
        mode: &LookupMode,
        module: &EffectiveModule,
    ) -> Result<Vec<BindingSite>, ProjectionLimit> {
        let Some(source) = self.instances.get(instance) else {
            return Ok(Vec::new());
        };
        let floor = if mode.exact_scope() {
            scope.len()
        } else {
            self.lexical_floor(instance, scope, budget)?
        };
        let mut sites = Vec::new();
        for binding in self
            .files
            .get(&instance)
            .into_iter()
            .flat_map(|bindings| &bindings.globs)
        {
            budget.consume_work()?;
            let visible = if mode.exact_scope() {
                scope == binding.lexical_scope
            } else {
                binding.lexical_scope.len() >= floor && scope.starts_with(&binding.lexical_scope)
            };
            let availability = binding
                .guard
                .availability_for_domain(context, &source.domain);
            if availability.is_available() && visible {
                let mut binding = binding.clone();
                if availability == super::GuardAvailability::Possible {
                    binding.quality = AnalysisQuality::Unresolved;
                }
                binding.quality = binding.quality.max(self.visibility_quality(
                    &binding.visibility,
                    module,
                    &mode.consumer,
                ));
                sites.push(BindingSite {
                    binding,
                    instance,
                    module: module.clone(),
                    crossed_include: false,
                });
                if sites.len() > MAX_BINDING_CANDIDATES {
                    return Ok(sites);
                }
            }
        }
        for (edge, child) in self.instances.includes_from(instance) {
            budget.consume_work()?;
            let visible = if mode.exact_scope() {
                scope == edge.parent_scope
            } else {
                edge.parent_scope.len() >= floor && scope.starts_with(&edge.parent_scope)
            };
            if edge.context == IncludeContext::Items && visible {
                for mut site in exports::collect(self, *child, context, budget)? {
                    site.binding.quality = site.binding.quality.max(self.visibility_quality(
                        &site.binding.visibility,
                        &site.module,
                        &mode.consumer,
                    ));
                    sites.push(site);
                    if sites.len() > MAX_BINDING_CANDIDATES {
                        return Ok(sites);
                    }
                }
            }
        }
        if floor == 0
            && let (Some(parent), SourceEntry::Include(edge)) =
                (source.parent, &source.entered_from)
        {
            let Some(parent_module) = self.effective_module(parent, &edge.parent_scope, budget)?
            else {
                return Ok(sites);
            };
            for mut site in self.glob_sites(
                parent,
                &edge.parent_scope,
                context,
                budget,
                mode,
                &parent_module,
            )? {
                site.crossed_include = true;
                sites.push(site);
                if sites.len() > MAX_BINDING_CANDIDATES {
                    break;
                }
            }
        }
        Ok(sites)
    }
}
