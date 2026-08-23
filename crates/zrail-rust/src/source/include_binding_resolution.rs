//! Bounded Rust lookup resolves written paths to effective policy identities.

use zrail_core::AnalysisQuality;

use super::{
    SourceInstanceId, SyntaxGuard,
    include_binding_helpers::{normalize, unresolved},
    include_bindings::{IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{LookupMode, ResolutionTrail, ResolutionUsage, ResolveRequest},
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
            ResolveRequest {
                instance,
                written,
                scope,
                depth,
                mode: LookupMode::lexical(module),
                usage,
            },
            trail,
            budget,
        )
    }

    pub(super) fn resolve_in(
        &self,
        request: ResolveRequest<'_>,
        trail: &mut ResolutionTrail,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        let ResolveRequest {
            instance,
            written,
            scope,
            depth,
            mode,
            usage,
        } = request;
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
                    ResolveRequest {
                        instance,
                        written: &crate_path,
                        scope,
                        depth: depth + 1,
                        mode,
                        usage,
                    },
                    trail,
                    budget,
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
                ResolveRequest {
                    instance: location.instance,
                    written: &location.written,
                    scope: &location.scope,
                    depth: depth + 1,
                    mode: LookupMode::explicit_extern(mode.consumer.clone()),
                    usage,
                },
                trail,
                budget,
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
        let request = ResolveRequest {
            instance,
            written,
            scope,
            depth,
            mode: mode.clone(),
            usage,
        };
        let aliases = self.resolve_aliases(&request, context, &module, trail, budget)?;
        let speculative_alias_miss =
            mode.speculative && aliases.as_ref().is_some_and(std::vec::Vec::is_empty);
        if let Some(mut resolved) = aliases
            && !resolved.is_empty()
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
            if speculative_alias_miss {
                return Ok(Vec::new());
            }
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
}
