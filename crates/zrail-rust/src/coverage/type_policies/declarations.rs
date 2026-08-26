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
            let (fields, fields_exact) = observed_fields(context, policy, file, declaration);
            let mut quality = support::resolution_quality(&resolution);
            if !fields_exact {
                quality = quality.max(AnalysisQuality::Unresolved);
            }
            let allowed = resolution.is_exact(&policy.identity)
                && shape_allowed(context, policy, file, declaration);
            observations.push(GovernedTypeObservation {
                path: file.relative.clone(),
                operation: "declaration".into(),
                observed: support::observed_identity(
                    &resolution,
                    policy
                        .identity
                        .rsplit("::")
                        .next()
                        .unwrap_or(&policy.identity),
                ),
                canonical: resolution.exact.iter().cloned().collect(),
                declaration_kind: Some(kind_name(declaration.kind).into()),
                visibility: Some(declaration.visibility.clone()),
                leaf_module: Some(declaration.leaf_module),
                fields,
                span: declaration.identity_span,
                lexical_scope: declaration.lexical_scope.clone(),
                quality,
                guard: declaration.guard.canonical_name(),
                compilation_domains: support::domains(
                    model,
                    file,
                    &declaration.guard,
                    policy.reachability,
                ),
                allowed,
                closed: None,
            });
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

fn observed_fields(
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
) -> (Option<Vec<GovernedTypeField>>, bool) {
    let Some(fields) = &declaration.fields else {
        return (None, true);
    };
    let mut exact = true;
    let fields = fields
        .iter()
        .map(|field| {
            let rendered =
                shape::render_source(&field.type_shape, context, file, policy.reachability);
            exact &= rendered.is_ok();
            support::observed_field(&field.name, &field.visibility, rendered)
        })
        .collect();
    (Some(fields), exact)
}

fn shape_allowed(
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
) -> bool {
    if policy
        .visibility
        .as_ref()
        .is_some_and(|expected| declaration.visibility != *expected)
        || policy
            .leaf_module
            .is_some_and(|expected| declaration.leaf_module != expected)
    {
        return false;
    }
    let Some(expected) = &policy.fields else {
        return true;
    };
    let Some(actual) = &declaration.fields else {
        return false;
    };
    declaration.kind == TypeDeclarationKind::NamedStruct
        && expected.len() == actual.len()
        && expected.iter().zip(actual).all(|(expected, actual)| {
            expected.name == actual.name
                && expected.visibility == actual.visibility
                && shape::render_contract(&expected.type_identity).is_ok_and(|expected| {
                    shape::render_source(&actual.type_shape, context, file, policy.reachability)
                        == Ok(expected)
                })
        })
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
