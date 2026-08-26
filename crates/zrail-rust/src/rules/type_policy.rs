//! Exact Rust type identities govern shape and duplication independently.

pub(crate) mod duplication;
mod duplication_expansions;
pub(crate) mod identity;
pub(crate) mod manual_impls;
pub(crate) mod shape;

use zrail_core::{AnalysisQuality, Finding, FindingSink, RustTypeContract};

use crate::source::{RustFileFacts, TypeDeclarationFact};

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    duplication::check_written(context, findings);
    for policy in &context.contract.source.rust.types {
        evaluate_policy(context, policy, findings);
    }
}

fn evaluate_policy(
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    findings: &mut FindingSink,
) {
    let mut selected = Vec::new();
    let mut uncertain = Vec::new();
    let mut wrong_subject = Vec::new();
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
                Some(crate::source::FactNamespace::Type),
            );
            if resolution.is_exact(&policy.identity) {
                selected.push((file, declaration));
            } else if written_subject(file, declaration, &policy.identity) {
                if resolution.unresolved || resolution.contains(&policy.identity) {
                    uncertain.push((file, declaration));
                } else {
                    wrong_subject.push((file, declaration, resolution.exact));
                }
            }
        }
    }
    let had_uncertain = !uncertain.is_empty();
    for (file, declaration) in uncertain {
        subject_finding(
            policy,
            file,
            Some(declaration),
            "cannot be resolved to one exact canonical identity",
            AnalysisQuality::Unresolved,
            findings,
        );
    }
    let had_wrong_subject = !wrong_subject.is_empty();
    for (file, declaration, observed) in &wrong_subject {
        subject_finding(
            policy,
            file,
            Some(declaration),
            &format!("resolves as {observed:?} instead of its configured canonical identity"),
            AnalysisQuality::Exact,
            findings,
        );
    }
    if !selected.is_empty() {
        for (file, declaration) in selected {
            shape::check(context, policy, file, declaration, findings);
            duplication::check_type(context, policy, file, declaration, findings);
        }
        return;
    }
    if context
        .source
        .files
        .iter()
        .all(|file| file.relative != policy.path)
    {
        findings.push(
            Finding::error(
                "RUST-TYPE-001",
                &policy.name,
                "type-policy",
                format!(
                    "exact type {} has no analyzed declaration file {}",
                    policy.identity, policy.path
                ),
            )
            .at(&policy.path, None)
            .because(&policy.reason)
            .with_help("restore the exact declaration file or remove the stale type policy"),
        );
    } else if !had_uncertain && !had_wrong_subject {
        findings.push(
            Finding::error(
                "RUST-TYPE-001",
                &policy.name,
                "type-policy",
                format!(
                    "exact type {} has no available declaration in {}",
                    policy.identity, policy.path
                ),
            )
            .at(&policy.path, None)
            .because(&policy.reason)
            .with_help("restore the exact declaration or remove the stale type policy"),
        );
    }
}

fn written_subject(
    file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
    identity: &str,
) -> bool {
    let expected = identity.rsplit("::").next().unwrap_or(identity);
    file.paths.iter().any(|fact| {
        fact.span == Some(declaration.identity_span) && fact.written.as_deref() == Some(expected)
    })
}

fn subject_finding(
    policy: &RustTypeContract,
    file: &RustFileFacts,
    declaration: Option<&TypeDeclarationFact>,
    problem: &str,
    quality: AnalysisQuality,
    findings: &mut FindingSink,
) {
    findings.push(
        Finding::error(
            "RUST-TYPE-001",
            &policy.name,
            "type-policy",
            format!("exact type {} {problem}", policy.identity),
        )
        .at(
            &file.relative,
            declaration.map(|declaration| declaration.identity_span),
        )
        .because(&policy.reason)
        .with_analysis(quality)
        .with_help("bind the declaration through the canonical Rust identity layer"),
    );
}
