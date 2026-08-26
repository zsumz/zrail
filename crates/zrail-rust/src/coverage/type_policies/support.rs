//! Shared deterministic rendering for exact type coverage observations.

use zrail_core::{AnalysisQuality, DuplicationTrait, PolicyReachability, TypeProhibition};

use crate::{
    engine::RepositoryModel,
    rules::type_policy::{duplication, identity},
    source::{RustFileFacts, SyntaxGuard},
};

use super::super::{GovernedCompilationDomain, GovernedTypeField};

pub(super) fn domains(
    model: &RepositoryModel,
    file: &RustFileFacts,
    guard: &SyntaxGuard,
    reachability: PolicyReachability,
) -> Vec<GovernedCompilationDomain> {
    model
        .compilation_domains
        .get(&file.relative)
        .into_iter()
        .flatten()
        .filter(|domain| {
            (reachability == PolicyReachability::All
                || matches!(
                    domain.mode,
                    crate::source::CompilationMode::Library
                        | crate::source::CompilationMode::Binary
                ))
                && guard.availability_in_domain(domain).is_available()
        })
        .map(|domain| GovernedCompilationDomain {
            package: domain.package.clone(),
            edition: domain.edition.clone(),
            target: domain.target.clone(),
            mode: compilation_mode(domain.mode).into(),
            feature_world: domain.feature_world.clone(),
            features: domain.active_features.iter().cloned().collect(),
        })
        .collect()
}

pub(super) fn resolution_quality(resolution: &identity::IdentityResolution) -> AnalysisQuality {
    if resolution.unresolved || resolution.exact.is_empty() {
        AnalysisQuality::Unresolved
    } else if resolution.exact.len() == 1 {
        AnalysisQuality::Exact
    } else {
        AnalysisQuality::Conservative
    }
}

pub(super) fn observed_identity(
    resolution: &identity::IdentityResolution,
    fallback: &str,
) -> String {
    if resolution.exact.len() == 1 {
        resolution.exact.iter().next().cloned().unwrap_or_default()
    } else {
        fallback.into()
    }
}

pub(super) fn observed_field(
    name: &str,
    visibility: &str,
    type_identity: Result<String, String>,
) -> GovernedTypeField {
    GovernedTypeField {
        name: name.into(),
        type_identity: type_identity.unwrap_or_else(|error| format!("<unresolved: {error}>")),
        visibility: visibility.into(),
    }
}

pub(super) fn derive_prohibition(value: DuplicationTrait) -> TypeProhibition {
    match value {
        DuplicationTrait::Clone => TypeProhibition::DeriveClone,
        DuplicationTrait::Copy => TypeProhibition::DeriveCopy,
    }
}

pub(super) fn impl_prohibition(value: DuplicationTrait) -> TypeProhibition {
    match value {
        DuplicationTrait::Clone => TypeProhibition::ImplClone,
        DuplicationTrait::Copy => TypeProhibition::ImplCopy,
    }
}

pub(super) fn permitted(policy: &zrail_core::RustTypeContract, value: TypeProhibition) -> bool {
    !duplication::denies(policy, value)
}

pub(super) const fn trait_name(value: DuplicationTrait) -> &'static str {
    match value {
        DuplicationTrait::Clone => "Clone",
        DuplicationTrait::Copy => "Copy",
    }
}

const fn compilation_mode(value: crate::source::CompilationMode) -> &'static str {
    match value {
        crate::source::CompilationMode::Library => "library",
        crate::source::CompilationMode::LibraryTest => "library-test",
        crate::source::CompilationMode::Binary => "binary",
        crate::source::CompilationMode::BinaryTest => "binary-test",
        crate::source::CompilationMode::IntegrationTest => "integration-test",
        crate::source::CompilationMode::Benchmark => "benchmark",
        crate::source::CompilationMode::Example => "example",
        crate::source::CompilationMode::ExampleTest => "example-test",
        crate::source::CompilationMode::BuildScript => "build-script",
    }
}
