//! Cargo roots, modules, and include occurrences form exact source instances.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    CompilationDomain, CompilationIncludeEdge, CompilationModuleEdge, LexicalSelfIdentity,
    SyntaxGuard, TraitBoundFact,
};

use super::source_instance_edges::SourceInstanceMetrics;

use inheritance::{InheritedBindings, child_context};

const MAX_SOURCE_INSTANCE_DEPTH: usize = 128;

pub(crate) use super::source_instance_edges::SourceInstanceIssue;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SourceInstanceId(pub(crate) usize);

struct SourceOccurrence {
    file: String,
    syntax: super::SourceSyntax,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CompilationRoot {
    pub(crate) file: String,
    pub(crate) syntax: super::SourceSyntax,
    pub(crate) domain: CompilationDomain,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SourceEntry {
    CargoRoot,
    Module(CompilationModuleEdge),
    Include(CompilationIncludeEdge),
}

#[derive(Clone, Debug)]
pub(crate) struct SourceInstance {
    pub(crate) file: String,
    pub(crate) syntax: super::SourceSyntax,
    pub(crate) domain: CompilationDomain,
    pub(crate) guard: SyntaxGuard,
    pub(crate) generic_types: Vec<String>,
    pub(crate) generic_values: Vec<String>,
    pub(crate) trait_bounds: Vec<TraitBoundFact>,
    pub(crate) current_self: Option<LexicalSelfIdentity>,
    pub(crate) value_shadows: Vec<(String, SyntaxGuard)>,
    pub(crate) parent: Option<SourceInstanceId>,
    pub(crate) entered_from: SourceEntry,
    depth: usize,
}

pub(crate) struct SourceInstances {
    instances: Vec<SourceInstance>,
    by_file: BTreeMap<(String, super::SourceSyntax), Vec<SourceInstanceId>>,
    module_children: BTreeMap<SourceInstanceId, Vec<(CompilationModuleEdge, SourceInstanceId)>>,
    include_children: BTreeMap<SourceInstanceId, Vec<(CompilationIncludeEdge, SourceInstanceId)>>,
    identities: BTreeSet<(String, super::SourceSyntax, CompilationDomain)>,
    derived_limit: usize,
    issues: Vec<SourceInstanceIssue>,
    metrics: SourceInstanceMetrics,
}

impl SourceInstances {
    fn add_child(
        &mut self,
        parent: SourceInstanceId,
        file: String,
        entry: SourceEntry,
    ) -> Option<SourceInstanceId> {
        let parent_instance = &self.instances[parent.0];
        let depth = parent_instance.depth + 1;
        if depth > MAX_SOURCE_INSTANCE_DEPTH {
            let mut chain = self.ancestor_chain(parent);
            chain.push(file.clone());
            self.record_issue(SourceInstanceIssue::DepthLimit { file, depth, chain });
            return None;
        }
        if self.ancestor_contains(parent, &file) {
            let mut chain = self.ancestor_chain(parent);
            chain.push(file);
            self.record_issue(SourceInstanceIssue::Cycle { chain });
            return None;
        }
        let (guard, inherited) = child_context(parent_instance, &entry)?;
        let syntax = match &entry {
            SourceEntry::Module(edge) => edge.child_syntax,
            SourceEntry::Include(edge) => edge.child_syntax,
            SourceEntry::CargoRoot => return None,
        };
        self.push(
            SourceOccurrence { file, syntax },
            parent_instance.domain.clone(),
            Some(parent),
            entry,
            guard,
            inherited,
            depth,
        )
    }

    fn push(
        &mut self,
        occurrence: SourceOccurrence,
        domain: CompilationDomain,
        parent: Option<SourceInstanceId>,
        entered_from: SourceEntry,
        guard: SyntaxGuard,
        inherited: InheritedBindings,
        depth: usize,
    ) -> Option<SourceInstanceId> {
        let SourceOccurrence { file, syntax } = occurrence;
        let identity = (file.clone(), syntax, domain.clone());
        let base = self.identities.insert(identity);
        if !base && self.metrics.derived_contexts >= self.derived_limit {
            self.record_issue(SourceInstanceIssue::DerivedContextLimit {
                used: self.metrics.derived_contexts.saturating_add(1),
                limit: self.derived_limit,
                file,
            });
            return None;
        }
        if base {
            self.metrics.base_contexts = self.metrics.base_contexts.saturating_add(1);
        } else {
            self.metrics.derived_contexts = self.metrics.derived_contexts.saturating_add(1);
        }
        let id = SourceInstanceId(self.instances.len());
        let InheritedBindings {
            generic_types,
            generic_values,
            trait_bounds,
            current_self,
            value_shadows,
        } = inherited;
        self.by_file
            .entry((file.clone(), syntax))
            .or_default()
            .push(id);
        self.instances.push(SourceInstance {
            file,
            syntax,
            domain,
            guard,
            generic_types,
            generic_values,
            trait_bounds,
            current_self,
            value_shadows,
            parent,
            entered_from,
            depth,
        });
        Some(id)
    }
}

#[path = "source_instance_build.rs"]
mod build;

#[path = "source_instance_access.rs"]
mod access;

#[path = "source_instance_inheritance.rs"]
mod inheritance;

#[cfg(test)]
#[path = "source_instance_test.rs"]
mod source_instance_test;
