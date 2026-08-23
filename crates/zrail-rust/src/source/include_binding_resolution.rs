//! Bounded Rust lookup resolves written paths to effective policy identities.

use zrail_core::AnalysisQuality;

use super::{
    SourceInstanceId, SyntaxGuard,
    include_binding_helpers::{normalize, split_root, unresolved},
    include_bindings::{IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{
        EffectiveModule, LookupMode, ResolutionKey, ResolutionTrail, ResolutionUsage,
    },
};

pub(super) const MAX_BINDING_STEPS: usize = 128;
pub(super) const MAX_BINDING_CANDIDATES: usize = 64;

impl IncludeBindings {
    pub(super) fn resolve_written(
        &self,
        instance: SourceInstanceId,
        written: &str,
        scope: &[zrail_core::SourceSpan],
        trail: &mut ResolutionTrail,
        depth: usize,
        budget: &mut ProjectionBudget,
        usage: ResolutionUsage,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        let Some(module) = self.effective_module(instance, scope, budget)? else {
            return Ok(vec![unresolved(written)]);
        };
        self.resolve_in(
            instance,
            written,
            scope,
            trail,
            depth,
            budget,
            LookupMode::lexical(module),
            usage,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn resolve_in(
        &self,
        instance: SourceInstanceId,
        written: &str,
        scope: &[zrail_core::SourceSpan],
        trail: &mut ResolutionTrail,
        depth: usize,
        budget: &mut ProjectionBudget,
        mode: LookupMode,
        usage: ResolutionUsage,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        budget.consume_work()?;
        if depth >= MAX_BINDING_STEPS
            || written.len() > super::include_binding_helpers::MAX_RESOLVED_PATH_BYTES
        {
            return Ok(vec![unresolved(written)]);
        }
        let Some(source) = self.instances.get(instance) else {
            return Ok(vec![unresolved(written)]);
        };
        if let Some(external) = written.strip_prefix("::") {
            if source.domain.edition == "2015" {
                let Some(crate_path) = super::include_binding_helpers::join("crate::", external)
                else {
                    return Ok(vec![unresolved(written)]);
                };
                let mut resolved = self.resolve_in(
                    instance,
                    &crate_path,
                    scope,
                    trail,
                    depth + 1,
                    budget,
                    mode,
                    usage,
                )?;
                for candidate in &mut resolved {
                    candidate.requires_projection = true;
                }
                return Ok(resolved);
            }
            let Some(location) =
                self.resolve_qualifiers(instance, &format!("crate::{external}"), scope, budget)?
            else {
                return Ok(vec![unresolved(written)]);
            };
            if location.unresolved {
                return Ok(vec![unresolved(written)]);
            }
            let mut resolved = self.resolve_in(
                location.instance,
                &location.written,
                &location.scope,
                trail,
                depth + 1,
                budget,
                LookupMode::explicit_extern(mode.consumer.clone()),
                usage,
            )?;
            for candidate in &mut resolved {
                candidate.crossed_include |= location.crossed_include;
                candidate.requires_projection = true;
            }
            return Ok(resolved);
        }
        let context = SyntaxGuard::for_test_only(source.domain.mode.enables_cfg_test());
        let qualified = self.resolve_qualifiers(instance, written, scope, budget)?;
        let (instance, written, scope, crossed_include, mode) = qualified.as_ref().map_or(
            (instance, written, scope, false, mode.clone()),
            |location| {
                (
                    location.instance,
                    location.written.as_str(),
                    location.scope.as_slice(),
                    location.crossed_include,
                    mode.module(),
                )
            },
        );
        if qualified
            .as_ref()
            .is_some_and(|location| location.unresolved)
        {
            return Ok(vec![unresolved(written)]);
        }
        let Some(module) = self.effective_module(instance, scope, budget)? else {
            return Ok(vec![unresolved(written)]);
        };
        if let Some(mut resolved) = self.resolve_aliases(
            instance, written, scope, context, trail, depth, budget, &mode, &module, usage,
        )? && !resolved.is_empty()
        {
            for candidate in &mut resolved {
                candidate.crossed_include |= crossed_include;
                candidate.requires_projection |= qualified.is_some();
            }
            return Ok(resolved);
        }
        let globs = self.glob_sites(instance, scope, context, budget, &mode, &module)?;
        if globs.len() > MAX_BINDING_CANDIDATES {
            return Ok(vec![unresolved(written)]);
        }
        let mut resolved = Vec::new();
        for site in globs {
            resolved.extend(self.expand_glob(&site, written, trail, depth + 1, budget, usage)?);
            if resolved.len() > MAX_BINDING_CANDIDATES {
                return Ok(vec![unresolved(written)]);
            }
        }
        let mut resolved = normalize(resolved);
        if resolved.is_empty() {
            return self.missing(
                instance,
                written,
                scope,
                crossed_include,
                &mode,
                &module,
                budget,
            );
        }
        if self.namespace_is_opaque(instance, scope, mode.exact_scope(), budget)? {
            for candidate in &mut resolved {
                candidate.quality = AnalysisQuality::Unresolved;
                candidate.requires_projection = true;
            }
        }
        for candidate in &mut resolved {
            candidate.crossed_include |= crossed_include;
            candidate.requires_projection |= qualified.is_some();
        }
        Ok(resolved)
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_aliases(
        &self,
        instance: SourceInstanceId,
        written: &str,
        scope: &[zrail_core::SourceSpan],
        context: SyntaxGuard,
        trail: &mut ResolutionTrail,
        depth: usize,
        budget: &mut ProjectionBudget,
        mode: &LookupMode,
        module: &EffectiveModule,
        usage: ResolutionUsage,
    ) -> Result<Option<Vec<ResolvedPath>>, ProjectionLimit> {
        budget.consume_work()?;
        let (root, suffix) = split_root(written);
        let mut sites = self.alias_sites(
            instance, root, suffix, scope, context, budget, mode, module, usage,
        )?;
        if sites.is_empty() {
            return Ok(None);
        }
        if sites.len() > MAX_BINDING_CANDIDATES {
            return Ok(Some(vec![unresolved(written)]));
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
                    super::BindingKind::Import | super::BindingKind::TypeAlias
                ) || split_root(&site.binding.target).0 != root
            });
            if sites.is_empty() {
                return Ok(None);
            }
        }
        let ambiguous = sites.len() > 1;
        let mut resolved = Vec::new();
        for site in sites {
            let mut expanded = self.expand_binding(
                &site,
                written,
                suffix,
                trail,
                depth + 1,
                budget,
                mode,
                usage,
            )?;
            if ambiguous {
                for candidate in &mut expanded {
                    candidate.quality = AnalysisQuality::Unresolved;
                    candidate.requires_projection = true;
                }
            }
            resolved.extend(expanded);
            if resolved.len() > MAX_BINDING_CANDIDATES {
                if owns_key {
                    trail.remove(&key);
                }
                return Ok(Some(vec![unresolved(written)]));
            }
        }
        if owns_key {
            trail.remove(&key);
        }
        Ok(Some(normalize(resolved)))
    }
}
