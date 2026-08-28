//! Path-scoped architecture guidance for humans and coding agents.

mod evidence;
mod macro_authority;
mod model;
mod owners;
mod policy;
mod render;

use std::path::Path;

use zrail_core::{glob_matches, normalize_relative};

use crate::{
    engine::{CheckError, load_model},
    inventory::classify_path,
    source::Reachability,
};

pub use model::{
    CallOwnerExplanation, CapabilityOwnerExplanation, ItemMacroAuthorityExplanation,
    MacroInvocationExplanation, PathExplanation,
};

/// Resolves the effective architecture policy for one repository-relative path.
///
/// `config` may be relative to `root`. `path` must be repository-relative and is
/// normalized before matching; absolute paths and parent traversal are rejected.
/// The operation reads repository data but does not write files or execute code.
pub fn explain_path(
    root: &Path,
    config: &Path,
    path: &Path,
) -> Result<PathExplanation, CheckError> {
    let model = load_model(root, config)?;
    let relative = normalize_relative(path).map_err(CheckError::from_message)?;
    let class = classify_path(&relative, &model.bundle.contract.source.rust.generated);
    let file_role = crate::source_policy::effective_file_role(
        &relative,
        class,
        &model.bundle.contract.source.rust,
    );
    let reachability = model
        .source
        .files
        .iter()
        .filter(|file| file.relative == relative)
        .fold(Reachability::UNREACHABLE, |reachability, file| {
            reachability.join(file.reachability)
        });
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
    let budget = crate::source_policy::budget_for(
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
    let sibling_tests_required = matches!(
        model.bundle.contract.source.rust.tests,
        zrail_core::TestMode::Sibling
    );
    let expected_sibling_test = sibling_tests_required
        .then(|| policy::sibling_path(&relative))
        .flatten();
    let macro_invocations = macro_authority::invocations(&model, &relative);
    let item_macro_authorities = macro_authority::item_authorities(&model, &relative);
    Ok(PathExplanation {
        schema: 2,
        path: relative,
        file_class: crate::source_policy::role_name(class).into(),
        inferred_file_role: crate::source_policy::role_name(file_role.inferred).into(),
        effective_file_role: crate::source_policy::role_name(file_role.effective).into(),
        file_role_reason: file_role.reason.map(str::to_owned),
        reachability: reachability.name(),
        package: package.map(|package| package.name.clone()),
        layer: layer.map(|layer| layer.name.clone()),
        profiles: layer.map_or_else(Vec::new, |layer| layer.profiles.clone()),
        profile_reachability: policy::profile_reachability(&model.bundle.contract, layer),
        scopes,
        permitted_dependency_layers: policy::dependency_layers(layer),
        external_dependencies: layer
            .map(|layer| policy::external_mode(layer.dependencies.external).into()),
        denied_effects: policy::denied_effects(&model.bundle.contract, layer),
        denied_syntax: policy::denied_syntax(&model.bundle.contract, layer),
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
        glob_imports: policy::glob_import_mode(
            model.bundle.contract.source.rust.hygiene.glob_imports,
        )
        .into(),
        macro_expansion: policy::macro_mode(model.bundle.contract.source.rust.macros.mode).into(),
        allowed_macro_expansions: model
            .bundle
            .contract
            .source
            .rust
            .macros
            .allow
            .iter()
            .map(|allowed| allowed.name.clone())
            .collect(),
        opaque_macro_inputs: model
            .bundle
            .contract
            .source
            .rust
            .macros
            .allow
            .iter()
            .filter(|allowed| allowed.inputs == zrail_core::MacroInputMode::Opaque)
            .map(|allowed| allowed.name.clone())
            .collect(),
        async_closed_macro_expansions: model
            .bundle
            .contract
            .source
            .rust
            .macros
            .allow
            .iter()
            .filter(|allowed| allowed.async_syntax == zrail_core::MacroAsyncSyntax::None)
            .map(|allowed| allowed.name.clone())
            .collect(),
        content_bound_macro_implementations: macro_authority::implementations(&model),
        macro_invocations,
        item_macro_authorities,
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
        design_target: budget.map(|budget| budget.target),
        hard_ceiling: budget.map(|budget| budget.hard),
        declarative_shape: policy::declarative_shape(
            file_role.effective,
            model.bundle.contract.source.rust.facades,
            model.bundle.contract.source.rust.entrypoints,
        ),
        module_docs_required: policy::module_docs_required(
            class,
            model.bundle.contract.source.rust.module_docs,
        ),
        sibling_tests_required,
    })
}

#[cfg(test)]
#[path = "explain_test.rs"]
mod explain_test;
