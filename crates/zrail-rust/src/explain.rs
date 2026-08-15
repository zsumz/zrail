//! Path-scoped architecture guidance for humans and coding agents.

mod evidence;
mod owners;
mod policy;
mod render;

use std::path::Path;

use serde::{Deserialize, Serialize};
use zrail_core::path::{glob_matches, normalize_relative};

use crate::{
    engine::{CheckError, load_model},
    inventory::classify_path,
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
    pub permitted_dependency_layers: Vec<String>,
    pub external_dependencies: Option<String>,
    pub denied_effects: Vec<String>,
    pub denied_symbols: Vec<String>,
    pub denied_methods: Vec<String>,
    pub denied_macros: Vec<String>,
    pub unsafe_code: String,
    pub lint_suppressions: String,
    pub expected_sibling_test: Option<String>,
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
    let budget = policy::budget_for(
        &relative,
        class,
        reachability,
        &model.bundle.contract.source.rust,
    );
    let matching_scopes = model
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
        .collect::<Vec<_>>();
    let scopes = matching_scopes
        .iter()
        .map(|scope| scope.name.clone())
        .collect();
    let capability_owners = owners::for_path(&model.bundle.contract, &relative);
    let call_owners = owners::calls_for_path(&model.bundle.contract, &relative);
    let invariants = evidence::for_path(&model.bundle.contract, &relative);
    let expected_sibling_test = policy::sibling_path(&relative);
    Ok(PathExplanation {
        schema: 2,
        path: relative,
        file_class: format!("{class:?}").to_ascii_lowercase(),
        reachability: reachability.name().into(),
        package: package.map(|package| package.name.clone()),
        layer: layer.map(|layer| layer.name.clone()),
        profiles: layer.map_or_else(Vec::new, |layer| layer.profiles.clone()),
        scopes,
        permitted_dependency_layers: policy::dependency_layers(layer),
        external_dependencies: layer
            .map(|layer| policy::external_mode(layer.dependencies.external).into()),
        denied_effects: policy::denied_effects(&model.bundle.contract, layer),
        denied_symbols: policy::denied_symbols(&matching_scopes),
        denied_methods: model
            .bundle
            .contract
            .source
            .rust
            .hygiene
            .deny_methods
            .clone(),
        denied_macros: model
            .bundle
            .contract
            .source
            .rust
            .hygiene
            .deny_macros
            .clone(),
        unsafe_code: policy::policy_mode(model.bundle.contract.source.rust.hygiene.unsafe_code)
            .into(),
        lint_suppressions: policy::lint_mode(
            model.bundle.contract.source.rust.hygiene.lint_suppressions,
        )
        .into(),
        expected_sibling_test,
        invariants,
        capability_owners,
        call_owners,
        design_target: budget.target,
        hard_ceiling: budget.hard,
        declarative_shape: policy::declarative_shape(
            class,
            model.bundle.contract.source.rust.facades,
            model.bundle.contract.source.rust.entrypoints,
        ),
        module_docs_required: policy::module_docs_required(
            class,
            model.bundle.contract.source.rust.module_docs,
        ),
        sibling_tests_required: matches!(
            model.bundle.contract.source.rust.tests,
            zrail_core::TestMode::Sibling
        ),
    })
}

#[cfg(test)]
#[path = "explain_test.rs"]
mod explain_test;
