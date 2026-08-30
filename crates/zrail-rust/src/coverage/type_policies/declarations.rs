//! Declaration and derive coverage retain exact shape and resolution status.

use zrail_core::{AnalysisQuality, RustTypeContract};

use crate::{
    engine::RepositoryModel,
    rules::{
        RuleContext,
        type_policy::{duplication, identity, shape},
    },
    source::{FactNamespace, RustFileFacts, TypeDeclarationFact, TypeDeclarationKind},
};

use super::{
    super::{GovernedTypeField, GovernedTypeObservation},
    SelectedDeclaration, support,
};

pub(super) fn observations<'a>(
    model: &RepositoryModel,
    context: &RuleContext<'a>,
    policy: &RustTypeContract,
) -> (Vec<GovernedTypeObservation>, Vec<SelectedDeclaration<'a>>) {
    let mut observations = Vec::new();
    let mut selected = Vec::new();
    for file in context
        .source
        .files
        .iter()
        .filter(|file| file.relative == policy.path)
    {
        for declaration in &file.type_policy.declarations {
            if !identity::applies(context, file, &declaration.guard, policy.reachability) {
                continue;
            }
            let resolution = identity::at_span(
                context,
                file,
                declaration.identity_span,
                policy.reachability,
                Some(FactNamespace::Type),
            );
            if !candidate(file, declaration, policy, &resolution) {
                continue;
            }
            for shape in shape::resolve(context, policy, file, declaration) {
                let domain_resolution = identity::at_span_in_domain(
                    file,
                    declaration.identity_span,
                    &shape.domain,
                    FactNamespace::Type,
                );
                let mut quality = support::resolution_quality(&domain_resolution);
                if !shape.is_exact() {
                    quality = quality.max(AnalysisQuality::Unresolved);
                }
                let allowed = domain_resolution.is_exact(&policy.identity)
                    && shape::problems(policy, &shape).is_empty();
                let fields = shape
                    .fields
                    .as_ref()
                    .ok()
                    .and_then(|fields| fields.as_ref())
                    .map(|fields| {
                        fields
                            .iter()
                            .map(|field| GovernedTypeField {
                                name: field.name.clone(),
                                visibility: field.visibility.clone(),
                                type_identity: field.type_identity.clone(),
                            })
                            .collect()
                    });
                observations.push(GovernedTypeObservation {
                    path: file.relative.clone(),
                    operation: "declaration".into(),
                    observed: support::observed_identity(
                        &domain_resolution,
                        policy
                            .identity
                            .rsplit("::")
                            .next()
                            .unwrap_or(&policy.identity),
                    ),
                    canonical: domain_resolution.exact.iter().cloned().collect(),
                    declaration_kind: Some(kind_name(declaration.kind).into()),
                    visibility: Some(declaration.visibility.clone()),
                    leaf_module: shape.leaf_module.ok(),
                    fields,
                    span: declaration.identity_span,
                    lexical_scope: declaration.lexical_scope.clone(),
                    quality,
                    guard: declaration.guard.canonical_name(),
                    compilation_domains: vec![support::domain(&shape.domain)],
                    allowed,
                    closed: None,
                });
            }
            observations.extend(derive_observations(model, policy, file, declaration));
            if resolution.is_exact(&policy.identity) {
                selected.push(SelectedDeclaration { file, declaration });
            }
        }
    }
    (observations, selected)
}

fn candidate(
    file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
    policy: &RustTypeContract,
    resolution: &identity::IdentityResolution,
) -> bool {
    if resolution.contains(&policy.identity) {
        return true;
    }
    let expected = policy
        .identity
        .rsplit("::")
        .next()
        .unwrap_or(&policy.identity);
    file.paths
        .iter()
        .filter(|fact| fact.span == Some(declaration.identity_span))
        .flat_map(crate::source::ObservedFact::policy_names)
        .any(|name| name.rsplit("::").next() == Some(expected))
}

fn derive_observations(
    model: &RepositoryModel,
    policy: &RustTypeContract,
    file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
) -> Vec<GovernedTypeObservation> {
    declaration
        .derives
        .iter()
        .filter_map(|derive| {
            let trait_kind = duplication::trait_from_hint(&derive.trait_hint)?;
            let compilation_domains =
                support::domains(model, file, &derive.guard, policy.reachability);
            if compilation_domains.is_empty() {
                return None;
            }
            let exact = file.macro_expansions.iter().any(|expansion| {
                expansion.span == Some(derive.span)
                    && expansion.is_compiler_builtin()
                    && expansion.candidates.iter().any(|candidate| {
                        candidate
                            .policy_names()
                            .any(|name| duplication::standard_trait(name) == Some(trait_kind))
                    })
            });
            Some(GovernedTypeObservation {
                path: file.relative.clone(),
                operation: "derive".into(),
                observed: support::trait_name(trait_kind).into(),
                canonical: Vec::new(),
                declaration_kind: None,
                visibility: None,
                leaf_module: None,
                fields: None,
                span: derive.span,
                lexical_scope: declaration.lexical_scope.clone(),
                quality: if exact {
                    AnalysisQuality::Exact
                } else {
                    AnalysisQuality::Unresolved
                },
                guard: derive.guard.canonical_name(),
                compilation_domains,
                allowed: support::permitted(policy, support::derive_prohibition(trait_kind)),
                closed: None,
            })
        })
        .collect()
}

const fn kind_name(value: TypeDeclarationKind) -> &'static str {
    match value {
        TypeDeclarationKind::NamedStruct => "named-struct",
        TypeDeclarationKind::Other => "other",
    }
}
