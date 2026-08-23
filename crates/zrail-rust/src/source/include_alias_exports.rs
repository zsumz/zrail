//! Item includes export only root-scope aliases into their caller namespace.

use std::collections::BTreeSet;

use super::{
    IncludeContext, SourceInstanceId, SyntaxGuard,
    include_binding_resolution::MAX_BINDING_STEPS,
    include_bindings::{BindingSite, IncludeBindings},
};

impl IncludeBindings {
    pub(super) fn exported_alias_sites(
        &self,
        instance: SourceInstanceId,
        name: &str,
        context: SyntaxGuard,
        seen: &mut BTreeSet<SourceInstanceId>,
    ) -> Vec<BindingSite> {
        if !seen.insert(instance) || seen.len() > MAX_BINDING_STEPS {
            return Vec::new();
        }
        let Some(source) = self.instances.get(instance) else {
            return Vec::new();
        };
        let mut sites = self
            .files
            .get(&source.file)
            .into_iter()
            .flatten()
            .filter(|binding| {
                binding.name.as_deref() == Some(name)
                    && binding.lexical_scope.is_empty()
                    && binding.guard.available_in(context)
            })
            .cloned()
            .map(|binding| BindingSite {
                binding,
                instance,
                crossed_include: true,
            })
            .collect::<Vec<_>>();
        for (edge, child) in self.instances.includes_from(instance) {
            if edge.context == IncludeContext::Items && edge.parent_scope.is_empty() {
                sites.extend(self.exported_alias_sites(*child, name, context, seen));
            }
        }
        seen.remove(&instance);
        sites
    }
}
