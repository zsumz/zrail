//! Definition catalogs bind each grammar occurrence to its exact source facts.

use std::collections::{BTreeMap, BTreeSet};

use crate::cargo::{CargoWorkspace, Package};
use crate::source::{
    CompilationDomain, CompilationIncludeEdge, CompilationModuleEdge, CompilationRoot, SourceIndex,
    SourceInstances,
};

use super::{MacroDefinitions, PackageOrigin};

impl MacroDefinitions {
    pub(in crate::source) fn collect_with_limit(
        index: &SourceIndex,
        cargo: &CargoWorkspace,
        _domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
        roots: &[CompilationRoot],
        edges: &[CompilationModuleEdge],
        includes: &[CompilationIncludeEdge],
        derived_limit: Option<usize>,
    ) -> Self {
        let mut definitions = Self {
            files: index
                .files
                .iter()
                .map(|file| {
                    (
                        (file.relative.clone(), file.syntax),
                        file.macro_definitions.clone(),
                    )
                })
                .collect(),
            packages: cargo
                .packages
                .iter()
                .map(|package| (package.name.clone(), package_origin(package)))
                .collect(),
            instances: SourceInstances::build_with_limit(roots, edges, includes, derived_limit),
            inline_module_names: super::super::macro_qualified_definition::inline_module_names(
                index,
            ),
            qualified_sites: BTreeMap::new(),
            qualified_sites_complete: true,
            names: BTreeMap::new(),
            overflowed: BTreeSet::new(),
        };
        definitions.collect_names();
        definitions.collect_qualified_sites();
        definitions
    }
}

fn package_origin(package: &Package) -> PackageOrigin {
    PackageOrigin {
        name: package.name.clone(),
        directory: package.directory.clone(),
    }
}
