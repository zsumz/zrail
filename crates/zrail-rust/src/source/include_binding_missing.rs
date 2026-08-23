//! Missing names fall back only when the active Rust namespace is complete.

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    SourceInstanceId,
    include_binding_helpers::{canonical_name, split_root, unresolved},
    include_bindings::{IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{EffectiveModule, LookupMode},
};

impl IncludeBindings {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn missing(
        &self,
        instance: SourceInstanceId,
        written: &str,
        scope: &[SourceSpan],
        crossed_include: bool,
        mode: &LookupMode,
        module: &EffectiveModule,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        if self.namespace_is_opaque(instance, scope, mode.exact_scope(), budget)? {
            let name = if mode.exact_scope() {
                canonical_name(&module.names, written).unwrap_or_else(|| written.into())
            } else {
                written.into()
            };
            return Ok(vec![unresolved(&name)]);
        }
        if mode.exact_scope() {
            return Ok(self.missing_module(instance, written, crossed_include, mode, module));
        }
        Ok(vec![ResolvedPath {
            name: written.into(),
            quality: AnalysisQuality::Exact,
            crossed_include,
            requires_projection: crossed_include,
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
            }];
        }
        if mode.speculative {
            return Vec::new();
        }
        let name = canonical_name(&module.names, written).unwrap_or_else(|| written.into());
        vec![unresolved(&name)]
    }
}
