//! Bounded lexical lookup projects imports through exact include occurrences.

use std::collections::BTreeSet;

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    IncludeContext, SourceEntry, SourceInstanceId, SyntaxGuard,
    include_binding_helpers::{join, normalize, select_site, split_root, unresolved},
    include_bindings::{BindingSite, IncludeBindings, ResolvedPath},
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
    ) -> Vec<ResolvedPath> {
        if depth >= MAX_BINDING_STEPS {
            return vec![unresolved(written)];
        }
        let Some(source) = self.instances.get(instance) else {
            return vec![unresolved(written)];
        };
        let context = SyntaxGuard::for_test_only(source.domain.mode.enables_cfg_test());
        if let Some(resolved) = self.resolve_aliases(instance, written, scope, context, seen, depth)
        {
            return resolved;
        }
        let globs = self.glob_sites(instance, scope, context);
        if globs.len() > MAX_BINDING_CANDIDATES {
            return vec![unresolved(written)];
        }
        if globs.is_empty() {
            return vec![ResolvedPath {
                name: written.into(),
                quality: AnalysisQuality::Exact,
                crossed_include: false,
            }];
        }
        let mut resolved = Vec::new();
        for site in globs {
            resolved.extend(self.expand_glob(&site, written, context, seen, depth + 1));
            if resolved.len() > MAX_BINDING_CANDIDATES {
                return vec![unresolved(written)];
            }
        }
        normalize(resolved)
    }

    fn resolve_aliases(
        &self,
        instance: SourceInstanceId,
        written: &str,
        scope: &[SourceSpan],
        context: SyntaxGuard,
        seen: &mut BTreeSet<(SourceInstanceId, String, Vec<SourceSpan>)>,
        depth: usize,
    ) -> Option<Vec<ResolvedPath>> {
        if depth >= MAX_BINDING_STEPS {
            return Some(vec![unresolved(written)]);
        }
        let (root, suffix) = split_root(written);
        let sites = self.alias_sites(instance, root, scope, context);
        if sites.is_empty() {
            return None;
        }
        if sites.len() > MAX_BINDING_CANDIDATES {
            return Some(vec![unresolved(written)]);
        }
        let key = (instance, root.to_owned(), scope.to_vec());
        if !seen.insert(key.clone()) {
            return Some(vec![unresolved(written)]);
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
                )
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
                return Some(vec![unresolved(written)]);
            }
        }
        seen.remove(&key);
        Some(normalize(resolved))
    }

    fn expand_glob(
        &self,
        site: &BindingSite,
        written: &str,
        context: SyntaxGuard,
        seen: &mut BTreeSet<(SourceInstanceId, String, Vec<SourceSpan>)>,
        depth: usize,
    ) -> Vec<ResolvedPath> {
        self.resolve_aliases(
            site.instance,
            &site.binding.target,
            &site.binding.lexical_scope,
            context,
            seen,
            depth,
        )
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
        .collect()
    }

    fn alias_sites(
        &self,
        instance: SourceInstanceId,
        name: &str,
        scope: &[SourceSpan],
        context: SyntaxGuard,
    ) -> Vec<BindingSite> {
        let mut selected = Vec::new();
        let mut depth = None;
        let Some(source) = self.instances.get(instance) else {
            return Vec::new();
        };
        for binding in self.files.get(&source.file).into_iter().flatten() {
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
            if edge.context != IncludeContext::Items || !scope.starts_with(&edge.parent_scope) {
                continue;
            }
            for mut site in self.exported_alias_sites(*child, name, context, &mut BTreeSet::new()) {
                site.crossed_include = true;
                select_site(&mut selected, &mut depth, edge.parent_scope.len(), site);
            }
        }
        if !selected.is_empty() {
            return selected;
        }
        if let (Some(parent), SourceEntry::Include(edge)) = (source.parent, &source.entered_from) {
            let mut inherited = self.alias_sites(parent, name, &edge.parent_scope, context);
            for site in &mut inherited {
                site.crossed_include = true;
            }
            return inherited;
        }
        Vec::new()
    }
}
