//! Typed bindings recurse through aliases or enter exact local module namespaces.

use super::{
    BindingAnchor, BindingKind, ModuleBinding,
    include_binding_helpers::{canonical_name, join, split_root, unresolved},
    include_bindings::{BindingSite, IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{LookupMode, ResolutionTrail, ResolutionUsage, ResolveRequest},
};

impl IncludeBindings {
    pub(super) fn expand_binding(
        &self,
        site: &BindingSite,
        request: &ResolveRequest<'_>,
        suffix: &str,
        trail: &mut ResolutionTrail,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        budget.consume_work()?;
        let mut resolved = match site.binding.kind {
            BindingKind::Module(module) => {
                self.expand_module(site, module, request, suffix, trail, budget)?
            }
            BindingKind::LocalType
                if request.usage == ResolutionUsage::Call && suffix.is_empty() =>
            {
                Vec::new()
            }
            BindingKind::LocalType | BindingKind::LocalConstructor | BindingKind::LocalValue => {
                let Some(name) = canonical_name(&site.module.names, request.written) else {
                    return Ok(vec![unresolved(request.written)]);
                };
                vec![ResolvedPath {
                    requires_projection: name != request.written || site.crossed_include,
                    name,
                    quality: site.binding.quality,
                    crossed_include: site.crossed_include,
                }]
            }
            BindingKind::OpaqueAlias
                if request.usage == ResolutionUsage::Type && suffix.is_empty() =>
            {
                let name = canonical_name(&site.module.names, request.written)
                    .unwrap_or_else(|| request.written.into());
                vec![ResolvedPath {
                    requires_projection: name != request.written || site.crossed_include,
                    name,
                    quality: site.binding.quality,
                    crossed_include: site.crossed_include,
                }]
            }
            BindingKind::OpaqueAlias => {
                let name = canonical_name(&site.module.names, request.written)
                    .unwrap_or_else(|| request.written.into());
                vec![unresolved(&name)]
            }
            BindingKind::Import | BindingKind::TypeAlias => {
                let Some(target) = join(&site.binding.target, suffix) else {
                    return Ok(vec![unresolved(request.written)]);
                };
                let Some(target) = self.anchor_target(site, target) else {
                    return Ok(vec![unresolved(request.written)]);
                };
                self.resolve_in(
                    ResolveRequest {
                        instance: site.instance,
                        written: &target,
                        scope: &site.binding.lexical_scope,
                        depth: request.depth,
                        mode: LookupMode::binding_target(
                            site.module.clone(),
                            !suffix.is_empty() || site.binding.kind == BindingKind::Import,
                        ),
                        usage: request.usage,
                    },
                    trail,
                    budget,
                )?
            }
            BindingKind::Glob => vec![unresolved(request.written)],
        };
        for candidate in &mut resolved {
            candidate.quality = candidate.quality.max(site.binding.quality);
            candidate.crossed_include |= site.crossed_include;
            candidate.requires_projection |= site.crossed_include
                || request.mode.exact_scope()
                || matches!(
                    site.binding.kind,
                    BindingKind::Import | BindingKind::TypeAlias
                );
        }
        Ok(resolved)
    }

    fn expand_module(
        &self,
        site: &BindingSite,
        module: ModuleBinding,
        request: &ResolveRequest<'_>,
        suffix: &str,
        trail: &mut ResolutionTrail,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        let Some(member) = suffix.strip_prefix("::") else {
            let name = canonical_name(&site.module.names, request.written)
                .unwrap_or_else(|| request.written.into());
            return Ok(vec![ResolvedPath {
                name,
                quality: site.binding.quality,
                crossed_include: site.crossed_include,
                requires_projection: true,
            }]);
        };
        let locations = self.module_locations(site, module, budget)?;
        let [(instance, scope)] = locations.as_slice() else {
            return Ok(vec![unresolved(request.written)]);
        };
        self.resolve_in(
            ResolveRequest {
                instance: *instance,
                written: member,
                scope,
                depth: request.depth,
                mode: request.mode.module(),
                usage: request.usage,
            },
            trail,
            budget,
        )
    }

    fn module_locations(
        &self,
        site: &BindingSite,
        module: ModuleBinding,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<(super::SourceInstanceId, Vec<zrail_core::SourceSpan>)>, ProjectionLimit> {
        match module {
            ModuleBinding::Inline(span) => {
                let mut scope = site.binding.lexical_scope.clone();
                scope.push(span);
                Ok(vec![(site.instance, scope)])
            }
            ModuleBinding::External(span) => {
                let mut locations = Vec::new();
                for (edge, child) in self.instances.modules_from(site.instance) {
                    budget.consume_work()?;
                    if site.binding.name.as_deref() == Some(&edge.module_name)
                        && edge.parent_scope == site.binding.lexical_scope
                        && edge.span == Some(span)
                    {
                        locations.push((*child, Vec::new()));
                        if locations.len()
                            > super::include_binding_resolution::MAX_BINDING_CANDIDATES
                        {
                            break;
                        }
                    }
                }
                Ok(locations)
            }
        }
    }

    pub(super) fn anchor_target(&self, site: &BindingSite, target: String) -> Option<String> {
        let edition_2015 = self
            .instances
            .get(site.instance)
            .is_some_and(|source| source.domain.edition == "2015");
        match site.binding.anchor {
            BindingAnchor::UsePath
                if edition_2015 && !matches!(split_root(&target).0, "crate" | "self" | "super") =>
            {
                join("crate::", &target)
            }
            BindingAnchor::Lexical | BindingAnchor::UsePath => Some(target),
            BindingAnchor::Absolute if edition_2015 => join("crate::", &target),
            BindingAnchor::Absolute | BindingAnchor::ExternRoot => join("::", &target),
            BindingAnchor::CrateRoot => {
                let (_, suffix) = split_root(&target);
                join("crate", suffix)
            }
        }
    }
}
