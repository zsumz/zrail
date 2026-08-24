//! Textual macro definitions resolve inside exact Cargo and lexical namespaces.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use crate::cargo::{CargoWorkspace, Package};

use super::macro_definition_candidate::{
    add_include_scope_uncertainty, candidate_order, discard_file_wide_definition_guess,
    local_policy_name, repository_candidate,
};
use super::{
    CompilationDomain, CompilationIncludeEdge, CompilationModuleEdge, CompilationRoot,
    MacroCandidate, MacroDerivation, MacroExpansionFact, SourceIndex, SourceInstances, SyntaxGuard,
    model::MacroDefinitionFact,
};

const MAX_DEFINITIONS_PER_DOMAIN: usize = 256;
const MAX_VISIBLE_DEFINITIONS: usize = 64;

pub(super) struct MacroDefinitions {
    pub(super) files: BTreeMap<String, Vec<MacroDefinitionFact>>,
    pub(super) packages: BTreeMap<String, PackageOrigin>,
    pub(super) domains: BTreeMap<String, BTreeSet<CompilationDomain>>,
    pub(super) instances: SourceInstances,
    pub(super) inline_module_names: BTreeMap<String, BTreeMap<zrail_core::SourceSpan, String>>,
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
}

pub(super) struct Resolution {
    pub(super) sites: BTreeSet<DefinitionSite>,
    pub(super) all_paths_local: bool,
    pub(super) include_scope_uncertain: bool,
    pub(super) definition_exact: bool,
}

impl MacroDefinitions {
    pub(super) fn collect_with_limit(
        index: &SourceIndex,
        cargo: &CargoWorkspace,
        domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
        roots: &[CompilationRoot],
        edges: &[CompilationModuleEdge],
        includes: &[CompilationIncludeEdge],
        derived_limit: Option<usize>,
    ) -> Self {
        let mut definitions = Self {
            files: index
                .files
                .iter()
                .map(|file| (file.relative.clone(), file.macro_definitions.clone()))
                .collect(),
            packages: cargo
                .packages
                .iter()
                .map(|package| (package.name.clone(), package_origin(package)))
                .collect(),
            domains: domains.clone(),
            instances: SourceInstances::build_with_limit(roots, edges, includes, derived_limit),
            inline_module_names: super::macro_qualified_definition::inline_module_names(index),
            qualified_sites: BTreeMap::new(),
            qualified_sites_complete: true,
            names: BTreeMap::new(),
            overflowed: BTreeSet::new(),
        };
        definitions.collect_names();
        definitions.collect_qualified_sites();
        definitions
    }

    pub(super) fn local_names<'a>(
        &'a self,
        file: &str,
        guard: SyntaxGuard,
    ) -> Option<BTreeSet<&'a str>> {
        let mut names = BTreeSet::new();
        for domain in self.active_domains(file, guard)? {
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

    pub(super) fn apply(&self, file: &str, expansion: &mut MacroExpansionFact) {
        let Some(instances) = self.active_instances(file, expansion.guard) else {
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

    fn apply_qualified(
        &self,
        expansion: &mut MacroExpansionFact,
        instances: &[super::SourceInstanceId],
    ) {
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
        for (file, domains) in &self.domains {
            let definitions = self.files.get(file).into_iter().flatten();
            for domain in domains {
                if self.overflowed.contains(domain) {
                    continue;
                }
                let names = self.names.entry(domain.clone()).or_default();
                for definition in definitions.clone() {
                    names.insert(definition.name.clone());
                }
                if names.len() > MAX_DEFINITIONS_PER_DOMAIN {
                    self.names.remove(domain);
                    self.overflowed.insert(domain.clone());
                }
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

fn package_origin(package: &Package) -> PackageOrigin {
    PackageOrigin {
        name: package.name.clone(),
        directory: package.directory.clone(),
    }
}
