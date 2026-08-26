//! Exact declaration-shape enforcement.

use zrail_core::{Finding, FindingSink, RustTypeContract};

use crate::source::{RustFileFacts, TypeDeclarationFact, TypeDeclarationKind};

use super::super::RuleContext;
use super::render::{render_contract, render_source};

pub(crate) fn check(
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
    findings: &mut FindingSink,
) {
    if let Some(expected) = &policy.visibility
        && declaration.visibility != *expected
    {
        mismatch(
            policy,
            file,
            declaration,
            &format!(
                "type visibility differs: expected {expected}, observed {}",
                declaration.visibility
            ),
            findings,
        );
    }
    if let Some(expected) = policy.leaf_module
        && declaration.leaf_module != expected
    {
        mismatch(
            policy,
            file,
            declaration,
            &format!("leaf-module shape differs: expected {expected}"),
            findings,
        );
    }
    let Some(expected) = &policy.fields else {
        return;
    };
    if declaration.kind != TypeDeclarationKind::NamedStruct {
        mismatch(
            policy,
            file,
            declaration,
            "exact fields require a named-field struct",
            findings,
        );
        return;
    }
    let Some(actual) = &declaration.fields else {
        return;
    };
    if expected.len() != actual.len() {
        mismatch(
            policy,
            file,
            declaration,
            &format!(
                "field count differs: expected {}, observed {}",
                expected.len(),
                actual.len()
            ),
            findings,
        );
        return;
    }
    for (expected, actual) in expected.iter().zip(actual) {
        let actual_type = render_source(&actual.type_shape, context, file, policy.reachability);
        let expected_type = render_contract(&expected.type_identity);
        let types_match = matches!(
            (&actual_type, &expected_type),
            (Ok(actual), Ok(expected)) if actual == expected
        );
        if expected.name != actual.name || expected.visibility != actual.visibility || !types_match
        {
            let observed = actual_type.unwrap_or_else(|error| format!("<unresolved: {error}>"));
            let expected_type = expected_type.unwrap_or_else(|error| format!("<invalid: {error}>"));
            mismatch(
                policy,
                file,
                declaration,
                &format!(
                    "field representation differs: expected {}: {} {} but observed {}: {} {}",
                    expected.visibility,
                    expected.name,
                    expected_type,
                    actual.visibility,
                    actual.name,
                    observed
                ),
                findings,
            );
        }
    }
}

fn mismatch(
    policy: &RustTypeContract,
    file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
    message: &str,
    findings: &mut FindingSink,
) {
    findings.push(
        Finding::error(
            "RUST-TYPE-002",
            &policy.name,
            "type-policy",
            format!("exact type {} {message}", policy.identity),
        )
        .at(&file.relative, Some(declaration.identity_span))
        .because(&policy.reason)
        .with_help("restore the reviewed exact declaration shape or update policy through review"),
    );
}
