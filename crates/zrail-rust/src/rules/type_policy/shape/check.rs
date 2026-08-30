//! Exact declaration-shape enforcement is independently evaluated in every domain.

use zrail_core::{Finding, FindingSink, RustTypeContract};

use crate::source::{RustFileFacts, TypeDeclarationFact};

use super::super::RuleContext;

pub(crate) fn check(
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
    findings: &mut FindingSink,
) {
    for shape in super::resolve(context, policy, file, declaration) {
        for (quality, message) in super::problems(policy, &shape) {
            findings.push(
                Finding::error(
                    "RUST-TYPE-002", &policy.name, "type-policy",
                    format!("exact type {} {message} in compilation domain [{}]",
                        policy.identity, shape.domain.canonical_identity()),
                )
                .at(&file.relative, Some(declaration.identity_span))
                .because(&policy.reason)
                .with_analysis(quality)
                .with_help("restore the reviewed exact declaration shape in every governed domain; item-replacing attributes are unsupported"),
            );
        }
    }
}
