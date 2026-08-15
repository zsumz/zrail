//! Path-scoped architecture guidance for humans and coding agents.

mod evidence;
mod owners;

use std::path::Path;

use serde::{Deserialize, Serialize};
use zrail_core::{
    Budget,
    path::{glob_matches, normalize_relative},
};

use crate::{
    engine::{CheckError, load_model},
    inventory::{FileClass, classify_path, under_root},
    source::Reachability,
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PathExplanation {
    pub schema: u64,
    pub path: String,
    pub file_class: String,
    pub reachability: String,
    pub package: Option<String>,
    pub layer: Option<String>,
    pub profiles: Vec<String>,
    pub scopes: Vec<String>,
    pub invariants: Vec<String>,
    pub capability_owners: Vec<CapabilityOwnerExplanation>,
    pub call_owners: Vec<CallOwnerExplanation>,
    pub design_target: usize,
    pub hard_ceiling: usize,
    pub declarative_shape: Option<bool>,
    pub module_docs_required: bool,
    pub sibling_tests_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityOwnerExplanation {
    pub name: String,
    pub capability: String,
    pub allow: Vec<String>,
    pub allowed_here: bool,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CallOwnerExplanation {
    pub name: String,
    pub call: String,
    pub allow: Vec<String>,
    pub allowed_here: bool,
    pub reason: String,
}

impl PathExplanation {
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut value| {
            value.push('\n');
            value
        })
    }

    pub fn human(&self) -> String {
        format!(
            concat!(
                "path: {}\n",
                "class: {}\n",
                "reachability: {}\n",
                "package: {}\n",
                "layer: {}\n",
                "profiles: {}\n",
                "scopes: {}\n",
                "invariants: {}\n",
                "capability owners: {}\n",
                "call owners: {}\n",
                "budget: target {}, hard {}\n",
                "declarative shape: {}\n",
                "module docs: {}\n",
                "sibling tests: {}\n",
            ),
            self.path,
            self.file_class,
            self.reachability,
            self.package.as_deref().unwrap_or("<none>"),
            self.layer.as_deref().unwrap_or("<none>"),
            display_list(&self.profiles),
            display_list(&self.scopes),
            display_list(&self.invariants),
            owners::display(&self.capability_owners),
            owners::display_calls(&self.call_owners),
            self.design_target,
            self.hard_ceiling,
            display_optional_bool(self.declarative_shape),
            self.module_docs_required,
            self.sibling_tests_required
        )
    }
}

pub fn explain_path(
    root: &Path,
    config: &Path,
    path: &Path,
) -> Result<PathExplanation, CheckError> {
    let model = load_model(root, config)?;
    let relative = normalize_relative(path).map_err(CheckError::from_message)?;
    let class = classify_path(&relative, &model.bundle.contract.source.rust.generated);
    let reachability = model
        .source
        .files
        .iter()
        .find(|file| file.relative == relative)
        .map_or(Reachability::Unreachable, |file| file.reachability);
    let package = model
        .cargo
        .packages
        .iter()
        .filter(|package| package.contains_file(&relative))
        .max_by_key(|package| package.directory.len());
    let layer = package.and_then(|package| {
        model.bundle.contract.layers.iter().find(|layer| {
            layer
                .packages
                .iter()
                .any(|pattern| glob_matches(pattern, &package.name))
        })
    });
    let budget = budget_for(
        &relative,
        class,
        reachability,
        &model.bundle.contract.source.rust,
    );
    let scopes = model
        .bundle
        .contract
        .scopes
        .iter()
        .filter(|scope| {
            scope
                .include
                .iter()
                .any(|pattern| glob_matches(pattern, &relative))
                && !scope
                    .exclude
                    .iter()
                    .any(|pattern| glob_matches(pattern, &relative))
        })
        .map(|scope| scope.name.clone())
        .collect();
    let capability_owners = owners::for_path(&model.bundle.contract, &relative);
    let call_owners = owners::calls_for_path(&model.bundle.contract, &relative);
    let invariants = evidence::for_path(&model.bundle.contract, &relative);
    Ok(PathExplanation {
        schema: 1,
        path: relative,
        file_class: format!("{class:?}").to_ascii_lowercase(),
        reachability: reachability.name().into(),
        package: package.map(|package| package.name.clone()),
        layer: layer.map(|layer| layer.name.clone()),
        profiles: layer.map_or_else(Vec::new, |layer| layer.profiles.clone()),
        scopes,
        invariants,
        capability_owners,
        call_owners,
        design_target: budget.target,
        hard_ceiling: budget.hard,
        declarative_shape: declarative_shape(
            class,
            model.bundle.contract.source.rust.facades,
            model.bundle.contract.source.rust.entrypoints,
        ),
        module_docs_required: module_docs_required(
            class,
            model.bundle.contract.source.rust.module_docs,
        ),
        sibling_tests_required: matches!(
            model.bundle.contract.source.rust.tests,
            zrail_core::TestMode::Sibling
        ),
    })
}

fn declarative_shape(
    class: FileClass,
    facades: zrail_core::FacadeMode,
    entrypoints: zrail_core::FacadeMode,
) -> Option<bool> {
    match class {
        FileClass::Facade => Some(facades == zrail_core::FacadeMode::Declarative),
        FileClass::EntryPoint => Some(entrypoints == zrail_core::FacadeMode::Declarative),
        _ => None,
    }
}

fn module_docs_required(class: FileClass, mode: zrail_core::ModuleDocsMode) -> bool {
    class != FileClass::Generated && mode == zrail_core::ModuleDocsMode::Required
}

fn budget_for(
    path: &str,
    class: FileClass,
    reachability: Reachability,
    rust: &zrail_core::RustSourceContract,
) -> Budget {
    if class != FileClass::Generated && reachability == Reachability::TestOnly {
        return rust.size.test;
    }
    match class {
        FileClass::Facade => rust.size.facade,
        FileClass::Implementation | FileClass::Test => rust.size.implementation,
        FileClass::Auxiliary | FileClass::EntryPoint => rust.size.auxiliary,
        FileClass::Generated => rust
            .generated
            .iter()
            .find(|generated| under_root(path, &generated.root))
            .map_or(rust.size.implementation, |generated| Budget {
                target: generated.target,
                hard: generated.hard,
            }),
    }
}

fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        "<none>".into()
    } else {
        values.join(", ")
    }
}

fn display_optional_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "<not applicable>",
    }
}

#[cfg(test)]
#[path = "explain_test.rs"]
mod explain_test;
