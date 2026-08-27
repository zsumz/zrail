//! Operation owners distinguish exact subjects from written method-name authority.

use zrail_core::{AnalysisQuality, Finding, FindingSink, OwnerContract, OwnerKind};

use crate::source::{ObservedFact, RustFileFacts, SourceOperationFact, SourceOperationKind};

use super::{normalized_path, ownership::fact_applies, path_matches};

pub(super) fn check(
    owner: &OwnerContract,
    file: &RustFileFacts,
    findings: &mut FindingSink,
) -> bool {
    let facts = matching(owner, file);
    if owner.kind != OwnerKind::MethodName
        && let Some(fact) = facts
            .iter()
            .find(|fact| fact.quality != AnalysisQuality::Exact)
    {
        findings.push(
            Finding::error(
                "OWN-006",
                &owner.name,
                "ownership",
                format!(
                    "owned {} {} cannot be resolved to one exact Rust identity",
                    operation_label(owner.kind),
                    owner.selector,
                ),
            )
            .at(&file.relative, fact.span)
            .because(&owner.reason)
            .with_analysis(fact.quality)
            .with_help("use an exact type or receiver identity at the ownership boundary"),
        );
    }
    !facts.is_empty()
}

pub(crate) fn matching<'a>(
    owner: &OwnerContract,
    file: &'a RustFileFacts,
) -> Vec<&'a ObservedFact> {
    file.operations
        .iter()
        .filter(|operation| operation_applies(owner, file, operation))
        .map(|operation| &operation.identity)
        .collect()
}

pub(crate) fn matching_operations<'a>(
    owner: &'a OwnerContract,
    file: &'a RustFileFacts,
) -> impl Iterator<Item = &'a SourceOperationFact> + 'a {
    file.operations
        .iter()
        .filter(|operation| operation_applies(owner, file, operation))
}

fn operation_applies(
    owner: &OwnerContract,
    file: &RustFileFacts,
    operation: &SourceOperationFact,
) -> bool {
    let fact = &operation.identity;
    operation_matches(owner, operation)
        && fact_applies(owner, file, fact)
        && selector_matches(owner, operation)
}

fn selector_matches(owner: &OwnerContract, operation: &SourceOperationFact) -> bool {
    let fact = &operation.identity;
    if owner.kind == OwnerKind::MethodName {
        return normalized_path(&owner.selector) == normalized_path(&fact.name);
    }
    opaque_field_matches(owner, operation)
        || path_matches(&owner.selector, fact)
        || (fact.quality != AnalysisQuality::Exact
            && fact.canonical.is_empty()
            && last_segment(&owner.selector) == last_segment(&fact.name))
}

fn opaque_field_matches(owner: &OwnerContract, operation: &SourceOperationFact) -> bool {
    if !matches!(
        (owner.kind, operation.kind),
        (
            OwnerKind::FieldRead | OwnerKind::FieldAuthority,
            SourceOperationKind::FieldRead
        )
    ) {
        return false;
    }
    let fact = &operation.identity;
    if fact.quality == AnalysisQuality::Exact {
        return false;
    }
    let Some((selector_base, _)) = owner.selector.rsplit_once("::") else {
        return false;
    };
    fact.policy_names().any(|name| {
        let Some(base) = name.strip_suffix("::*") else {
            return false;
        };
        normalized_path(selector_base) == normalized_path(base)
            || base == "<unresolved>"
            || (fact.canonical.is_empty() && last_segment(selector_base) == last_segment(base))
    })
}

fn last_segment(path: &str) -> &str {
    path.rsplit("::")
        .next()
        .unwrap_or(path)
        .strip_prefix("r#")
        .unwrap_or_else(|| path.rsplit("::").next().unwrap_or(path))
}

fn operation_matches(owner: &OwnerContract, operation: &SourceOperationFact) -> bool {
    match (owner.kind, operation.kind) {
        (
            OwnerKind::TypeConstruction,
            SourceOperationKind::TypeConstruction | SourceOperationKind::ConstructorCapability,
        )
        | (OwnerKind::MethodName, SourceOperationKind::MethodCall)
        | (OwnerKind::FieldRead | OwnerKind::FieldAuthority, SourceOperationKind::FieldRead)
        | (
            OwnerKind::FieldWrite | OwnerKind::FieldAuthority | OwnerKind::FieldMutation,
            SourceOperationKind::FieldWrite,
        )
        | (
            OwnerKind::FieldMutableBorrow | OwnerKind::FieldAuthority | OwnerKind::FieldMutation,
            SourceOperationKind::FieldMutableBorrow,
        ) => true,
        (OwnerKind::FieldMutation, SourceOperationKind::FieldReceiverCall) => operation
            .method
            .as_ref()
            .is_some_and(|method| owner.mutating_methods.binary_search(method).is_ok()),
        _ => false,
    }
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
        OwnerKind::Call => "call",
        OwnerKind::Capability => "capability use",
        OwnerKind::Directory => "directory use",
    }
}

#[cfg(test)]
#[path = "ownership_operation_test.rs"]
mod ownership_operation_test;
