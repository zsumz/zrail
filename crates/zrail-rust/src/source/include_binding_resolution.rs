//! Bounded lexical lookup projects imports through exact include occurrences.

use std::collections::BTreeSet;

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    IncludeContext, SourceEntry, SourceInstanceId, SyntaxGuard,
    include_binding_helpers::{join, normalize, select_site, split_root, unresolved},
    include_bindings::{BindingSite, IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
};

pub(super) const MAX_BINDING_STEPS: usize = 128;
const MAX_BINDING_CANDIDATES: usize = 64;

impl IncludeBindings {
    pub(super) fn resolve_written(
        &self,
        instance: SourceInstanceId,
        written: &str,
        scope: &[SourceSpan],
        seen: &mut BTreeSet<(SourceInstanceId, String, Vec<SourceSpan>)>,
        depth: usize,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
        budget.consume_work()?;
        if depth >= MAX_BINDING_STEPS {
            return Ok(vec![unresolved(written)]);
        }
        let Some(source) = self.instances.get(instance) else {
            return Ok(vec![unresolved(written)]);
        };
        let context = SyntaxGuard::for_test_only(source.domain.mode.enables_cfg_test());
        let qualified = self.resolve_qualifiers(instance, written, scope, budget)?;
        let (instance, written, scope, crossed_include) =
            qualified
                .as_ref()
                .map_or((instance, written, scope, false), |location| {
                    (
                        location.instance,
                        location.written.as_str(),
                        location.scope.as_slice(),
                        location.crossed_include,
                    )
                });
        if qualified
            .as_ref()
            .is_some_and(|location| location.unresolved)
        {
            return Ok(vec![unresolved(written)]);
        }
        if let Some(mut resolved) =
            self.resolve_aliases(instance, written, scope, context, seen, depth, budget)?
        {
            for candidate in &mut resolved {
                candidate.crossed_include |= crossed_include;
            }
            return Ok(resolved);
        }
        let globs = self.glob_sites(instance, scope, context, budget)?;
        if globs.len() > MAX_BINDING_CANDIDATES {
            return Ok(vec![unresolved(written)]);
        }
        if globs.is_empty() {
            return Ok(vec![ResolvedPath {
                name: written.into(),
                quality: AnalysisQuality::Exact,
                crossed_include,
            }]);
        }
        let mut resolved = Vec::new();
        for site in globs {
            resolved.extend(self.expand_glob(&site, written, context, seen, depth + 1, budget)?);
            if resolved.len() > MAX_BINDING_CANDIDATES {
                return Ok(vec![unresolved(written)]);
            }
        }
        let mut resolved = normalize(resolved);
        for candidate in &mut resolved {
            candidate.crossed_include |= crossed_include;
        }
        Ok(resolved)
    }

    pub(super) fn resolve_aliases(
        &self,
        instance: SourceInstanceId,
        written: &str,
        scope: &[SourceSpan],
        context: SyntaxGuard,
        seen: &mut BTreeSet<(SourceInstanceId, String, Vec<SourceSpan>)>,
        depth: usize,
        budget: &mut ProjectionBudget,
    ) -> Result<Option<Vec<ResolvedPath>>, ProjectionLimit> {
        budget.consume_work()?;
        if depth >= MAX_BINDING_STEPS {
            return Ok(Some(vec![unresolved(written)]));
        }
        let (root, suffix) = split_root(written);
        let sites = self.alias_sites(instance, root, scope, context, budget)?;
        if sites.is_empty() {
            return Ok(None);
        }
        if sites.len() > MAX_BINDING_CANDIDATES {
            return Ok(Some(vec![unresolved(written)]));
        }
        let key = (instance, root.to_owned(), scope.to_vec());
        if !seen.insert(key.clone()) {
            return Ok(Some(vec![unresolved(written)]));
        }
        let mut resolved = Vec::new();
        for site in sites {
            let target = join(&site.binding.target, suffix);
            let target_root = split_root(&target).0;
            let expanded = if target_root == root {
                vec![ResolvedPath {
                    name: target,
                    quality: site.binding.quality,
                    crossed_include: site.crossed_include,
                }]
            } else {
                self.resolve_aliases(
                    site.instance,
                    &target,
                    &site.binding.lexical_scope,
                    context,
                    seen,
                    depth + 1,
                    budget,
                )?
                .unwrap_or_else(|| {
                    vec![ResolvedPath {
                        name: target,
                        quality: AnalysisQuality::Exact,
                        crossed_include: false,
                    }]
                })
                .into_iter()
                .map(|mut candidate| {
                    candidate.quality = candidate.quality.max(site.binding.quality);
                    candidate.crossed_include |= site.crossed_include;
                    candidate
                })
                .collect()
            };
            resolved.extend(expanded);
            if resolved.len() > MAX_BINDING_CANDIDATES {
                seen.remove(&key);
                return Ok(Some(vec![unresolved(written)]));
            }
        }
        seen.remove(&key);
        Ok(Some(normalize(resolved)))
    }

    fn alias_sites(
        &self,
        instance: SourceInstanceId,
        name: &str,
        scope: &[SourceSpan],
        context: SyntaxGuard,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<BindingSite>, ProjectionLimit> {
        let mut selected = Vec::new();
        let mut depth = None;
        let Some(source) = self.instances.get(instance) else {
            return Ok(Vec::new());
        };
        for binding in self
            .files
            .get(&source.file)
            .and_then(|bindings| bindings.named.get(name))
            .into_iter()
            .flatten()
        {
            budget.consume_work()?;
            if binding.name.as_deref() == Some(name)
                && binding.guard.available_in(context)
                && scope.starts_with(&binding.lexical_scope)
            {
                select_site(
                    &mut selected,
                    &mut depth,
                    binding.lexical_scope.len(),
                    BindingSite {
                        binding: binding.clone(),
                        instance,
                        crossed_include: false,
                    },
                );
            }
        }
        for (edge, child) in self.instances.includes_from(instance) {
            budget.consume_work()?;
            if edge.context != IncludeContext::Items || !scope.starts_with(&edge.parent_scope) {
                continue;
            }
            for mut site in
                self.exported_alias_sites(*child, name, context, &mut BTreeSet::new(), budget)?
            {
                site.crossed_include = true;
                select_site(&mut selected, &mut depth, edge.parent_scope.len(), site);
            }
        }
        if !selected.is_empty() {
            return Ok(selected);
        }
        if let (Some(parent), SourceEntry::Include(edge)) = (source.parent, &source.entered_from) {
            let mut inherited =
                self.alias_sites(parent, name, &edge.parent_scope, context, budget)?;
            for site in &mut inherited {
                site.crossed_include = true;
            }
            return Ok(inherited);
        }
        Ok(Vec::new())
    }
}
