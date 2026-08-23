//! Glob imports project conservatively through bounded include namespaces.

use std::collections::BTreeSet;

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    IncludeContext, SourceEntry, SourceInstanceId, SyntaxGuard,
    include_binding_helpers::join,
    include_binding_resolution::MAX_BINDING_STEPS,
    include_bindings::{BindingSite, IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
};

impl IncludeBindings {
    pub(super) fn expand_glob(
        &self,
        site: &BindingSite,
        written: &str,
        context: SyntaxGuard,
        seen: &mut BTreeSet<(SourceInstanceId, String, Vec<SourceSpan>)>,
        depth: usize,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        budget.consume_work()?;
        Ok(self
            .resolve_aliases(
                site.instance,
                &site.binding.target,
                &site.binding.lexical_scope,
                context,
                seen,
                depth,
                budget,
            )?
            .unwrap_or_else(|| {
                vec![ResolvedPath {
                    name: site.binding.target.clone(),
                    quality: AnalysisQuality::Exact,
                    crossed_include: false,
                }]
            })
            .into_iter()
            .map(|candidate| ResolvedPath {
                name: join(&candidate.name, &format!("::{written}")),
                quality: candidate
                    .quality
                    .max(site.binding.quality)
                    .max(AnalysisQuality::Conservative),
                crossed_include: candidate.crossed_include || site.crossed_include,
            })
            .collect())
    }

    pub(super) fn glob_sites(
        &self,
        instance: SourceInstanceId,
        scope: &[SourceSpan],
        context: SyntaxGuard,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<BindingSite>, ProjectionLimit> {
        let Some(source) = self.instances.get(instance) else {
            return Ok(Vec::new());
        };
        let mut sites = Vec::new();
        for binding in self
            .files
            .get(&source.file)
            .into_iter()
            .flat_map(|bindings| &bindings.globs)
        {
            budget.consume_work()?;
            if binding.name.is_none()
                && scope.starts_with(&binding.lexical_scope)
                && binding.guard.available_in(context)
            {
                sites.push(BindingSite {
                    binding: binding.clone(),
                    instance,
                    crossed_include: false,
                });
            }
        }
        for (edge, child) in self.instances.includes_from(instance) {
            budget.consume_work()?;
            if edge.context == IncludeContext::Items && scope.starts_with(&edge.parent_scope) {
                sites.extend(self.exported_glob_sites(
                    *child,
                    context,
                    &mut BTreeSet::new(),
                    budget,
                )?);
            }
        }
        if let (Some(parent), SourceEntry::Include(edge)) = (source.parent, &source.entered_from) {
            sites.extend(self.glob_sites(parent, &edge.parent_scope, context, budget)?);
        }
        if source.parent.is_some() {
            for site in &mut sites {
                site.crossed_include |= site.instance != instance;
            }
        }
        Ok(sites)
    }

    fn exported_glob_sites(
        &self,
        instance: SourceInstanceId,
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
            .into_iter()
            .flat_map(|bindings| &bindings.globs)
        {
            budget.consume_work()?;
            if binding.name.is_none()
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
                sites.extend(self.exported_glob_sites(*child, context, seen, budget)?);
            }
        }
        seen.remove(&instance);
        Ok(sites)
    }
}
