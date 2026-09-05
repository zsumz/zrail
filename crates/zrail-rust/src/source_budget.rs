//! One deterministic physical-file budget resolution for checks, plans, and coverage.

use serde::{Deserialize, Serialize};
use zrail_core::{
    FileRole, RustSourceContract, ScopedBudgetContract, SizeExceptionContract, SizeRole,
    SizeTargetMode, SizeThresholds, glob_matches,
};

use crate::{
    inventory::FileClass,
    source::{Reachability, RustFileFacts, SourceIndex},
    source_policy,
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Fully resolved line-budget authority for a physical source file.
pub struct EffectiveSizeBudget {
    /// Canonical identity of the global role default or selected override.
    pub policy_id: String,
    /// Size selection role; it does not change compilation/test reachability.
    pub role: SizeRole,
    /// Selected thresholds before applying the exact hard exception.
    pub thresholds: SizeThresholds,
    /// Full matched selector and its reason, when an override applies.
    pub scoped: Option<ScopedBudgetContract>,
    /// Exact-file hard allowance with its own reason and tracking metadata.
    pub exception: Option<SizeExceptionContract>,
    /// True when a measured ratchet cannot bypass the independent hard ceiling.
    pub independent_hard: bool,
    /// Required accountability form in the scoped mode, including for future exceptions.
    #[serde(default)]
    pub exception_metadata: Option<zrail_core::SizeExceptionMetadata>,
}

impl EffectiveSizeBudget {
    /// Effective maximum, including an explicit exact-file exception if present.
    pub fn hard_ceiling(&self) -> usize {
        self.exception
            .as_ref()
            .map_or(self.thresholds.hard, |exception| exception.hard)
    }
}

pub(crate) fn for_file(
    file: &RustFileFacts,
    rust: &RustSourceContract,
) -> Result<Option<EffectiveSizeBudget>, String> {
    for_path(
        &file.relative,
        file.class,
        file.reachability,
        &file.packages,
        rust,
    )
}

pub(crate) fn for_path(
    path: &str,
    class: FileClass,
    reachability: Reachability,
    packages: &[String],
    rust: &RustSourceContract,
) -> Result<Option<EffectiveSizeBudget>, String> {
    let role = size_role(path, class, reachability, rust);
    let mut scopes = rust
        .budgets
        .iter()
        .flat_map(|policy| &policy.overrides)
        .filter(|scope| matches(scope, path, role, packages))
        .collect::<Vec<_>>();
    scopes.sort_by(|left, right| left.name.cmp(&right.name));
    if scopes.len() > 1 {
        return Err(format!(
            "RUST-SIZE-006 rust.file-size: {path:?} matches competing same-tier budgets: {}",
            scopes
                .iter()
                .map(|scope| scope.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let scoped = scopes.first().copied();
    let thresholds = scoped.map(|scope| scope.budget).or_else(|| {
        source_policy::budget_for(path, class, reachability, rust).map(|budget| SizeThresholds {
            target: budget.target,
            soft: None,
            hard: budget.hard,
            target_mode: SizeTargetMode::Error,
        })
    });
    let exception = rust
        .budgets
        .iter()
        .flat_map(|policy| &policy.exceptions)
        .find(|exception| exception.path == path);
    let Some(thresholds) = thresholds else {
        if exception.is_some() {
            return Err(format!(
                "RUST-SIZE-008 rust.file-size: exception for {path:?} has no active budget"
            ));
        }
        return Ok(None);
    };
    let policy_id = scoped.map_or_else(
        || default_policy_id(path, class, reachability, rust),
        |scope| format!("rust:size:scope:{}", scope.name),
    );
    Ok(Some(EffectiveSizeBudget {
        policy_id,
        role,
        thresholds,
        scoped: scoped.cloned(),
        exception: exception.cloned(),
        independent_hard: rust.budgets.is_some(),
        exception_metadata: rust
            .budgets
            .as_ref()
            .map(|policy| policy.exception_metadata),
    }))
}

pub(crate) fn validate(
    source: &SourceIndex,
    rust: &RustSourceContract,
    cargo: &crate::cargo::CargoWorkspace,
) -> Result<(), String> {
    let Some(policy) = &rust.budgets else {
        return Ok(());
    };
    let packages = cargo
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for scope in &policy.overrides {
        for package in &scope.packages {
            if !packages.contains(package.as_str()) {
                return Err(format!(
                    "RUST-SIZE-006 rust.file-size: budget {:?} names unknown workspace package {package:?}",
                    scope.name
                ));
            }
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    for file in &source.files {
        if seen.insert(&file.relative) {
            for_file(file, rust)?;
        }
    }
    Ok(())
}

fn matches(scope: &ScopedBudgetContract, path: &str, role: SizeRole, packages: &[String]) -> bool {
    (scope.roles.is_empty() || scope.roles.contains(&role))
        && (scope.packages.is_empty()
            || scope
                .packages
                .iter()
                .any(|package| packages.contains(package)))
        && scope
            .include
            .iter()
            .any(|pattern| glob_matches(pattern, path))
        && !scope
            .exclude
            .iter()
            .any(|pattern| glob_matches(pattern, path))
}

fn size_role(
    path: &str,
    class: FileClass,
    reachability: Reachability,
    rust: &RustSourceContract,
) -> SizeRole {
    if class == FileClass::Generated {
        return SizeRole::Generated;
    }
    if reachability.is_test_only() {
        return if matches!(path.rsplit('/').next(), Some("lib.rs" | "mod.rs"))
            || rust
                .file_roles
                .iter()
                .any(|role| role.path == path && role.role == FileRole::TestFacade)
        {
            SizeRole::TestFacade
        } else {
            SizeRole::Test
        };
    }
    match source_policy::effective_file_role(path, class, rust).effective {
        FileClass::Facade => SizeRole::Facade,
        FileClass::Implementation | FileClass::Test => SizeRole::Implementation,
        FileClass::Auxiliary => SizeRole::Auxiliary,
        FileClass::EntryPoint => SizeRole::Entrypoint,
        FileClass::Generated => SizeRole::Generated,
    }
}

fn default_policy_id(
    path: &str,
    class: FileClass,
    reachability: Reachability,
    rust: &RustSourceContract,
) -> String {
    if class == FileClass::Generated {
        return rust
            .generated
            .iter()
            .find(|source| crate::inventory::under_root(path, &source.root))
            .map_or_else(
                || "rust:size:generated".into(),
                |source| format!("rust:generated:{}", source.root),
            );
    }
    let role = if reachability.is_test_only() {
        "test"
    } else {
        match source_policy::effective_file_role(path, class, rust).effective {
            FileClass::Facade => "facade",
            FileClass::Implementation | FileClass::Test => "implementation",
            FileClass::Auxiliary | FileClass::EntryPoint => "auxiliary",
            FileClass::Generated => "generated",
        }
    };
    format!("rust:size:{role}")
}
