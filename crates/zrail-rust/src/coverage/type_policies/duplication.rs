//! Manual implementation and opaque expansion coverage mirrors enforcement.

use std::collections::BTreeSet;

use zrail_core::{RustTypeContract, TypeProhibition};

use crate::{
    engine::RepositoryModel,
    rules::{
        RuleContext, closes_type_duplication,
        type_policy::{identity, manual_impls},
    },
};

use super::{super::GovernedTypeObservation, SelectedDeclaration, support};

pub(super) fn observations(
    model: &RepositoryModel,
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    declarations: &[SelectedDeclaration<'_>],
) -> Vec<GovernedTypeObservation> {
    let mut observations = manual_impls(model, context, policy, declarations);
    if !support::permitted(policy, TypeProhibition::OpaqueExpansion) {
        observations.extend(opaque_expansions(model, context, policy, declarations));
    }
    observations
}

fn manual_impls(
    model: &RepositoryModel,
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    declarations: &[SelectedDeclaration<'_>],
) -> Vec<GovernedTypeObservation> {
    let mut observations = Vec::new();
    for file in &context.source.files {
        for implementation in &file.type_policy.trait_impls {
            if !declarations.iter().any(|selected| {
                manual_impls::same_active_world(
                    context,
                    policy,
                    selected.file,
                    selected.declaration,
                    file,
                    implementation,
                )
            }) {
                continue;
            }
            let analysis = manual_impls::analyze(context, policy, file, implementation);
            let trait_kind = match analysis.matched {
                manual_impls::ManualImplMatch::Irrelevant => continue,
                manual_impls::ManualImplMatch::Exact(trait_kind) => Some(trait_kind),
                manual_impls::ManualImplMatch::Possible(trait_kind) => trait_kind,
            };
            let allowed = trait_kind.is_some_and(|trait_kind| {
                support::permitted(policy, support::impl_prohibition(trait_kind))
            });
            let trait_observed = trait_kind.map_or_else(
                || implementation.trait_hint.clone(),
                |trait_kind| support::trait_name(trait_kind).into(),
            );
            let target_observed = support::observed_identity(
                &analysis.target,
                written_target(file, implementation).unwrap_or("<unresolved>"),
            );
            observations.push(GovernedTypeObservation {
                path: file.relative.clone(),
                operation: "manual-impl".into(),
                observed: format!("{trait_observed} for {target_observed}"),
                canonical: analysis.trait_identity.exact.iter().cloned().collect(),
                declaration_kind: None,
                visibility: None,
                leaf_module: None,
                fields: None,
                span: implementation.trait_span,
                lexical_scope: implementation.lexical_scope.clone(),
                quality: support::resolution_quality(&analysis.trait_identity)
                    .max(support::resolution_quality(&analysis.target)),
                guard: implementation.guard.canonical_name(),
                compilation_domains: support::domains(
                    model,
                    file,
                    &implementation.guard,
                    policy.reachability,
                ),
                allowed,
                closed: None,
            });
        }
    }
    observations
}

fn written_target<'a>(
    file: &'a crate::source::RustFileFacts,
    implementation: &crate::source::TraitImplFact,
) -> Option<&'a str> {
    let span = implementation.type_span?;
    file.paths
        .iter()
        .find(|fact| fact.span == Some(span))
        .and_then(|fact| fact.written.as_deref())
}

fn opaque_expansions(
    model: &RepositoryModel,
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    declarations: &[SelectedDeclaration<'_>],
) -> Vec<GovernedTypeObservation> {
    let mut observations = Vec::new();
    let mut emitted = BTreeSet::new();
    for selected in declarations {
        let declaration_domains = identity::domain_identities(
            context,
            selected.file,
            &selected.declaration.guard,
            policy.reachability,
        )
        .into_iter()
        .collect::<BTreeSet<_>>();
        let packages = selected.file.packages.iter().collect::<BTreeSet<_>>();
        for file in context.source.files.iter().filter(|file| {
            file.packages
                .iter()
                .any(|package| packages.contains(package))
        }) {
            for fact in file.item_macros.iter().chain(&file.opaque_binding_macros) {
                let Some(span) = fact.span else {
                    continue;
                };
                if !identity::applies(context, file, &fact.guard, policy.reachability)
                    || !identity::domain_identities(context, file, &fact.guard, policy.reachability)
                        .into_iter()
                        .any(|domain| declaration_domains.contains(&domain))
                    || !emitted.insert((file.relative.clone(), span))
                {
                    continue;
                }
                let closed = file.macro_expansions.iter().any(|expansion| {
                    expansion.span == Some(span)
                        && closes_type_duplication(
                            context.contract,
                            context.source,
                            context.resolved_cargo,
                            expansion,
                        )
                });
                observations.push(GovernedTypeObservation {
                    path: file.relative.clone(),
                    operation: "opaque-expansion".into(),
                    observed: fact.name.clone(),
                    canonical: fact.policy_names().map(str::to_owned).collect(),
                    declaration_kind: None,
                    visibility: None,
                    leaf_module: None,
                    fields: None,
                    span,
                    lexical_scope: fact.lexical_scope.clone(),
                    quality: fact.quality,
                    guard: fact.guard.canonical_name(),
                    compilation_domains: support::domains(
                        model,
                        file,
                        &fact.guard,
                        policy.reachability,
                    ),
                    allowed: closed,
                    closed: Some(closed),
                });
            }
        }
    }
    observations
}
