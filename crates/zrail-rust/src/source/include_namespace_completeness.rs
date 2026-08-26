//! Opaque item expansion makes an otherwise missing module member unresolved.

use std::collections::BTreeSet;

use zrail_core::SourceSpan;

use super::{
    IncludeContext, SourceEntry, SourceInstanceId,
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum NamespaceOpacity {
    None,
    Authorized,
    Blocking,
}

impl NamespaceOpacity {
    pub(super) const fn is_opaque(self) -> bool {
        !matches!(self, Self::None)
    }

    pub(super) const fn blocks_completeness(self) -> bool {
        matches!(self, Self::Blocking)
    }
}

impl IncludeBindings {
    pub(super) fn namespace_opacity(
        &self,
        instance: SourceInstanceId,
        scope: &[SourceSpan],
        exact_scope: bool,
        budget: &mut ProjectionBudget,
    ) -> Result<NamespaceOpacity, ProjectionLimit> {
        self.namespace_opacity_inner(instance, scope, exact_scope, &mut BTreeSet::new(), budget)
    }

    fn namespace_opacity_inner(
        &self,
        instance: SourceInstanceId,
        scope: &[SourceSpan],
        exact_scope: bool,
        seen: &mut BTreeSet<SourceInstanceId>,
        budget: &mut ProjectionBudget,
    ) -> Result<NamespaceOpacity, ProjectionLimit> {
        budget.consume_work()?;
        if !seen.insert(instance) {
            return Ok(NamespaceOpacity::None);
        }
        let Some(source) = self.instances.get(instance) else {
            return Ok(NamespaceOpacity::Blocking);
        };
        let floor = self.lexical_floor(&source.file, scope, budget)?;
        let mut opacity = self
            .opaque_namespace_scopes
            .get(&source.file)
            .into_iter()
            .flatten()
            .filter(|(opaque, guard, _)| {
                guard.availability_in_domain(&source.domain).is_available()
                    && if exact_scope {
                        opaque == scope
                    } else {
                        opaque.len() >= floor && scope.starts_with(opaque)
                    }
            })
            .map(|(_, _, authorized)| {
                if *authorized {
                    NamespaceOpacity::Authorized
                } else {
                    NamespaceOpacity::Blocking
                }
            })
            .max()
            .unwrap_or(NamespaceOpacity::None);
        for (edge, child) in self.instances.includes_from(instance) {
            budget.consume_work()?;
            let visible = if exact_scope {
                edge.parent_scope == scope
            } else {
                edge.parent_scope.len() >= floor && scope.starts_with(&edge.parent_scope)
            };
            if edge.context == IncludeContext::Items && visible {
                opacity = opacity.max(self.namespace_opacity_inner(
                    *child,
                    &[],
                    exact_scope,
                    seen,
                    budget,
                )?);
            }
        }
        if self.lexical_floor(&source.file, scope, budget)? == 0
            && let (Some(parent), SourceEntry::Include(edge)) =
                (source.parent, &source.entered_from)
        {
            opacity = opacity.max(self.namespace_opacity_inner(
                parent,
                &edge.parent_scope,
                exact_scope,
                seen,
                budget,
            )?);
        }
        seen.remove(&instance);
        Ok(opacity)
    }
}
