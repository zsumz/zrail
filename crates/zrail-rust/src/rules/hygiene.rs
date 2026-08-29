//! Configurable production-source hygiene rails.

use zrail_core::{Finding, FindingSink, LintSuppressionMode, PolicyMode};

use crate::inventory::FileClass;

use super::{
    RuleContext,
    count_ratchet::{self, CountRatchetSpec},
};

mod glob_imports;

use glob_imports::check_glob_imports;
pub(crate) use glob_imports::glob_import_is_allowed;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let hygiene = &context.contract.source.rust.hygiene;
    for selector in &hygiene.deny_methods {
        let debt = format!("uses of denied method {selector}()");
        count_ratchet::evaluate(
            context,
            CountRatchetSpec {
                rule: "rust.hygiene.denied-method",
                finding_id: "RUST-HYG-007",
                finding_rule: "rust.hygiene.method.ratchet",
                category: "source-hygiene",
                debt: &debt,
                report_source_lock_drift: true,
            },
            Some(selector),
            findings,
            |file, findings| report_denied_methods(file, selector, findings),
        );
    }
    for selector in &hygiene.deny_macros {
        let debt = format!("uses of denied macro {selector}!");
        count_ratchet::evaluate(
            context,
            CountRatchetSpec {
                rule: "rust.hygiene.denied-macro",
                finding_id: "RUST-HYG-008",
                finding_rule: "rust.hygiene.macro.ratchet",
                category: "source-hygiene",
                debt: &debt,
                report_source_lock_drift: true,
            },
            Some(selector),
            findings,
            |file, findings| report_denied_macros(file, selector, findings),
        );
    }
    if hygiene.lint_suppressions != LintSuppressionMode::Allow {
        count_ratchet::evaluate(
            context,
            CountRatchetSpec {
                rule: "rust.hygiene.lint-suppressions",
                finding_id: "RUST-HYG-006",
                finding_rule: "rust.hygiene.lint-suppression.ratchet",
                category: "source-hygiene",
                debt: "lint-suppression violations",
                report_source_lock_drift: true,
            },
            None,
            findings,
            |file, findings| report_lint_suppressions(file, hygiene.lint_suppressions, findings),
        );
    }
    if hygiene.unsafe_code == PolicyMode::Deny {
        count_ratchet::evaluate(
            context,
            CountRatchetSpec {
                rule: "rust.hygiene.unsafe",
                finding_id: "RUST-HYG-005",
                finding_rule: "rust.hygiene.unsafe.ratchet",
                category: "source-hygiene",
                debt: "unsafe constructs",
                report_source_lock_drift: true,
            },
            None,
            findings,
            report_unsafe_constructs,
        );
    }
    check_glob_imports(context, findings);
}

fn report_denied_methods(
    file: &crate::source::RustFileFacts,
    selector: &str,
    findings: &mut FindingSink,
) {
    for method in file
        .methods
        .iter()
        .filter(|method| count_ratchet::method_matches(selector, method))
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

fn report_denied_macros(
    file: &crate::source::RustFileFacts,
    selector: &str,
    findings: &mut FindingSink,
) {
    for invocation in file
        .macros
        .iter()
        .filter(|invocation| count_ratchet::macro_matches(selector, invocation))
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

fn report_lint_suppressions(
    file: &crate::source::RustFileFacts,
    mode: LintSuppressionMode,
    findings: &mut FindingSink,
) {
    if file.class == FileClass::Generated {
        return;
    }
    for suppression in file
        .lint_suppressions
        .iter()
        .filter(|suppression| count_ratchet::lint_suppression_violates(mode, suppression))
    {
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

fn report_unsafe_constructs(file: &crate::source::RustFileFacts, findings: &mut FindingSink) {
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

#[cfg(test)]
#[path = "hygiene_test.rs"]
mod hygiene_test;
