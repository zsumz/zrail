//! Inventories reuse authored syntax facts and the exact source bytes already parsed.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{RustInventoryCount, RustInventorySubject, SourceSpan, sha256_hex};

use crate::{
    inventory::RepositoryInventory,
    source::{SourceIndex, SourceSyntax},
};

use super::{
    GovernedRustInventory, RustInventoryInput, RustInventoryOccurrence, select::selected_names,
};

const MAX_OCCURRENCES: usize = 50_000;
const MAX_INPUT_BYTES: usize = 256 * 1024 * 1024;

pub(super) struct Observations<'a> {
    sources: BTreeMap<&'a str, &'a str>,
    methods: BTreeMap<&'a str, Vec<&'a crate::source::ObservedFact>>,
    expressions: BTreeMap<&'a str, Vec<&'a crate::source::ObservedFact>>,
    paths: BTreeMap<&'a str, Vec<&'a crate::source::ObservedFact>>,
    renames: BTreeMap<&'a str, Vec<&'a crate::source::ObservedFact>>,
    impls: BTreeMap<&'a str, Vec<&'a crate::source::ObservedFact>>,
    work: usize,
    occurrences: usize,
    inputs: BTreeMap<String, RustInventoryInput>,
    input_bytes: usize,
}

impl<'a> Observations<'a> {
    pub(super) fn new(inventory: &'a RepositoryInventory, source: &'a SourceIndex) -> Self {
        let mut methods = BTreeMap::<_, Vec<_>>::new();
        let mut expressions = BTreeMap::<_, Vec<_>>::new();
        let mut paths = BTreeMap::<_, Vec<_>>::new();
        let mut renames = BTreeMap::<_, Vec<_>>::new();
        let mut impls = BTreeMap::<_, Vec<_>>::new();
        for file in &source.files {
            if file.syntax == SourceSyntax::Items {
                for (target, authored) in [
                    (&mut methods, &file.authored_methods),
                    (&mut expressions, &file.authored_expressions),
                    (&mut paths, &file.authored_paths),
                    (&mut renames, &file.authored_renames),
                    (&mut impls, &file.authored_impls),
                ] {
                    if let Some(authored) = authored {
                        target
                            .entry(file.relative.as_str())
                            .or_default()
                            .extend(authored);
                    }
                }
            }
        }
        Self {
            sources: inventory
                .rust_files
                .iter()
                .map(|file| (file.relative.as_str(), file.source.as_str()))
                .collect(),
            methods,
            expressions,
            paths,
            renames,
            impls,
            work: 0,
            occurrences: 0,
            inputs: BTreeMap::new(),
            input_bytes: 0,
        }
    }

    pub(super) fn populate(
        &mut self,
        report: &mut GovernedRustInventory,
        paths: &[String],
    ) -> Result<(), String> {
        let subject = &report.policy.subject;
        let names = subject
            .names()
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let facts = match subject {
            RustInventorySubject::WrittenMethods { .. } => &self.methods,
            RustInventorySubject::WrittenExpressionPaths { .. } => &self.expressions,
            RustInventorySubject::WrittenPathsContaining { .. } => &self.paths,
            RustInventorySubject::WrittenImportRenames { .. } => &self.renames,
            RustInventorySubject::WrittenTraitImpls { .. } => &self.impls,
        };
        let mut counts = BTreeMap::<(&str, &str), usize>::new();
        for path in paths {
            let facts = facts.get(path.as_str()).ok_or_else(|| format!(
                "selected file {path:?} has no complete Rust file parse; source exclusions, workspace boundaries, and expression fragments cannot satisfy this inventory"
            ))?;
            let source = self
                .sources
                .get(path.as_str())
                .ok_or_else(|| format!("selected file {path:?} has no bound source input"))?;
            if !self.inputs.contains_key(path) {
                self.input_bytes += source.len();
                if self.input_bytes > MAX_INPUT_BYTES {
                    return Err(format!(
                        "Rust inventories exceed the {MAX_INPUT_BYTES}-byte unique input limit"
                    ));
                }
                self.inputs.insert(
                    path.clone(),
                    RustInventoryInput {
                        path: path.clone(),
                        sha256: sha256_hex(source.as_bytes()),
                        bytes: source.len(),
                    },
                );
            }
            report.inputs.push(self.inputs[path].clone());
            let mut physical = BTreeSet::<(&str, SourceSpan)>::new();
            for fact in facts {
                for name in selected_names(subject, fact, &names, &mut self.work)? {
                    let span = fact.span.ok_or_else(|| {
                        format!("written subject {name:?} in {path:?} has no exact source span")
                    })?;
                    physical.insert((name, span));
                }
            }
            for (name, span) in physical {
                self.occurrences += 1;
                if self.occurrences > MAX_OCCURRENCES {
                    return Err(format!(
                        "Rust inventories exceed the {MAX_OCCURRENCES}-occurrence observation limit"
                    ));
                }
                *counts.entry((path, name)).or_default() += 1;
                report.observed_count += 1;
                if report.occurrence_sample.len() < 16 {
                    report.occurrence_sample.push(RustInventoryOccurrence {
                        path: path.clone(),
                        name: name.into(),
                        span,
                    });
                }
            }
        }
        report.counts = counts
            .into_iter()
            .map(|((path, name), count)| RustInventoryCount {
                path: path.into(),
                name: name.into(),
                count,
            })
            .collect();
        report.occurrences_omitted = report.observed_count - report.occurrence_sample.len();
        Ok(())
    }
}
