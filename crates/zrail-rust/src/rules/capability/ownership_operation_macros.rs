//! Opaque expansion output remains unresolved at exact operation-owner boundaries.

use zrail_core::{AnalysisQuality, Finding, FindingSink, OwnerContract, OwnerKind};

use crate::source::{MacroExpansionFact, RustFileFacts, SyntaxGuard};

use super::{RuleContext, ownership};

pub(super) fn check_allowed(
    context: &RuleContext<'_>,
    owner: &OwnerContract,
    file: &RustFileFacts,
    findings: &mut FindingSink,
) -> bool {
    let expansions = unresolved_expansions(context, owner, file).collect::<Vec<_>>();
    for expansion in &expansions {
        findings.push(
            Finding::error(
                "OWN-006",
                &owner.name,
                "ownership",
                format!(
                    "macro expansion {} may emit owned {} {}, but its source operations are opaque",
                    expansion.name,
                    operation_label(owner.kind),
                    owner.selector,
                ),
            )
            .at(&file.relative, expansion.span)
            .because(&owner.reason)
            .with_analysis(AnalysisQuality::Unresolved)
            .with_help("bind the exact macro provenance and attest source_operations = \"none\""),
        );
    }
    !expansions.is_empty()
}

pub(super) fn reject_outside(
    context: &RuleContext<'_>,
    owner: &OwnerContract,
    file: &RustFileFacts,
    findings: &mut FindingSink,
) {
    if !is_source_operation_owner(owner.kind) {
        return;
    }
    for expansion in unresolved_expansions(context, owner, file) {
        findings.push(
            Finding::error(
                "OWN-003",
                &owner.name,
                "ownership",
                format!(
                    "macro expansion {} may emit owned {} {}; allowed owner: {}",
                    expansion.name,
                    operation_label(owner.kind),
                    owner.selector,
                    owner.allow.join(", "),
                ),
            )
            .at(&file.relative, expansion.span)
            .because(&owner.reason)
            .with_analysis(AnalysisQuality::Unresolved)
            .with_help("move the macro invocation into its declared owner or attest source_operations = \"none\" after exact review"),
        );
    }
}

fn unresolved_expansions<'a>(
    context: &'a RuleContext<'_>,
    owner: &'a OwnerContract,
    file: &'a RustFileFacts,
) -> impl Iterator<Item = &'a MacroExpansionFact> + 'a {
    file.macro_expansions.iter().filter(|expansion| {
        expansion.observation.guard != SyntaxGuard::Never
            && ownership::fact_applies(owner, file, &expansion.observation)
            && !crate::rules::closes_source_operations(
                context.contract,
                context.source,
                context.resolved_cargo,
                expansion,
            )
    })
}

const fn is_source_operation_owner(kind: OwnerKind) -> bool {
    matches!(
        kind,
        OwnerKind::TypeConstruction
            | OwnerKind::MethodName
            | OwnerKind::FieldRead
            | OwnerKind::FieldWrite
            | OwnerKind::FieldMutableBorrow
            | OwnerKind::FieldMutation
            | OwnerKind::FieldAuthority
    )
}

const fn operation_label(kind: OwnerKind) -> &'static str {
    match kind {
        OwnerKind::TypeConstruction => "type construction",
        OwnerKind::MethodName => "written method call",
        OwnerKind::FieldRead => "field read",
        OwnerKind::FieldWrite => "field write",
        OwnerKind::FieldMutableBorrow => "field mutable borrow",
        OwnerKind::FieldMutation => "field mutation",
        OwnerKind::FieldAuthority => "field access",
        OwnerKind::Call | OwnerKind::Capability | OwnerKind::Directory => "source operation",
    }
}
