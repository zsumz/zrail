//! Bounded textual lookup follows exact module and include source instances.

use std::collections::BTreeSet;

use zrail_core::SourceSpan;

use super::{
    CompilationDomain, IncludeContext, SourceEntry, SourceInstanceId, SyntaxGuard,
    macro_definitions::{DefinitionSite, MacroDefinitions, Resolution},
    model::MacroDefinitionFact,
};

const MAX_LOOKUP_STEPS: usize = 256;

impl MacroDefinitions {
    pub(super) fn resolve(
        &self,
        instance: SourceInstanceId,
        name: &str,
        scope: &[SourceSpan],
        point: Option<SourceSpan>,
        seen: &mut BTreeSet<SourceInstanceId>,
    ) -> Option<Resolution> {
        if !seen.insert(instance) || seen.len() > MAX_LOOKUP_STEPS {
            return None;
        }
        let (definition, include_scope_uncertain) = self
            .preceding_definition(instance, name, scope, point?, seen)
            .ok()?;
        if let Some((site, definition_exact)) = definition {
            seen.remove(&instance);
            return Some(Resolution {
                sites: BTreeSet::from([site]),
                all_paths_local: true,
                include_scope_uncertain,
                definition_exact,
            });
        }
        let source = self.instances.get(instance)?;
        let parent = source.parent;
        let entry = source.entered_from.clone();
        let mut resolved = match (parent, entry) {
            (_, SourceEntry::CargoRoot) => Resolution {
                sites: BTreeSet::new(),
                all_paths_local: false,
                include_scope_uncertain: false,
                definition_exact: true,
            },
            (Some(parent), SourceEntry::Module(edge)) => {
                self.resolve(parent, name, &edge.parent_scope, edge.span, seen)?
            }
            (Some(parent), SourceEntry::Include(edge)) => {
                let mut resolved = self.resolve(
                    parent,
                    name,
                    &edge.parent_scope,
                    Some(edge.include_span),
                    seen,
                )?;
                resolved.include_scope_uncertain = true;
                resolved
            }
            _ => return None,
        };
        resolved.include_scope_uncertain |= include_scope_uncertain;
        resolved.definition_exact &= source.guard.is_exact();
        seen.remove(&instance);
        Some(resolved)
    }

    fn preceding_definition(
        &self,
        instance: SourceInstanceId,
        name: &str,
        scope: &[SourceSpan],
        point: SourceSpan,
        seen: &mut BTreeSet<SourceInstanceId>,
    ) -> Result<(Option<(DefinitionSite, bool)>, bool), ()> {
        let source = self.instances.get(instance).ok_or(())?;
        let mut selected = self
            .visible_local_definition(&source.file, &source.domain, name, scope, Some(point))
            .map(|definition| {
                Ok((
                    definition.lexical_scope.len(),
                    definition.span.ok_or(())?,
                    self.site(&source.file, &source.domain)?,
                    source.guard.is_exact() && definition.guard.is_exact(),
                ))
            })
            .transpose()?;
        let mut include_scope_uncertain = false;
        for (edge, child) in self.instances.includes_from(instance) {
            if edge.context != IncludeContext::Items
                || !scope.starts_with(&edge.parent_scope)
                || !before(edge.include_span, point)
            {
                continue;
            }
            include_scope_uncertain = true;
            let Some((site, exact)) = self.exported_definition(*child, name, seen)? else {
                continue;
            };
            let candidate = (
                edge.parent_scope.len(),
                edge.include_span,
                site,
                exact && edge.guard.is_exact(),
            );
            if selected
                .as_ref()
                .is_none_or(|current| (candidate.0, candidate.1) > (current.0, current.1))
            {
                selected = Some(candidate);
            }
        }
        Ok((
            selected.map(|(_, _, site, exact)| (site, exact)),
            include_scope_uncertain,
        ))
    }

    fn exported_definition(
        &self,
        instance: SourceInstanceId,
        name: &str,
        seen: &mut BTreeSet<SourceInstanceId>,
    ) -> Result<Option<(DefinitionSite, bool)>, ()> {
        if !seen.insert(instance) || seen.len() > MAX_LOOKUP_STEPS {
            return Err(());
        }
        let source = self.instances.get(instance).ok_or(())?;
        let mut selected = self
            .visible_local_definition(&source.file, &source.domain, name, &[], None)
            .map(|definition| {
                Ok((
                    definition.span.ok_or(())?,
                    self.site(&source.file, &source.domain)?,
                    source.guard.is_exact() && definition.guard.is_exact(),
                ))
            })
            .transpose()?;
        for (edge, child) in self.instances.includes_from(instance) {
            if edge.context != IncludeContext::Items || !edge.parent_scope.is_empty() {
                continue;
            }
            let Some((site, exact)) = self.exported_definition(*child, name, seen)? else {
                continue;
            };
            let candidate = (edge.include_span, site, exact && edge.guard.is_exact());
            if selected
                .as_ref()
                .is_none_or(|current| candidate.0 > current.0)
            {
                selected = Some(candidate);
            }
        }
        seen.remove(&instance);
        Ok(selected.map(|(_, site, exact)| (site, exact)))
    }

    fn visible_local_definition(
        &self,
        file: &str,
        domain: &CompilationDomain,
        name: &str,
        scope: &[SourceSpan],
        point: Option<SourceSpan>,
    ) -> Option<&MacroDefinitionFact> {
        self.files
            .get(file)?
            .iter()
            .filter(|definition| definition.name == name)
            .filter(|definition| definition.guard.available_in(domain_guard(domain)))
            .filter(|definition| scope.starts_with(&definition.lexical_scope))
            .filter(|definition| {
                definition
                    .span
                    .is_some_and(|span| point.is_none_or(|point| before(span, point)))
            })
            .max_by_key(|definition| (definition.lexical_scope.len(), definition.span))
    }

    pub(super) fn active_instances(
        &self,
        file: &str,
        guard: SyntaxGuard,
    ) -> Option<Vec<SourceInstanceId>> {
        if !self.instances.complete {
            return None;
        }
        Some(
            self.instances
                .for_file(file)
                .iter()
                .copied()
                .filter(|id| {
                    self.instances
                        .get(*id)
                        .is_some_and(|instance| guard.available_in(domain_guard(&instance.domain)))
                })
                .collect(),
        )
    }

    pub(super) fn active_domains(
        &self,
        file: &str,
        guard: SyntaxGuard,
    ) -> Option<Vec<&CompilationDomain>> {
        Some(
            self.domains
                .get(file)?
                .iter()
                .filter(|domain| guard.available_in(domain_guard(domain)))
                .collect(),
        )
    }

    pub(super) fn site(
        &self,
        file: &str,
        domain: &CompilationDomain,
    ) -> Result<DefinitionSite, ()> {
        let package = self.packages.get(&domain.package).ok_or(())?;
        Ok(DefinitionSite {
            file: file.into(),
            package: package.name.clone(),
            directory: package.directory.clone(),
        })
    }
}

fn domain_guard(domain: &CompilationDomain) -> SyntaxGuard {
    SyntaxGuard::for_test_only(domain.mode.enables_cfg_test())
}

fn before(left: SourceSpan, right: SourceSpan) -> bool {
    (left.line, left.column) < (right.line, right.column)
}
