//! Typed bindings recurse through aliases or enter exact local module namespaces.

use super::{
    BindingAnchor, BindingKind, ModuleBinding,
    include_binding_helpers::{canonical_name, join, split_root, unresolved},
    include_bindings::{BindingSite, IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{LookupMode, ResolutionTrail, ResolutionUsage},
};

impl IncludeBindings {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn expand_binding(
        &self,
        site: &BindingSite,
        written: &str,
        suffix: &str,
        trail: &mut ResolutionTrail,
        depth: usize,
        budget: &mut ProjectionBudget,
        mode: &LookupMode,
        usage: ResolutionUsage,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        budget.consume_work()?;
        let mut resolved = match site.binding.kind {
            BindingKind::Module(module) => self.expand_module(
                site, module, written, suffix, trail, depth, budget, mode, usage,
            )?,
            BindingKind::LocalType if usage == ResolutionUsage::Call && suffix.is_empty() => {
                Vec::new()
            }
            BindingKind::LocalType | BindingKind::LocalConstructor | BindingKind::LocalValue => {
                let Some(name) = canonical_name(&site.module.names, written) else {
                    return Ok(vec![unresolved(written)]);
                };
                vec![ResolvedPath {
                    requires_projection: name != written || site.crossed_include,
                    name,
                    quality: site.binding.quality,
                    crossed_include: site.crossed_include,
                }]
            }
            BindingKind::OpaqueAlias => {
                let name =
                    canonical_name(&site.module.names, written).unwrap_or_else(|| written.into());
                vec![unresolved(&name)]
            }
            BindingKind::Import | BindingKind::TypeAlias => {
                let Some(target) = join(&site.binding.target, suffix) else {
                    return Ok(vec![unresolved(written)]);
                };
                let Some(target) = self.anchor_target(site, target) else {
                    return Ok(vec![unresolved(written)]);
                };
                self.resolve_in(
                    site.instance,
                    &target,
                    &site.binding.lexical_scope,
                    trail,
                    depth,
                    budget,
                    LookupMode::binding_target(site.module.clone()),
                    usage,
                )?
            }
            BindingKind::Glob => vec![unresolved(written)],
        };
        for candidate in &mut resolved {
            candidate.quality = candidate.quality.max(site.binding.quality);
            candidate.crossed_include |= site.crossed_include;
            candidate.requires_projection |= site.crossed_include
                || mode.exact_scope()
                || matches!(
                    site.binding.kind,
                    BindingKind::Import | BindingKind::TypeAlias
                );
        }
        Ok(resolved)
    }

    #[allow(clippy::too_many_arguments)]
    fn expand_module(
        &self,
        site: &BindingSite,
        module: ModuleBinding,
        written: &str,
        suffix: &str,
        trail: &mut ResolutionTrail,
        depth: usize,
        budget: &mut ProjectionBudget,
        mode: &LookupMode,
        usage: ResolutionUsage,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        let Some(member) = suffix.strip_prefix("::") else {
            let name =
                canonical_name(&site.module.names, written).unwrap_or_else(|| written.into());
            return Ok(vec![ResolvedPath {
                name,
                quality: site.binding.quality,
                crossed_include: site.crossed_include,
                requires_projection: true,
            }]);
        };
        let locations = self.module_locations(site, module, budget)?;
        let [(instance, scope)] = locations.as_slice() else {
            return Ok(vec![unresolved(written)]);
        };
        self.resolve_in(
            *instance,
            member,
            scope,
            trail,
            depth,
            budget,
            mode.module(),
            usage,
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
