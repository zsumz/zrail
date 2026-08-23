//! Bounded lookup follows Rust textual scope through exact compilation edges.

use std::collections::BTreeSet;

use zrail_core::SourceSpan;

use super::{
    CompilationDomain, SyntaxGuard,
    macro_definitions::{DefinitionSite, MacroDefinitions, Resolution},
    model::MacroDefinitionFact,
};

const MAX_PARENT_PATHS: usize = 128;

impl MacroDefinitions {
    pub(super) fn resolve(
        &self,
        file: &str,
        domain: &CompilationDomain,
        name: &str,
        scope: &[SourceSpan],
        point: Option<SourceSpan>,
        seen: &mut BTreeSet<(String, CompilationDomain)>,
    ) -> Option<Resolution> {
        let key = (file.to_owned(), domain.clone());
        if !seen.insert(key.clone()) || seen.len() > MAX_PARENT_PATHS {
            return None;
        }
        if self
            .local_definition(file, domain, name, scope, point)
            .ok()?
            .is_some()
        {
            seen.remove(&key);
            return Some(Resolution {
                sites: BTreeSet::from([self.site(file, domain)?]),
                all_paths_local: true,
            });
        }
        let parents = self.parents.get(&key).cloned().unwrap_or_default();
        if parents.len() > MAX_PARENT_PATHS {
            return None;
        }
        let mut sites = BTreeSet::new();
        let mut all_paths_local = !parents.is_empty();
        for edge in parents {
            let resolved = self.resolve(
                &edge.parent,
                domain,
                name,
                &edge.parent_scope,
                edge.span,
                seen,
            )?;
            all_paths_local &= resolved.all_paths_local;
            sites.extend(resolved.sites);
        }
        seen.remove(&key);
        Some(Resolution {
            sites,
            all_paths_local,
        })
    }

    fn local_definition(
        &self,
        file: &str,
        domain: &CompilationDomain,
        name: &str,
        scope: &[SourceSpan],
        point: Option<SourceSpan>,
    ) -> Result<Option<&MacroDefinitionFact>, ()> {
        let point = point.ok_or(())?;
        let definitions = self.files.get(file).ok_or(())?;
        Ok(definitions
            .iter()
            .filter(|definition| definition.name == name)
            .filter(|definition| definition.guard.available_in(domain_guard(domain)))
            .filter(|definition| scope.starts_with(&definition.lexical_scope))
            .filter(|definition| definition.span.is_some_and(|span| before(span, point)))
            .max_by_key(|definition| (definition.lexical_scope.len(), definition.span)))
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

    fn site(&self, file: &str, domain: &CompilationDomain) -> Option<DefinitionSite> {
        let package = self.packages.get(&domain.package)?;
        Some(DefinitionSite {
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
