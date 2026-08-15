//! Configurable production-source hygiene rails.

use zrail_core::{Finding, FindingSink, LintSuppressionMode, PolicyMode};

use crate::inventory::FileClass;

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let hygiene = &context.contract.source.rust.hygiene;
    for file in context
        .source
        .files
        .iter()
        .filter(|file| !matches!(file.class, FileClass::Test | FileClass::Auxiliary))
    {
        for method in &file.methods {
            if hygiene
                .deny_methods
                .iter()
                .any(|denied| denied == &method.name)
            {
                findings.push(
                    Finding::error(
                        "RUST-HYG-001",
                        "rust.hygiene.method",
                        "source-hygiene",
                        format!("production source uses denied method {}()", method.name),
                    )
                    .at(&file.relative, method.span)
                    .with_analysis(method.quality)
                    .with_help("return or translate the failure explicitly"),
                );
            }
        }
        for invocation in &file.macros {
            if hygiene
                .deny_macros
                .iter()
                .any(|denied| denied == &invocation.name)
            {
                findings.push(
                    Finding::error(
                        "RUST-HYG-002",
                        "rust.hygiene.macro",
                        "source-hygiene",
                        format!("production source uses denied macro {}!", invocation.name),
                    )
                    .at(&file.relative, invocation.span)
                    .with_analysis(invocation.quality),
                );
            }
        }
        if hygiene.lint_suppressions != LintSuppressionMode::Allow
            && file.class != FileClass::Generated
        {
            for suppression in file.lint_suppressions.iter().filter(|suppression| {
                hygiene.lint_suppressions == LintSuppressionMode::Deny
                    || suppression.name == "unreasoned lint suppression"
            }) {
                findings.push(
                    Finding::error(
                        "RUST-HYG-003",
                        "rust.hygiene.lint-suppression",
                        "source-hygiene",
                        if suppression.name == "unreasoned lint suppression" {
                            "production source suppresses a compiler or Clippy lint without a reason"
                        } else {
                            "production source suppresses a compiler or Clippy lint"
                        },
                    )
                    .at(&file.relative, suppression.span)
                    .with_help("fix the warning or configure the lint once at workspace scope"),
                );
            }
        }
        if hygiene.unsafe_code == PolicyMode::Deny {
            for unsafe_construct in &file.unsafe_constructs {
                findings.push(
                    Finding::error(
                        "RUST-HYG-004",
                        "rust.hygiene.unsafe",
                        "source-hygiene",
                        format!("production source contains {}", unsafe_construct.name),
                    )
                    .at(&file.relative, unsafe_construct.span),
                );
            }
        }
    }
}
