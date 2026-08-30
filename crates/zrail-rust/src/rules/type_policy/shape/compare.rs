//! Shape comparison consumes only a domain-resolved declaration representation.

use zrail_core::{AnalysisQuality, RustTypeContract};

use crate::source::TypeDeclarationKind;

use super::resolve::ResolvedDeclarationShape;

pub(crate) fn problems(
    policy: &RustTypeContract,
    actual: &ResolvedDeclarationShape,
) -> Vec<(AnalysisQuality, String)> {
    let mut problems = Vec::new();
    if policy.visibility.is_none() && policy.leaf_module.is_none() && policy.fields.is_none() {
        return problems;
    }
    if let Some(opacity) = &actual.opacity {
        return vec![(AnalysisQuality::Unresolved, opacity.clone())];
    }
    if let Some(expected) = &policy.visibility
        && *expected != actual.visibility
    {
        problems.push((
            AnalysisQuality::Exact,
            format!(
                "type visibility differs: expected {expected}, observed {}",
                actual.visibility
            ),
        ));
    }
    if let Some(expected) = policy.leaf_module {
        match &actual.leaf_module {
            Ok(observed) if expected != *observed => problems.push((
                AnalysisQuality::Exact,
                format!("leaf-module shape differs: expected {expected}, observed {observed}"),
            )),
            Err(error) => problems.push((AnalysisQuality::Unresolved, error.clone())),
            _ => {}
        }
    }
    let Some(expected) = &policy.fields else {
        return problems;
    };
    let actual_fields = match &actual.fields {
        Err(error) => {
            problems.push((AnalysisQuality::Unresolved, error.clone()));
            return problems;
        }
        Ok(Some(fields)) if actual.kind == TypeDeclarationKind::NamedStruct => fields,
        _ => {
            problems.push((
                AnalysisQuality::Exact,
                "exact fields require a named-field struct".into(),
            ));
            return problems;
        }
    };
    if expected.len() != actual_fields.len() {
        problems.push((
            AnalysisQuality::Exact,
            format!(
                "field count differs: expected {}, observed {}",
                expected.len(),
                actual_fields.len()
            ),
        ));
        return problems;
    }
    for (expected, actual) in expected.iter().zip(actual_fields) {
        let expected_type = super::render::render_contract(&expected.type_identity)
            .unwrap_or_else(|error| format!("<invalid: {error}>"));
        if expected.name != actual.name
            || expected.visibility != actual.visibility
            || expected_type != actual.type_identity
        {
            problems.push((
                AnalysisQuality::Exact,
                format!(
                    "field representation differs: expected {}: {} {} but observed {}: {} {}",
                    expected.visibility,
                    expected.name,
                    expected_type,
                    actual.visibility,
                    actual.name,
                    actual.type_identity
                ),
            ));
        }
    }
    problems
}
