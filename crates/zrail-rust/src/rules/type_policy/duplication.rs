//! Clone/Copy syntax and whole-world macro opacity fail closed independently.

use zrail_core::{
    AnalysisQuality, DuplicationTrait, Finding, FindingSink, RustTypeContract, TypeLinearity,
    TypeProhibition,
};

use crate::source::{DuplicationSyntaxKind, RustFileFacts, TypeDeclarationFact};

use super::{RuleContext, identity};

pub(super) fn check_written(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let policy = &context.contract.source.rust.duplication;
    for file in &context.source.files {
        for fact in &file.type_policy.syntax {
            if !identity::applies(context, file, &fact.guard, policy.reachability) {
                continue;
            }
            let denied = match fact.kind {
                DuplicationSyntaxKind::Import => policy.deny_imports.contains(&fact.trait_name),
                DuplicationSyntaxKind::MacroToken => {
                    policy.deny_macro_tokens.contains(&fact.trait_name)
                }
            };
            if !denied {
                continue;
            }
            let (id, rule, operation) = match fact.kind {
                DuplicationSyntaxKind::Import => (
                    "RUST-TYPE-006",
                    "rust.duplication.import",
                    "explicit import or alias",
                ),
                DuplicationSyntaxKind::MacroToken => (
                    "RUST-TYPE-007",
                    "rust.duplication.macro-token",
                    "opaque macro token",
                ),
            };
            findings.push(
                Finding::error(
                    id,
                    rule,
                    "type-policy",
                    format!(
                        "production source contains {operation} for {}",
                        trait_name(fact.trait_name)
                    ),
                )
                .at(&file.relative, Some(fact.span))
                .with_analysis(AnalysisQuality::Exact)
                .with_help("remove the written duplication surface or narrow the reviewed policy"),
            );
        }
    }
}

pub(super) fn check_type(
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    declaration_file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
    findings: &mut FindingSink,
) {
    check_derives(context, policy, declaration_file, declaration, findings);
    super::manual_impls::check(context, policy, declaration_file, declaration, findings);
    if denies(policy, TypeProhibition::OpaqueExpansion) {
        super::duplication_expansions::check(
            context,
            policy,
            declaration_file,
            declaration,
            findings,
        );
    }
}

fn check_derives(
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
    findings: &mut FindingSink,
) {
    for derive in &declaration.derives {
        if !identity::applies(context, file, &derive.guard, policy.reachability) {
            continue;
        }
        let Some(trait_kind) = trait_from_hint(&derive.trait_hint) else {
            continue;
        };
        let prohibited = match trait_kind {
            DuplicationTrait::Clone => TypeProhibition::DeriveClone,
            DuplicationTrait::Copy => TypeProhibition::DeriveCopy,
        };
        if !denies(policy, prohibited) {
            continue;
        }
        let quality = derive_quality(&file.macro_expansions, derive.span, trait_kind);
        findings.push(
            Finding::error(
                "RUST-TYPE-003",
                &policy.name,
                "type-policy",
                format!(
                    "linear type {} derives {}",
                    policy.identity,
                    trait_name(trait_kind)
                ),
            )
            .at(&file.relative, Some(derive.span))
            .because(&policy.reason)
            .with_analysis(quality)
            .with_help("remove the duplication derive; unresolved derive provenance fails closed"),
        );
    }
}

fn derive_quality(
    expansions: &[crate::source::MacroExpansionFact],
    span: zrail_core::SourceSpan,
    trait_kind: DuplicationTrait,
) -> AnalysisQuality {
    if expansions.iter().any(|expansion| {
        expansion.span == Some(span)
            && expansion.is_compiler_builtin()
            && expansion.candidates.iter().any(|candidate| {
                candidate
                    .policy_names()
                    .any(|name| standard_trait(name) == Some(trait_kind))
            })
    }) {
        AnalysisQuality::Exact
    } else {
        AnalysisQuality::Unresolved
    }
}

pub(crate) fn denies(policy: &RustTypeContract, prohibition: TypeProhibition) -> bool {
    policy.linearity == TypeLinearity::Required || policy.deny.contains(&prohibition)
}

pub(crate) fn standard_trait(name: &str) -> Option<DuplicationTrait> {
    match name.trim_start_matches("::") {
        "Clone" | "core::clone::Clone" | "std::clone::Clone" => Some(DuplicationTrait::Clone),
        "Copy" | "core::marker::Copy" | "std::marker::Copy" => Some(DuplicationTrait::Copy),
        _ => None,
    }
}

pub(crate) fn trait_from_hint(name: &str) -> Option<DuplicationTrait> {
    match name {
        "Clone" => Some(DuplicationTrait::Clone),
        "Copy" => Some(DuplicationTrait::Copy),
        _ => None,
    }
}

const fn trait_name(value: DuplicationTrait) -> &'static str {
    match value {
        DuplicationTrait::Clone => "Clone",
        DuplicationTrait::Copy => "Copy",
    }
}

#[cfg(test)]
#[path = "duplication_test.rs"]
mod duplication_test;
