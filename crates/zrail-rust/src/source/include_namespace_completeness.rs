//! Opaque item expansion makes an otherwise missing module member unresolved.

use std::collections::BTreeSet;

use zrail_core::SourceSpan;

use super::{
    IncludeContext, SourceEntry, SourceInstanceId,
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
};

impl IncludeBindings {
    pub(super) fn namespace_is_opaque(
        &self,
        instance: SourceInstanceId,
        scope: &[SourceSpan],
        exact_scope: bool,
        budget: &mut ProjectionBudget,
    ) -> Result<bool, ProjectionLimit> {
        self.namespace_is_opaque_inner(instance, scope, exact_scope, &mut BTreeSet::new(), budget)
    }

    fn namespace_is_opaque_inner(
        &self,
        instance: SourceInstanceId,
        scope: &[SourceSpan],
        exact_scope: bool,
        seen: &mut BTreeSet<SourceInstanceId>,
        budget: &mut ProjectionBudget,
    ) -> Result<bool, ProjectionLimit> {
        budget.consume_work()?;
        if !seen.insert(instance) {
            return Ok(false);
        }
        let Some(source) = self.instances.get(instance) else {
            return Ok(true);
        };
        let context = super::SyntaxGuard::for_test_only(source.domain.mode.enables_cfg_test());
        let floor = self.lexical_floor(&source.file, scope, budget)?;
        if self
            .opaque_namespace_scopes
            .get(&source.file)
            .is_some_and(|scopes| {
                scopes.iter().any(|(opaque, guard)| {
                    guard.availability_in(context).is_available()
                        && if exact_scope {
                            opaque == scope
                        } else {
                            opaque.len() >= floor && scope.starts_with(opaque)
                        }
                })
            })
        {
            return Ok(true);
        }
        for (edge, child) in self.instances.includes_from(instance) {
            budget.consume_work()?;
            let visible = if exact_scope {
                edge.parent_scope == scope
            } else {
                edge.parent_scope.len() >= floor && scope.starts_with(&edge.parent_scope)
            };
            if edge.context == IncludeContext::Items
                && visible
                && self.namespace_is_opaque_inner(*child, &[], exact_scope, seen, budget)?
            {
                return Ok(true);
            }
        }
        if self.lexical_floor(&source.file, scope, budget)? == 0
            && let (Some(parent), SourceEntry::Include(edge)) =
                (source.parent, &source.entered_from)
        {
            return self.namespace_is_opaque_inner(
                parent,
                &edge.parent_scope,
                exact_scope,
                seen,
                budget,
            );
        }
        seen.remove(&instance);
        Ok(false)
    }
}
