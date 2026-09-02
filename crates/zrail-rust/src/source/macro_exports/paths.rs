//! Macro export paths resolve inside one logical crate or one Cargo dependency scope.

use std::collections::BTreeSet;

use crate::cargo::{CrateRootAuthority, DependencySource};

use super::super::logical_modules::LogicalModule;
use super::{MacroExports, MacroOrigin, ModuleResolution, normalize};

impl MacroExports {
    pub(super) fn resolve_module(&self, start: &LogicalModule, written: &str) -> ModuleResolution {
        let segments = written
            .split("::")
            .filter(|segment| !segment.is_empty())
            .map(normalize)
            .collect::<Vec<_>>();
        if segments.is_empty() {
            return ModuleResolution::Local {
                modules: BTreeSet::from([start.clone()]),
            };
        }
        if matches!(segments[0], "crate" | "self" | "super") {
            return self.resolve_keyword_path(start, &segments);
        }
        let mut local = if start.domain.edition == "2015" {
            start.root()
        } else {
            start.clone()
        };
        local
            .path
            .extend(segments.iter().map(|segment| (*segment).to_owned()));
        if self.modules.contains(&local) {
            return ModuleResolution::Local {
                modules: BTreeSet::from([local]),
            };
        }
        self.resolve_dependency(start, &segments)
            .unwrap_or(ModuleResolution::Missing)
    }

    fn resolve_keyword_path(&self, start: &LogicalModule, segments: &[&str]) -> ModuleResolution {
        let mut module = match segments[0] {
            "crate" => start.root(),
            "self" => start.clone(),
            "super" => {
                let Some(parent) = start.parent() else {
                    return ModuleResolution::Missing;
                };
                parent
            }
            _ => return ModuleResolution::Missing,
        };
        let mut index = 1;
        while segments[0] == "super" && segments.get(index) == Some(&"super") {
            let Some(parent) = module.parent() else {
                return ModuleResolution::Missing;
            };
            module = parent;
            index += 1;
        }
        module.path.extend(
            segments[index..]
                .iter()
                .map(|segment| (*segment).to_owned()),
        );
        if self.modules.contains(&module) {
            ModuleResolution::Local {
                modules: BTreeSet::from([module]),
            }
        } else {
            ModuleResolution::Missing
        }
    }

    fn resolve_dependency(
        &self,
        start: &LogicalModule,
        segments: &[&str],
    ) -> Option<ModuleResolution> {
        let dependencies = self.package_dependencies.get(&start.domain.package)?;
        let matches = dependencies
            .iter()
            .filter(|dependency| normalize(&dependency.crate_root) == segments[0])
            .collect::<Vec<_>>();
        if matches.is_empty() {
            return None;
        }
        if matches.len() > 1 {
            return Some(ModuleResolution::Unknown(format!(
                "dependency root {:?} has more than one authority",
                segments[0]
            )));
        }
        let dependency = matches[0];
        if dependency.crate_root_authority == CrateRootAuthority::Unresolved {
            return Some(ModuleResolution::External(vec![MacroOrigin::Unresolved]));
        }
        match &dependency.source {
            DependencySource::WorkspaceMember { .. } | DependencySource::RepositoryPath { .. } => {
                let mut roots = self
                    .package_roots
                    .get(&dependency.name)
                    .cloned()
                    .unwrap_or_default();
                roots.retain(|root| {
                    root.domain.feature_world == start.domain.feature_world
                        || root.domain.feature_world.is_none()
                });
                let modules = roots
                    .into_iter()
                    .map(|mut root| {
                        root.path
                            .extend(segments[1..].iter().map(|segment| (*segment).to_owned()));
                        root
                    })
                    .filter(|module| self.modules.contains(module))
                    .collect::<BTreeSet<_>>();
                Some(if modules.is_empty() {
                    ModuleResolution::Unknown(format!(
                        "repository dependency root {:?} has no analyzable macro module",
                        segments[0]
                    ))
                } else {
                    ModuleResolution::Local { modules }
                })
            }
            source @ (DependencySource::Registry { .. } | DependencySource::Git { .. }) => {
                Some(ModuleResolution::External(vec![MacroOrigin::External {
                    package: dependency.name.clone(),
                    source: source.clone(),
                }]))
            }
        }
    }

    pub(super) fn repository_origin(&self, module: &LogicalModule) -> Vec<MacroOrigin> {
        self.package_directories
            .get(&module.domain.package)
            .map_or_else(
                || vec![MacroOrigin::Unresolved],
                |directory| {
                    vec![MacroOrigin::Repository {
                        package: module.domain.package.clone(),
                        directory: directory.clone(),
                    }]
                },
            )
    }
}

pub(super) fn split_target(path: &str) -> (&str, &str) {
    path.rsplit_once("::")
        .map_or(("", path), |(module, name)| (module, normalize(name)))
}
