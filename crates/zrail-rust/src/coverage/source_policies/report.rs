//! Source-policy coverage collection.

use zrail_core::{AnalysisQuality, DuplicationTrait};

use crate::{
    engine::RepositoryModel,
    rules::{assigned_profiles, async_syntax_name, closes_async_syntax, glob_import_is_allowed},
    source::DuplicationSyntaxKind,
};

use super::{
    super::{GovernedSourcePolicyOccurrence, GovernedSourcePolicyRail},
    support::{
        applies, direct_occurrence, duplication_trait_name, glob_mode, macro_occurrence,
        policy_domains, reachability, visibility,
    },
};

pub(crate) fn report(model: &RepositoryModel) -> Vec<GovernedSourcePolicyRail> {
    let mut rails = vec![glob_imports(model)];
    rails.extend(duplication(model));
    for (profile_name, profile) in &model.bundle.contract.profiles {
        for denied in &profile.syntax.deny {
            let mut occurrences = Vec::new();
            for file in model.source.files.iter().filter(|file| {
                assigned_profiles(&model.bundle.contract, file, &model.cargo.packages)
                    .contains(profile_name.as_str())
            }) {
                occurrences.extend(file.async_syntax.iter().filter_map(|fact| {
                    (fact.kind == *denied && applies(profile.reachability, file, &fact.observation))
                        .then(|| direct_occurrence(model, file, fact))
                        .flatten()
                }));
                occurrences.extend(file.macro_expansions.iter().filter_map(|expansion| {
                    applies(profile.reachability, file, &expansion.observation)
                        .then(|| {
                            macro_occurrence(
                                model,
                                file,
                                expansion,
                                closes_async_syntax(
                                    &model.bundle.contract,
                                    &model.source,
                                    model.resolved_cargo.as_ref(),
                                    expansion,
                                ),
                            )
                        })
                        .flatten()
                }));
            }
            occurrences.sort();
            let syntax = async_syntax_name(*denied);
            rails.push(GovernedSourcePolicyRail {
                policy_id: format!("profile:{profile_name}:syntax:{syntax}"),
                policy: syntax.into(),
                profile: Some(profile_name.clone()),
                reachability: reachability(profile.reachability).into(),
                occurrences,
            });
        }
    }
    rails.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
    rails
}

fn duplication(model: &RepositoryModel) -> Vec<GovernedSourcePolicyRail> {
    let policy = &model.bundle.contract.source.rust.duplication;
    let imports = policy
        .deny_imports
        .iter()
        .copied()
        .map(|trait_name| duplication_rail(model, DuplicationSyntaxKind::Import, trait_name));
    let macro_tokens =
        policy.deny_macro_tokens.iter().copied().map(|trait_name| {
            duplication_rail(model, DuplicationSyntaxKind::MacroToken, trait_name)
        });
    imports.chain(macro_tokens).collect()
}

fn duplication_rail(
    model: &RepositoryModel,
    kind: DuplicationSyntaxKind,
    trait_kind: DuplicationTrait,
) -> GovernedSourcePolicyRail {
    let selected_reachability = model.bundle.contract.source.rust.duplication.reachability;
    let operation = match kind {
        DuplicationSyntaxKind::Import => "import",
        DuplicationSyntaxKind::MacroToken => "macro-token",
    };
    let trait_name = duplication_trait_name(trait_kind);
    let mut occurrences = model
        .source
        .files
        .iter()
        .flat_map(|file| {
            file.type_policy
                .syntax
                .iter()
                .filter(move |fact| fact.kind == kind && fact.trait_name == trait_kind)
                .filter_map(move |fact| {
                    let compilation_domains =
                        policy_domains(model, &file.relative, &fact.guard, selected_reachability);
                    (!compilation_domains.is_empty()).then(|| GovernedSourcePolicyOccurrence {
                        path: file.relative.clone(),
                        operation: operation.into(),
                        observed: trait_name.into(),
                        visibility: None,
                        lexical_scope: fact.lexical_scope.clone(),
                        span: fact.span,
                        quality: AnalysisQuality::Exact,
                        guard: fact.guard.canonical_name(),
                        compilation_domains,
                        allowed: false,
                    })
                })
        })
        .collect::<Vec<_>>();
    occurrences.sort();
    GovernedSourcePolicyRail {
        policy_id: format!("rust:duplication:{operation}:{trait_name}"),
        policy: trait_name.into(),
        profile: None,
        reachability: reachability(selected_reachability).into(),
        occurrences,
    }
}

fn glob_imports(model: &RepositoryModel) -> GovernedSourcePolicyRail {
    let mode = model.bundle.contract.source.rust.hygiene.glob_imports;
    let mut occurrences = model
        .source
        .files
        .iter()
        .flat_map(|file| {
            let effective = crate::source_policy::effective_file_role(
                &file.relative,
                file.class,
                &model.bundle.contract.source.rust,
            )
            .effective;
            file.glob_imports
                .iter()
                .map(move |fact| GovernedSourcePolicyOccurrence {
                    path: file.relative.clone(),
                    operation: "glob-import".into(),
                    observed: format!("{}::*", fact.target),
                    visibility: Some(visibility(&fact.visibility)),
                    lexical_scope: fact.lexical_scope.clone(),
                    span: fact.span,
                    quality: AnalysisQuality::Exact,
                    guard: fact.guard.canonical_name(),
                    compilation_domains: super::support::domains(
                        model,
                        &file.relative,
                        &fact.guard,
                    ),
                    allowed: glob_import_is_allowed(mode, effective, file.reachability, fact),
                })
        })
        .collect::<Vec<_>>();
    occurrences.sort();
    GovernedSourcePolicyRail {
        policy_id: "rust:hygiene:glob-imports".into(),
        policy: glob_mode(mode).into(),
        profile: None,
        reachability: "all".into(),
        occurrences,
    }
}
