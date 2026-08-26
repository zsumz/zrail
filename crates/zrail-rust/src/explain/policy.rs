//! Effective path policy rendered as stable agent-facing vocabulary.

use std::{collections::BTreeSet, path::Path};

use zrail_core::{
    Contract, Effect, ExternalDependencyMode, FacadeMode, GlobImportMode, LayerContract,
    LintSuppressionMode, MacroExpansionMode, ModuleDocsMode, PolicyMode, ScopeContract,
};

use crate::inventory::FileClass;

pub(super) fn dependency_layers(layer: Option<&LayerContract>) -> Vec<String> {
    let Some(layer) = layer else {
        return Vec::new();
    };
    std::iter::once(&layer.name)
        .chain(&layer.may_depend_on)
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) const fn external_mode(mode: ExternalDependencyMode) -> &'static str {
    match mode {
        ExternalDependencyMode::Allow => "allow",
        ExternalDependencyMode::Locked => "locked",
        ExternalDependencyMode::None => "none",
    }
}

pub(super) fn denied_effects(contract: &Contract, layer: Option<&LayerContract>) -> Vec<String> {
    let Some(layer) = layer else {
        return Vec::new();
    };
    layer
        .profiles
        .iter()
        .filter_map(|name| contract.profiles.get(name))
        .flat_map(|profile| profile.effects.deny.iter().copied())
        .map(effect_name)
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn denied_syntax(contract: &Contract, layer: Option<&LayerContract>) -> Vec<String> {
    let Some(layer) = layer else {
        return Vec::new();
    };
    layer
        .profiles
        .iter()
        .filter_map(|name| contract.profiles.get(name))
        .flat_map(|profile| profile.syntax.deny.iter().copied())
        .map(crate::rules::async_syntax_name)
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn profile_reachability(
    contract: &Contract,
    layer: Option<&LayerContract>,
) -> Vec<String> {
    let Some(layer) = layer else {
        return Vec::new();
    };
    layer
        .profiles
        .iter()
        .filter_map(|name| {
            contract.profiles.get(name).map(|profile| {
                let reachability = match profile.reachability {
                    zrail_core::PolicyReachability::All => "all files and facts",
                    zrail_core::PolicyReachability::Production => {
                        "production files and ordinary facts"
                    }
                };
                format!("{name}: {reachability}")
            })
        })
        .collect()
}

pub(super) fn denied_symbols(scopes: &[&ScopeContract]) -> Vec<String> {
    scopes
        .iter()
        .flat_map(|scope| scope.symbols.deny.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) const fn policy_mode(mode: PolicyMode) -> &'static str {
    match mode {
        PolicyMode::Allow => "allow",
        PolicyMode::Deny => "deny",
    }
}

pub(super) const fn lint_mode(mode: LintSuppressionMode) -> &'static str {
    match mode {
        LintSuppressionMode::Allow => "allow",
        LintSuppressionMode::Reasoned => "reasoned",
        LintSuppressionMode::Deny => "deny",
    }
}

pub(super) const fn glob_import_mode(mode: GlobImportMode) -> &'static str {
    match mode {
        GlobImportMode::Allow => "allow",
        GlobImportMode::FacadeReexportsOnly => "facade-reexports-only",
        GlobImportMode::Deny => "deny",
    }
}

pub(super) const fn macro_mode(mode: MacroExpansionMode) -> &'static str {
    match mode {
        MacroExpansionMode::Allow => "allow",
        MacroExpansionMode::DenyUnreviewed => "deny-unreviewed",
    }
}

pub(super) fn sibling_path(path: &str) -> Option<String> {
    let path = Path::new(path);
    let name = path.file_name()?.to_str()?;
    if !path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
        || name.ends_with("_test.rs")
    {
        return None;
    }
    let stem = path.file_stem()?.to_str()?;
    let sibling = format!("{stem}_test.rs");
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or(Some(sibling.clone()), |parent| {
            parent.to_str().map(|parent| format!("{parent}/{sibling}"))
        })
}

pub(super) fn declarative_shape(
    class: FileClass,
    facades: FacadeMode,
    entrypoints: FacadeMode,
) -> Option<bool> {
    match class {
        FileClass::Facade => Some(facades == FacadeMode::Declarative),
        FileClass::EntryPoint => Some(entrypoints == FacadeMode::Declarative),
        _ => None,
    }
}

pub(super) fn module_docs_required(class: FileClass, mode: ModuleDocsMode) -> bool {
    class != FileClass::Generated && mode == ModuleDocsMode::Required
}

const fn effect_name(effect: Effect) -> &'static str {
    match effect {
        Effect::Filesystem => "filesystem",
        Effect::CompileFilesystem => "compile-filesystem",
        Effect::Network => "network",
        Effect::Process => "process",
        Effect::Synchronization => "synchronization",
        Effect::Thread => "thread",
        Effect::WallClock => "wall-clock",
        Effect::AsyncRuntime => "async-runtime",
        Effect::Database => "database",
        Effect::ContainerRuntime => "container-runtime",
        Effect::Environment => "environment",
        Effect::CompileEnvironment => "compile-environment",
        Effect::Randomness => "randomness",
    }
}

#[cfg(test)]
#[path = "policy_test.rs"]
mod policy_test;
