//! Glob imports project conservatively through bounded include namespaces.

use std::collections::BTreeSet;

use zrail_core::SourceSpan;

use super::{
    IncludeContext, SourceEntry, SourceInstanceId, SyntaxGuard,
    include_binding_resolution::MAX_BINDING_STEPS,
    include_bindings::{BindingSite, IncludeBindings},
};

impl IncludeBindings {
    pub(super) fn glob_sites(
        &self,
        instance: SourceInstanceId,
        scope: &[SourceSpan],
        context: SyntaxGuard,
    ) -> Vec<BindingSite> {
        let Some(source) = self.instances.get(instance) else {
            return Vec::new();
        };
        let mut sites = self
            .files
            .get(&source.file)
            .into_iter()
            .flatten()
            .filter(|binding| {
                binding.name.is_none()
                    && scope.starts_with(&binding.lexical_scope)
                    && binding.guard.available_in(context)
            })
            .cloned()
            .map(|binding| BindingSite {
                binding,
                instance,
                crossed_include: false,
            })
            .collect::<Vec<_>>();
        for (edge, child) in self.instances.includes_from(instance) {
            if edge.context == IncludeContext::Items && scope.starts_with(&edge.parent_scope) {
                sites.extend(self.exported_glob_sites(*child, context, &mut BTreeSet::new()));
            }
        }
        if let (Some(parent), SourceEntry::Include(edge)) = (source.parent, &source.entered_from) {
            sites.extend(self.glob_sites(parent, &edge.parent_scope, context));
        }
        if source.parent.is_some() {
            for site in &mut sites {
                site.crossed_include |= site.instance != instance;
            }
        }
        sites
    }

    fn exported_glob_sites(
        &self,
        instance: SourceInstanceId,
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
                binding.name.is_none()
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
                sites.extend(self.exported_glob_sites(*child, context, seen));
            }
        }
        seen.remove(&instance);
        sites
    }
}
