//! Effective Rust crate roots remain exact or fail closed before policy matching.

use std::collections::BTreeSet;

use zrail_core::{
    AnalysisQuality, Contract, Finding, FindingSink, MacroExpansionMode, OwnerKind,
    path::glob_matches,
};

use crate::cargo::{CrateRootAuthority, Package, attestation_matches, rust_crate_root};

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let mut used_attestations = BTreeSet::new();
    for package in &context.cargo.packages {
        for dependency in &package.dependencies {
            if dependency.crate_root_authority == CrateRootAuthority::Attested {
                for (index, _) in context
                    .contract
                    .dependencies
                    .crate_roots
                    .iter()
                    .enumerate()
                    .filter(|(_, attestation)| {
                        attestation_matches(attestation, &dependency.name, &dependency.source)
                    })
                {
                    used_attestations.insert(index);
                }
            }
            if dependency.crate_root_authority != CrateRootAuthority::Unresolved
                || !identity_required(context.contract, package, &dependency.name)
            {
                continue;
            }
            findings.push(
                Finding::error(
                    "CARGO-IDENTITY-001",
                    "cargo.crate-root",
                    "dependency",
                    format!(
                        "dependency {:?} has no exact Rust crate-root identity",
                        dependency.alias
                    ),
                )
                .at(package.manifest_path(), None)
                .with_analysis(AnalysisQuality::Unresolved)
                .with_help(
                    "inspect the local library target, use an explicit Cargo package rename, or add a reviewed dependencies.crate_root attestation",
                ),
            );
        }
    }
    for (index, attestation) in context.contract.dependencies.crate_roots.iter().enumerate() {
        if !used_attestations.contains(&index) {
            findings.push(
                Finding::error(
                    "CARGO-IDENTITY-002",
                    "cargo.crate-root",
                    "dependency",
                    format!(
                        "crate-root attestation for package {:?} at {} matches no unresolved external dependency",
                        attestation.package,
                        attestation.source.identity()
                    ),
                )
                .because(&attestation.reason)
                .with_help("remove stale dependency identity authority"),
            );
        }
    }
}

fn identity_required(contract: &Contract, package: &Package, dependency: &str) -> bool {
    let root = rust_crate_root(dependency);
    contract
        .source
        .rust
        .hygiene
        .deny_macros
        .iter()
        .any(|path| references_root(path, &root))
        || contract
            .scopes
            .iter()
            .flat_map(|scope| &scope.symbols.deny)
            .any(|path| references_root(path, &root))
        || contract
            .owners
            .iter()
            .filter(|owner| owner.kind != OwnerKind::Directory)
            .any(|owner| references_root(&owner.selector, &root))
        || (contract.source.rust.macros.mode == MacroExpansionMode::DenyUnreviewed
            && contract
                .source
                .rust
                .macros
                .allow
                .iter()
                .any(|allowed| references_root(&allowed.name, &root)))
        || contract
            .layers
            .iter()
            .filter(|layer| {
                layer
                    .packages
                    .iter()
                    .any(|selector| glob_matches(selector, &package.name))
            })
            .flat_map(|layer| &layer.profiles)
            .filter_map(|profile| contract.profiles.get(profile))
            .flat_map(|profile| &profile.effects.deny)
            .flat_map(|effect| super::capability::effect_tokens(*effect))
            .any(|path| references_root(path, &root))
}

fn references_root(path: &str, root: &str) -> bool {
    path.strip_prefix("r#")
        .unwrap_or(path)
        .split("::")
        .next()
        .is_some_and(|candidate| candidate == root)
}
