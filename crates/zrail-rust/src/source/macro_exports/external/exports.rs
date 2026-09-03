//! A bounded syntax pass proves direct macro names exported by one external module.

use std::collections::{BTreeMap, BTreeSet};

#[path = "exports/analyzer.rs"]
mod analyzer;
#[path = "exports/paths.rs"]
mod paths;
#[path = "exports/use_tree.rs"]
mod use_tree;

use syn::{Attribute, Ident, Visibility};

use super::archive::VerifiedPackage;

const MAX_MODULES: usize = 4_096;

#[derive(Clone, Debug, Default)]
pub(in crate::source::macro_resolution::exports) struct ModuleExports {
    pub(in crate::source::macro_resolution::exports) macros: BTreeSet<String>,
    pub(in crate::source::macro_resolution::exports) uncertain: BTreeMap<String, String>,
    pub(in crate::source::macro_resolution::exports) open: Option<String>,
}

#[derive(Debug, Default)]
struct ModuleSurface {
    public: bool,
    bindings: Vec<UseBinding>,
    open: Option<String>,
}

#[derive(Debug)]
struct UseBinding {
    exported: String,
    target: Vec<String>,
    conditional: bool,
}

#[derive(Debug)]
pub(super) struct PackageExports {
    modules: BTreeMap<Vec<String>, ModuleSurface>,
    root_macros: BTreeSet<String>,
    uncertain_root_macros: BTreeSet<String>,
    root_open: Option<String>,
}

impl PackageExports {
    pub(super) fn analyze(package: &VerifiedPackage) -> Result<Self, String> {
        analyzer::analyze(package)
    }

    pub(super) fn module(&self, path: &[String]) -> ModuleExports {
        let Some(surface) = self.modules.get(path) else {
            return ModuleExports {
                open: Some(format!(
                    "checksum-matched crate source does not expose module {:?}",
                    path.join("::")
                )),
                ..ModuleExports::default()
            };
        };
        if !surface.public {
            return ModuleExports {
                open: Some(format!(
                    "checksum-matched crate module {:?} is not unconditionally public",
                    path.join("::")
                )),
                ..ModuleExports::default()
            };
        }
        let mut output = ModuleExports {
            open: surface.open.clone(),
            ..ModuleExports::default()
        };
        let mut exported_names = BTreeSet::new();
        for binding in &surface.bindings {
            if !exported_names.insert(binding.exported.clone()) {
                output.macros.remove(&binding.exported);
                output.uncertain.insert(
                    binding.exported.clone(),
                    "external module contains duplicate public export names".into(),
                );
                continue;
            }
            let target = paths::absolute_target(path, &binding.target);
            let exact_root = target
                .as_deref()
                .is_some_and(|target| target.len() == 1 && self.root_macros.contains(&target[0]));
            if exact_root && !binding.conditional {
                output.macros.insert(binding.exported.clone());
                continue;
            }
            let uncertain_root = target.as_deref().is_some_and(|target| {
                target.len() == 1 && self.uncertain_root_macros.contains(&target[0])
            });
            let reason = if binding.conditional {
                "external macro re-export is conditional".to_owned()
            } else if uncertain_root {
                "external macro definition is conditional".to_owned()
            } else if let Some(reason) = &self.root_open {
                reason.clone()
            } else {
                "external named export is not a proven macro_rules export".to_owned()
            };
            output.uncertain.insert(binding.exported.clone(), reason);
        }
        for name in output.uncertain.keys() {
            output.macros.remove(name);
        }
        if output.open.is_some() {
            output.macros.clear();
        }
        output
    }
}

fn identifier(ident: &Ident) -> String {
    let spelling = ident.to_string();
    spelling.strip_prefix("r#").unwrap_or(&spelling).to_owned()
}

fn conditional(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr"))
}

fn has_attribute(attributes: &[Attribute], name: &str) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident(name))
}

fn is_public(visibility: &Visibility) -> bool {
    matches!(visibility, Visibility::Public(_))
}

#[cfg(test)]
#[path = "exports_test.rs"]
mod exports_test;
