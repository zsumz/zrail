//! Missing names fall back only when the active Rust namespace is complete.

use zrail_core::AnalysisQuality;

use super::{
    SourceInstanceId,
    include_binding_helpers::{canonical_name, opaque, split_root, unresolved},
    include_bindings::{IncludeBindings, ResolvedOrigin, ResolvedPath, ResolvedTerminal},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{EffectiveModule, LookupMode, ResolveRequest},
};

impl IncludeBindings {
    pub(super) fn missing(
        &self,
        request: &ResolveRequest<'_>,
        crossed_include: bool,
        module: &EffectiveModule,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        let opacity = self.namespace_opacity(
            request.instance,
            request.scope,
            request.mode.exact_scope(),
            budget,
        )?;
        if opacity.is_opaque() {
            let name = if request.mode.exact_scope() {
                canonical_name(&module.names, request.written)
                    .unwrap_or_else(|| request.written.into())
            } else {
                request.written.into()
            };
            return Ok(vec![opaque(&name, opacity.blocks_completeness())]);
        }
        if request.mode.exact_scope() {
            return Ok(self.missing_module(
                request.instance,
                request.written,
                crossed_include,
                &request.mode,
                module,
            ));
        }
        if self.extern_prelude_precedes_implicit(request) {
            return Ok(vec![ResolvedPath {
                name: request.written.into(),
                quality: AnalysisQuality::Exact,
                crossed_include,
                requires_projection: crossed_include,
                blocks_completeness: false,
                origin: ResolvedOrigin::External,
                terminal: ResolvedTerminal::Unknown,
            }]);
        }
        if request.allow_implicit_prelude
            && let Some(prelude) = self.implicit_prelude_candidate(
                request.instance,
                request.written,
                request.scope,
                crossed_include,
                &request.mode,
                request.usage,
                &request.guard,
            )
        {
            return Ok(vec![prelude]);
        }
        if request.allow_implicit_prelude
            && super::include_bindings::known_implicit_prelude_name(split_root(request.written).0)
        {
            return Ok(vec![unresolved(request.written)]);
        }
        Ok(vec![ResolvedPath {
            name: request.written.into(),
            quality: AnalysisQuality::Exact,
            crossed_include,
            requires_projection: crossed_include,
            blocks_completeness: false,
            origin: ResolvedOrigin::External,
            terminal: ResolvedTerminal::Unknown,
        }])
    }

    fn missing_module(
        &self,
        instance: SourceInstanceId,
        written: &str,
        crossed_include: bool,
        mode: &LookupMode,
        module: &EffectiveModule,
    ) -> Vec<ResolvedPath> {
        if mode.extern_root()
            || (module.names.is_empty()
                && self
                    .instances
                    .get(instance)
                    .is_some_and(|source| source.domain.edition == "2015")
                && matches!(split_root(written).0, "std" | "core"))
        {
            return vec![ResolvedPath {
                name: written.into(),
                quality: AnalysisQuality::Exact,
                crossed_include,
                requires_projection: true,
                blocks_completeness: false,
                origin: ResolvedOrigin::External,
                terminal: ResolvedTerminal::Unknown,
            }];
        }
        if mode.speculative {
            return Vec::new();
        }
        let name = canonical_name(&module.names, written).unwrap_or_else(|| written.into());
        vec![unresolved(&name)]
    }
}
