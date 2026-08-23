//! Item includes export only root-scope aliases into their caller namespace.

use std::collections::BTreeSet;

use super::{
    IncludeContext, SourceInstanceId, SyntaxGuard,
    include_binding_resolution::{MAX_BINDING_CANDIDATES, MAX_BINDING_STEPS},
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
                let Some(module) = self.effective_module(instance, &[], budget)? else {
                    continue;
                };
                sites.push(BindingSite {
                    binding: binding.clone(),
                    instance,
                    module,
                    crossed_include: true,
                });
                if sites.len() > MAX_BINDING_CANDIDATES {
                    break;
                }
            }
        }
        for (edge, child) in self.instances.includes_from(instance) {
            budget.consume_work()?;
            if edge.context == IncludeContext::Items && edge.parent_scope.is_empty() {
                for site in self.exported_alias_sites(*child, name, context, seen, budget)? {
                    sites.push(site);
                    if sites.len() > MAX_BINDING_CANDIDATES {
                        break;
                    }
                }
            }
            if sites.len() > MAX_BINDING_CANDIDATES {
                break;
            }
        }
        seen.remove(&instance);
        Ok(sites)
    }
}
