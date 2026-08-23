//! Sibling tests must be separate, test-only, and reachable through Rust modules.

mod inline;

use std::path::Path;

use zrail_core::{Finding, FindingSink, TestMode};

use crate::source::RustFileFacts;

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    if context.contract.source.rust.tests != TestMode::Sibling {
        return;
    }
    inline::check(context, findings);
    check_declarations(context, findings);
}

fn check_declarations(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for test in context
        .source
        .files
        .iter()
        .filter(|file| is_sibling_test(&file.relative))
    {
        if test.reachability.is_non_test_target() {
            findings.push(
                Finding::error(
                    "RUST-TEST-004",
                    "rust.tests.reachability",
                    "test-placement",
                    format!(
                        "sibling test {} is reachable by production code",
                        test.relative
                    ),
                )
                .at(&test.relative, None)
                .with_help("remove every unconditional Cargo or module edge to this test file"),
            );
            continue;
        }
        let exact = context
            .module_edges
            .iter()
            .filter(|edge| edge.child == test.relative)
            .collect::<Vec<_>>();
        if test.reachability.is_test_only()
            && exact
                .iter()
                .any(|edge| edge.guard.is_test_only() && edge.reachability.is_test_only())
        {
            continue;
        }
        if let Some(edge) = exact.first() {
            findings.push(
                missing_declaration(test, "is declared without #[cfg(test)]")
                    .at(&edge.parent, edge.span),
            );
            continue;
        }
        let stem = Path::new(&test.relative)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if let Some((source, declaration)) = context
            .source
            .files
            .iter()
            .flat_map(|source| {
                source
                    .modules
                    .iter()
                    .map(move |declaration| (source, declaration))
            })
            .find(|(_, declaration)| declaration.name == stem)
        {
            findings.push(
                Finding::error(
                    "RUST-TEST-003",
                    "rust.tests.path",
                    "test-placement",
                    format!(
                        "module {stem} does not resolve to sibling test {}",
                        test.relative
                    ),
                )
                .at(&source.relative, declaration.span)
                .with_help("declare it from the containing module or use an exact #[path]"),
            );
        } else {
            findings.push(missing_declaration(
                test,
                "has no reachable #[cfg(test)] module declaration",
            ));
        }
    }
}

fn is_sibling_test(path: &str) -> bool {
    path.ends_with("_test.rs") && path.split('/').any(|component| component == "src")
}

fn missing_declaration(file: &RustFileFacts, message: &str) -> Finding {
    Finding::error(
        "RUST-TEST-002",
        "rust.tests.declaration",
        "test-placement",
        format!("sibling test {} {message}", file.relative),
    )
    .at(&file.relative, None)
}
