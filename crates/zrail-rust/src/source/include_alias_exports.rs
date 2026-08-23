//! Item includes export only root-scope aliases into their caller namespace.

use std::collections::BTreeSet;

use super::{
    IncludeContext, SourceInstanceId, SyntaxGuard,
    include_binding_resolution::MAX_BINDING_STEPS,
    include_bindings::{BindingSite, IncludeBindings},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
};

impl IncludeBindings {
    pub(super) fn exported_alias_sites(
        &self,
        instance: SourceInstanceId,
        name: &str,
        context: SyntaxGuard,
        seen: &mut BTreeSet<SourceInstanceId>,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<BindingSite>, ProjectionLimit> {
        budget.consume_work()?;
        if !seen.insert(instance) || seen.len() > MAX_BINDING_STEPS {
            return Ok(Vec::new());
        }
        let Some(source) = self.instances.get(instance) else {
            return Ok(Vec::new());
        };
        let mut sites = Vec::new();
        for binding in self
            .files
            .get(&source.file)
            .and_then(|bindings| bindings.named.get(name))
            .into_iter()
            .flatten()
        {
            budget.consume_work()?;
            if binding.name.as_deref() == Some(name)
                && binding.lexical_scope.is_empty()
                && binding.guard.available_in(context)
            {
                sites.push(BindingSite {
                    binding: binding.clone(),
                    instance,
                    crossed_include: true,
                });
            }
        }
        for (edge, child) in self.instances.includes_from(instance) {
            budget.consume_work()?;
            if edge.context == IncludeContext::Items && edge.parent_scope.is_empty() {
                sites.extend(self.exported_alias_sites(*child, name, context, seen, budget)?);
            }
        }
        seen.remove(&instance);
        Ok(sites)
    }
}
