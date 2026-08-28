//! Qualified local macros bind to one occurrence-specific definition site.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::{
    MacroCandidate, MacroOrigin, SourceEntry, SourceInstanceId, SourceSyntax,
    macro_definitions::{DefinitionSite, MacroDefinitions},
};

const MAX_QUALIFIED_DEFINITION_PATHS: usize = 16_384;

pub(super) fn inline_module_names(
    index: &super::SourceIndex,
) -> BTreeMap<(String, SourceSyntax), BTreeMap<zrail_core::SourceSpan, String>> {
    index
        .files
        .iter()
        .map(|file| {
            (
                (file.relative.clone(), file.syntax),
                file.import_bindings
                    .iter()
                    .filter_map(|binding| match binding.kind {
                        super::BindingKind::Module(super::ModuleBinding::Inline(span)) => {
                            binding.name.as_ref().map(|name| (span, name.clone()))
                        }
                        _ => None,
                    })
                    .collect(),
            )
        })
        .collect()
}

impl MacroDefinitions {
    pub(super) fn collect_qualified_sites(&mut self) {
        let mut sites = BTreeMap::<(SourceInstanceId, String), BTreeSet<DefinitionSite>>::new();
        for ((file, syntax), definitions) in &self.files {
            for instance in self.instances.for_source(file, *syntax) {
                let Some(source) = self.instances.get(*instance) else {
                    self.qualified_sites_complete = false;
                    return;
                };
                if source.guard.availability_in_domain(&source.domain)
                    != super::GuardAvailability::Exact
                {
                    continue;
                }
                for definition in definitions
                    .iter()
                    .filter(|definition| {
                        definition.guard.availability_in_domain(&source.domain)
                            == super::GuardAvailability::Exact
                    })
                    .filter(|definition| {
                        self.definition_is_module_scoped(file, *syntax, definition)
                    })
                {
                    let Some((root, mut names)) = self.module_location(*instance, &[]) else {
                        self.qualified_sites_complete = false;
                        return;
                    };
                    names.extend(self.inline_names(file, *syntax, &definition.lexical_scope));
                    names.push(normalize(&definition.name));
                    let Some(site) = self.site(file, &source.domain, definition).ok() else {
                        self.qualified_sites_complete = false;
                        return;
                    };
                    if sites.len() == MAX_QUALIFIED_DEFINITION_PATHS
                        && !sites.contains_key(&(root, names.join("::")))
                    {
                        self.qualified_sites_complete = false;
                        self.qualified_sites.clear();
                        return;
                    }
                    sites
                        .entry((root, names.join("::")))
                        .or_default()
                        .insert(site);
                }
            }
        }
        self.qualified_sites = sites;
    }

    pub(super) fn attach_qualified_definitions(
        &self,
        expansion: &mut super::MacroExpansionFact,
        instances: &[SourceInstanceId],
    ) {
        if !self.qualified_sites_complete || instances.is_empty() {
            return;
        }
        for candidate in &mut expansion.candidates {
            if candidate.definition.is_some() || !pending_resolvable(candidate) {
                continue;
            }
            let names = candidate
                .policy_names()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let [written] = names.as_slice() else {
                continue;
            };
            let mut selected = BTreeSet::new();
            let mut complete = true;
            for instance in instances {
                let Some((root, module)) =
                    self.module_location(*instance, &expansion.lexical_scope)
                else {
                    complete = false;
                    break;
                };
                let Some(source) = self.instances.get(*instance) else {
                    complete = false;
                    break;
                };
                if !source.guard.is_exact() {
                    complete = false;
                    break;
                }
                let Some(path) = qualify(module, written, source.domain.edition == "2015") else {
                    complete = false;
                    break;
                };
                let Some(sites) = self.qualified_sites.get(&(root, path)) else {
                    complete = false;
                    break;
                };
                let Some(site) = one(sites) else {
                    complete = false;
                    break;
                };
                selected.insert(site.clone());
            }
            if complete && let Some(site) = one(&selected) {
                candidate.definition = Some(site.file.clone());
                candidate.definition_name = Some(site.name.clone());
                candidate.definition_sha256 = Some(site.sha256.clone());
            }
        }
    }

    fn module_location(
        &self,
        instance: SourceInstanceId,
        scope: &[zrail_core::SourceSpan],
    ) -> Option<(SourceInstanceId, Vec<String>)> {
        let source = self.instances.get(instance)?;
        let (root, mut names) = match (&source.parent, &source.entered_from) {
            (None, SourceEntry::CargoRoot) => (instance, Vec::new()),
            (Some(parent), SourceEntry::Module(edge)) => {
                let (root, mut names) = self.module_location(*parent, &edge.parent_scope)?;
                names.push(normalize(&edge.module_name));
                (root, names)
            }
            (Some(parent), SourceEntry::Include(edge)) => {
                self.module_location(*parent, &edge.parent_scope)?
            }
            _ => return None,
        };
        names.extend(self.inline_names(&source.file, source.syntax, scope));
        Some((root, names))
    }

    fn inline_names(
        &self,
        file: &str,
        syntax: SourceSyntax,
        scope: &[zrail_core::SourceSpan],
    ) -> Vec<String> {
        let Some(modules) = self.inline_module_names.get(&(file.into(), syntax)) else {
            return Vec::new();
        };
        scope
            .iter()
            .filter_map(|span| modules.get(span).map(|name| normalize(name)))
            .collect()
    }

    fn definition_is_module_scoped(
        &self,
        file: &str,
        syntax: SourceSyntax,
        definition: &super::model::MacroDefinitionFact,
    ) -> bool {
        let modules = self.inline_module_names.get(&(file.into(), syntax));
        definition
            .lexical_scope
            .iter()
            .all(|span| modules.is_some_and(|modules| modules.contains_key(span)))
    }
}

fn pending_resolvable(candidate: &MacroCandidate) -> bool {
    candidate.observation.quality != AnalysisQuality::Unresolved
        && !candidate.origins.is_empty()
        && candidate
            .origins
            .iter()
            .all(|origin| matches!(origin, MacroOrigin::Pending { .. }))
}

fn qualify(mut module: Vec<String>, written: &str, edition_2015: bool) -> Option<String> {
    let mut segments = written
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(normalize)
        .collect::<Vec<_>>();
    let first = segments.first().map(String::as_str)?;
    match first {
        "crate" => {
            module.clear();
            segments.remove(0);
        }
        "self" => {
            segments.remove(0);
        }
        "super" => {
            while segments.first().is_some_and(|segment| segment == "super") {
                module.pop()?;
                segments.remove(0);
            }
        }
        _ if edition_2015 => module.clear(),
        _ => {}
    }
    module.extend(segments);
    (!module.is_empty()).then(|| module.join("::"))
}

fn one<T>(values: &BTreeSet<T>) -> Option<&T> {
    (values.len() == 1).then(|| values.iter().next()).flatten()
}

fn normalize(segment: &str) -> String {
    segment.strip_prefix("r#").unwrap_or(segment).into()
}
