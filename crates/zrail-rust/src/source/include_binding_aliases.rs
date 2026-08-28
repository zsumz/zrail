//! Import and type-alias targets retain their distinct Rust lookup semantics.

use zrail_core::AnalysisQuality;

use super::super::{
    include_binding_helpers::{join, split_root, unresolved},
    include_bindings::{
        BindingSite, IncludeBindings, ResolvedOrigin, ResolvedPath, ResolvedTerminal,
    },
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{LookupMode, ResolutionTrail, ResolveRequest},
};

impl IncludeBindings {
    pub(super) fn expand_import_binding(
        &self,
        site: &BindingSite,
        request: &ResolveRequest<'_>,
        suffix: &str,
        trail: &mut ResolutionTrail,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        let Some(target) = join(&site.binding.target, suffix) else {
            return Ok(vec![unresolved(request.written)]);
        };
        let Some(target) = self.anchor_target(site, target) else {
            return Ok(vec![unresolved(request.written)]);
        };
        let root = split_root(target.trim_start_matches("::")).0;
        if self.is_extern_root(site.instance, root) {
            return Ok(vec![ResolvedPath {
                name: target,
                quality: AnalysisQuality::Exact,
                crossed_include: false,
                requires_projection: true,
                blocks_completeness: false,
                origin: ResolvedOrigin::External,
                terminal: ResolvedTerminal::Unknown,
            }]);
        }
        self.resolve_in(
            ResolveRequest {
                instance: site.instance,
                written: &target,
                scope: &site.binding.lexical_scope,
                depth: request.depth,
                mode: LookupMode::binding_target(site.module.clone(), true),
                usage: request.usage,
                guard: request.guard.clone(),
                allow_implicit_prelude: false,
            },
            trail,
            budget,
        )
    }

    pub(super) fn expand_type_alias_binding(
        &self,
        site: &BindingSite,
        request: &ResolveRequest<'_>,
        suffix: &str,
        trail: &mut ResolutionTrail,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        let Some(target) = join(&site.binding.target, suffix) else {
            return Ok(vec![unresolved(request.written)]);
        };
        let Some(target) = self.anchor_target(site, target) else {
            return Ok(vec![unresolved(request.written)]);
        };
        let root = split_root(target.trim_start_matches("::")).0;
        if site.binding.generic_types.iter().any(|generic| {
            generic.strip_prefix("r#").unwrap_or(generic) == root.strip_prefix("r#").unwrap_or(root)
        }) {
            return Ok(vec![unresolved(&target)]);
        }
        self.resolve_in(
            ResolveRequest {
                instance: site.instance,
                written: &target,
                scope: &site.binding.lexical_scope,
                depth: request.depth,
                mode: LookupMode::binding_target(site.module.clone(), false),
                usage: request.usage,
                guard: request.guard.clone(),
                allow_implicit_prelude: true,
            },
            trail,
            budget,
        )
    }
}
