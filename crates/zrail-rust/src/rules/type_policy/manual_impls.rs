//! Manual duplication impls fail closed when either governed identity can be relevant.

use std::collections::BTreeSet;

use zrail_core::{
    AnalysisQuality, DuplicationTrait, Finding, FindingSink, RustTypeContract, TypeProhibition,
};

use crate::source::{FactNamespace, RustFileFacts, TraitImplFact, TypeDeclarationFact};

use super::{RuleContext, duplication, identity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ManualImplMatch {
    Irrelevant,
    Exact(DuplicationTrait),
    Possible(Option<DuplicationTrait>),
}

#[derive(Debug)]
pub(crate) struct ManualImplAnalysis {
    pub(crate) target: identity::IdentityResolution,
    pub(crate) trait_identity: identity::IdentityResolution,
    pub(crate) matched: ManualImplMatch,
}

pub(super) fn check(
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    declaration_file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
    findings: &mut FindingSink,
) {
    for file in &context.source.files {
        for implementation in &file.type_policy.trait_impls {
            if !same_active_world(
                context,
                policy,
                declaration_file,
                declaration,
                file,
                implementation,
            ) {
                continue;
            }
            let analysis = analyze(context, policy, file, implementation);
            let (trait_kind, quality) = match analysis.matched {
                ManualImplMatch::Exact(trait_kind) if denied_impl(policy, trait_kind) => {
                    (Some(trait_kind), AnalysisQuality::Exact)
                }
                ManualImplMatch::Possible(Some(trait_kind)) if denied_impl(policy, trait_kind) => {
                    (Some(trait_kind), AnalysisQuality::Unresolved)
                }
                ManualImplMatch::Irrelevant
                | ManualImplMatch::Exact(_)
                | ManualImplMatch::Possible(Some(_)) => continue,
                ManualImplMatch::Possible(None) => {
                    if !denied_impl(policy, DuplicationTrait::Clone)
                        && !denied_impl(policy, DuplicationTrait::Copy)
                    {
                        continue;
                    }
                    (None, AnalysisQuality::Unresolved)
                }
            };
            findings.push(finding(policy, file, implementation, trait_kind, quality));
        }
    }
}

pub(crate) fn same_active_world(
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    declaration_file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
    implementation_file: &RustFileFacts,
    implementation: &TraitImplFact,
) -> bool {
    if !identity::applies(
        context,
        implementation_file,
        &implementation.guard,
        policy.reachability,
    ) || !implementation_file
        .packages
        .iter()
        .any(|package| declaration_file.packages.contains(package))
    {
        return false;
    }
    let declaration_domains = identity::domain_identities(
        context,
        declaration_file,
        &declaration.guard,
        policy.reachability,
    )
    .into_iter()
    .collect::<BTreeSet<_>>();
    identity::domain_identities(
        context,
        implementation_file,
        &implementation.guard,
        policy.reachability,
    )
    .into_iter()
    .any(|domain| declaration_domains.contains(&domain))
}

pub(crate) fn analyze(
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    file: &RustFileFacts,
    implementation: &TraitImplFact,
) -> ManualImplAnalysis {
    let target =
        implementation
            .type_span
            .map_or_else(identity::IdentityResolution::unresolved, |span| {
                identity::at_span(
                    context,
                    file,
                    span,
                    policy.reachability,
                    Some(FactNamespace::Type),
                )
            });
    let trait_identity = identity::at_span(
        context,
        file,
        implementation.trait_span,
        policy.reachability,
        None,
    );
    let matched = classify(policy, &target, &trait_identity, &implementation.trait_hint);
    ManualImplAnalysis {
        target,
        trait_identity,
        matched,
    }
}

fn classify(
    policy: &RustTypeContract,
    target: &identity::IdentityResolution,
    trait_identity: &identity::IdentityResolution,
    trait_hint: &str,
) -> ManualImplMatch {
    let target_exact = target.is_exact(&policy.identity);
    let target_possible =
        target.unresolved || target.exact.is_empty() || target.contains(&policy.identity);
    if !target_exact && !target_possible {
        return ManualImplMatch::Irrelevant;
    }

    let exact_traits = trait_identity
        .exact
        .iter()
        .filter_map(|name| duplication::standard_trait(name))
        .collect::<Vec<_>>();
    if !trait_identity.unresolved && trait_identity.exact.len() == 1 {
        return exact_traits
            .first()
            .copied()
            .map_or(ManualImplMatch::Irrelevant, |trait_kind| {
                if target_exact {
                    ManualImplMatch::Exact(trait_kind)
                } else {
                    ManualImplMatch::Possible(Some(trait_kind))
                }
            });
    }
    if !trait_identity.unresolved && !trait_identity.exact.is_empty() && exact_traits.is_empty() {
        return ManualImplMatch::Irrelevant;
    }
    if !denied_impl(policy, DuplicationTrait::Clone) && !denied_impl(policy, DuplicationTrait::Copy)
    {
        return ManualImplMatch::Irrelevant;
    }
    let only_exact = if exact_traits.len() == 1 {
        exact_traits.first().copied()
    } else {
        None
    };
    let hinted = duplication::trait_from_hint(trait_hint).or(only_exact);
    ManualImplMatch::Possible(hinted)
}

fn finding(
    policy: &RustTypeContract,
    file: &RustFileFacts,
    implementation: &TraitImplFact,
    trait_kind: Option<DuplicationTrait>,
    quality: AnalysisQuality,
) -> Finding {
    let description = match (quality, trait_kind) {
        (AnalysisQuality::Exact, Some(trait_kind)) => format!(
            "linear type {} manually implements {}",
            policy.identity,
            trait_name(trait_kind)
        ),
        _ => format!(
            "manual impl could duplicate governed type {} but its target or trait identity is unresolved",
            policy.identity
        ),
    };
    Finding::error("RUST-TYPE-004", &policy.name, "type-policy", description)
        .at(&file.relative, Some(implementation.trait_span))
        .because(&policy.reason)
        .with_analysis(quality)
        .with_help(
            "use exact canonical target and trait identities, or remove the duplication impl",
        )
}

fn denied_impl(policy: &RustTypeContract, trait_kind: DuplicationTrait) -> bool {
    let prohibition = match trait_kind {
        DuplicationTrait::Clone => TypeProhibition::ImplClone,
        DuplicationTrait::Copy => TypeProhibition::ImplCopy,
    };
    duplication::denies(policy, prohibition)
}

const fn trait_name(value: DuplicationTrait) -> &'static str {
    match value {
        DuplicationTrait::Clone => "Clone",
        DuplicationTrait::Copy => "Copy",
    }
}

#[cfg(test)]
#[path = "manual_impls_test.rs"]
mod manual_impls_test;
