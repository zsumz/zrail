//! Textual macro definitions resolve inside exact Cargo and lexical namespaces.

mod catalog;
mod domains;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::macro_definition_candidate::{
    add_include_scope_uncertainty, candidate_order, discard_file_wide_definition_guess,
    local_policy_name, repository_candidate,
};
use super::{
    CompilationDomain, MacroCandidate, MacroDerivation, MacroExpansionFact, SourceInstanceId,
    SourceInstances, SourceSyntax, SyntaxGuard, model::MacroDefinitionFact,
};

const MAX_DEFINITIONS_PER_DOMAIN: usize = 256;
const MAX_VISIBLE_DEFINITIONS: usize = 64;

pub(super) struct MacroDefinitions {
    pub(super) files: BTreeMap<(String, SourceSyntax), Vec<MacroDefinitionFact>>,
    pub(super) packages: BTreeMap<String, PackageOrigin>,
    pub(super) instances: SourceInstances,
    pub(super) inline_module_names:
        BTreeMap<(String, SourceSyntax), BTreeMap<zrail_core::SourceSpan, String>>,
    pub(super) qualified_sites:
        BTreeMap<(super::SourceInstanceId, String), BTreeSet<DefinitionSite>>,
    pub(super) qualified_sites_complete: bool,
    names: BTreeMap<CompilationDomain, BTreeSet<String>>,
    overflowed: BTreeSet<CompilationDomain>,
}

#[derive(Clone)]
pub(super) struct PackageOrigin {
    pub(super) name: String,
    pub(super) directory: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct DefinitionSite {
    pub(super) file: String,
    pub(super) package: String,
    pub(super) directory: String,
    pub(super) name: String,
    pub(super) sha256: String,
}

pub(super) struct Resolution {
    pub(super) sites: BTreeSet<DefinitionSite>,
    pub(super) all_paths_local: bool,
    pub(super) include_scope_uncertain: bool,
    pub(super) definition_exact: bool,
}

impl MacroDefinitions {
    pub(super) fn local_names<'a>(
        &'a self,
        file: &str,
        syntax: SourceSyntax,
        guard: &SyntaxGuard,
    ) -> Option<BTreeSet<&'a str>> {
        let mut names = BTreeSet::new();
        for instance in self.active_instances(file, syntax, guard)? {
            let domain = &self.instances.get(instance)?.domain;
            if self.overflowed.contains(domain) {
                return None;
            }
            names.extend(
                self.names
                    .get(domain)
                    .into_iter()
                    .flatten()
                    .map(String::as_str),
            );
            if names.len() > MAX_DEFINITIONS_PER_DOMAIN {
                return None;
            }
        }
        Some(names)
    }

    pub(super) fn apply(
        &self,
        file: &str,
        syntax: SourceSyntax,
        expansion: &mut MacroExpansionFact,
    ) {
        let Some(instances) = self.active_instances(file, syntax, &expansion.guard) else {
            Self::add_unknown(expansion);
            return;
        };
        if instances.is_empty() {
            Self::add_unknown(expansion);
            return;
        }
        if expansion.name.contains("::") {
            self.apply_qualified(expansion, &instances);
            return;
        }
        let mut sites = BTreeSet::new();
        let mut every_instance_local = !instances.is_empty();
        let mut include_scope_uncertain = false;
        let mut definitions_exact = true;
        for instance in &instances {
            let mut seen = BTreeSet::new();
            let resolution = self.resolve(
                *instance,
                &expansion.name,
                &expansion.lexical_scope,
                expansion.span,
                &mut seen,
            );
            let Some(resolution) = resolution else {
                Self::add_unknown(expansion);
                return;
            };
            every_instance_local &= resolution.all_paths_local;
            include_scope_uncertain |= resolution.include_scope_uncertain;
            definitions_exact &= resolution.definition_exact;
            sites.extend(resolution.sites);
            if sites.len() > MAX_VISIBLE_DEFINITIONS {
                Self::add_unknown(expansion);
                return;
            }
        }
        let policy_name = local_policy_name(expansion);
        if every_instance_local {
            expansion.candidates.clear();
        } else {
            discard_file_wide_definition_guess(expansion);
            if include_scope_uncertain {
                add_include_scope_uncertainty(expansion);
            }
        }
        let candidates = sites
            .into_iter()
            .map(|site| repository_candidate(expansion, &policy_name, site))
            .collect::<Vec<_>>();
        expansion.candidates.extend(candidates);
        self.attach_qualified_definitions(expansion, &instances);
        if !definitions_exact {
            Self::add_unknown(expansion);
        }
        expansion.candidates.sort_by(candidate_order);
        expansion.candidates.dedup();
        expansion.refresh_quality();
    }

    fn apply_qualified(&self, expansion: &mut MacroExpansionFact, instances: &[SourceInstanceId]) {
        let mut include_scope_uncertain = false;
        for instance in instances {
            let mut seen = BTreeSet::new();
            let Some(resolution) = self.resolve(
                *instance,
                &expansion.name,
                &expansion.lexical_scope,
                expansion.span,
                &mut seen,
            ) else {
                Self::add_unknown(expansion);
                return;
            };
            include_scope_uncertain |= resolution.include_scope_uncertain;
        }
        self.attach_qualified_definitions(expansion, instances);
        if include_scope_uncertain {
            add_include_scope_uncertainty(expansion);
            expansion.candidates.sort_by(candidate_order);
            expansion.candidates.dedup();
            expansion.refresh_quality();
        }
    }

    fn collect_names(&mut self) {
        for (_, source) in self.instances.iter() {
            let definitions = self
                .files
                .get(&(source.file.clone(), source.syntax))
                .into_iter()
                .flatten();
            if self.overflowed.contains(&source.domain) {
                continue;
            }
            let names = self.names.entry(source.domain.clone()).or_default();
            for definition in definitions {
                names.insert(definition.name.clone());
            }
            if names.len() > MAX_DEFINITIONS_PER_DOMAIN {
                self.names.remove(&source.domain);
                self.overflowed.insert(source.domain.clone());
            }
        }
    }

    fn add_unknown(expansion: &mut MacroExpansionFact) {
        let mut observation = expansion.observation.clone();
        observation.canonical.clear();
        observation.quality = AnalysisQuality::Unresolved;
        expansion.candidates.push(MacroCandidate::unresolved(
            observation,
            MacroDerivation::LocalDefinition,
        ));
        expansion.refresh_quality();
    }
}
