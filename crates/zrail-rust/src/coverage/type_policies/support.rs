//! Shared deterministic rendering for exact type coverage observations.

use zrail_core::{AnalysisQuality, DuplicationTrait, PolicyReachability, TypeProhibition};

use crate::{
    engine::RepositoryModel,
    rules::type_policy::{duplication, identity},
    source::{RustFileFacts, SyntaxGuard},
};

use super::super::GovernedCompilationDomain;

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
        .map(domain)
        .collect()
}

pub(super) fn domain(domain: &crate::source::CompilationDomain) -> GovernedCompilationDomain {
    GovernedCompilationDomain {
        package: domain.package.clone(),
        edition: domain.edition.clone(),
        target: domain.target.clone(),
        mode: domain.mode.canonical_name().into(),
        feature_world: domain.feature_world.clone(),
        features: domain.active_features.iter().cloned().collect(),
    }
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
