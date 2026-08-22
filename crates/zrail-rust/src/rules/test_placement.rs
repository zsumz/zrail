//! Sibling tests must be separate, test-only, and reachable through Rust modules.

mod inline;

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use zrail_core::{Finding, FindingSink, TestMode};

use crate::source::{
    ModuleDeclaration, ModuleTarget, Reachability, RustFileFacts, SubmoduleBase, join_relative,
    module_target,
};

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    if context.contract.source.rust.tests != TestMode::Sibling {
        return;
    }
    inline::check(context, findings);
    check_declarations(context, findings);
}

fn check_declarations(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let file_paths = context
        .source
        .files
        .iter()
        .map(|file| file.relative.as_str())
        .collect::<BTreeSet<_>>();
    let crate_roots = context
        .cargo
        .packages
        .iter()
        .flat_map(|package| {
            package
                .targets
                .iter()
                .filter_map(|target| join_relative(&package.directory, &target.path).ok())
        })
        .collect::<BTreeSet<_>>();
    let declarations = context
        .source
        .files
        .iter()
        .flat_map(|source| {
            let file_paths = &file_paths;
            let submodule_base = if crate_roots.contains(&source.relative) || is_mod_rs(source) {
                SubmoduleBase::SourceParent
            } else {
                SubmoduleBase::FileStemDirectory
            };
            source.modules.iter().map(move |declaration| {
                let target = resolved_module_target(
                    &source.relative,
                    submodule_base,
                    declaration,
                    file_paths,
                );
                (source, declaration, target)
            })
        })
        .collect::<Vec<_>>();

    for test in context
        .source
        .files
        .iter()
        .filter(|file| is_sibling_test(&file.relative))
    {
        if test.reachability.is_production() {
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
        let exact = declarations
            .iter()
            .filter(|(_, _, target)| target.as_deref() == Some(test.relative.as_str()))
            .collect::<Vec<_>>();
        if test.reachability == Reachability::TestOnly
            && exact.iter().any(|(_, declaration, _)| declaration.cfg_test)
        {
            continue;
        }
        if let Some((source, declaration, _)) = exact.first() {
            findings.push(
                missing_declaration(test, "is declared without #[cfg(test)]")
                    .at(&source.relative, declaration.span),
            );
            continue;
        }
        let stem = Path::new(&test.relative)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if let Some((source, declaration, _)) = declarations
            .iter()
            .find(|(_, declaration, _)| declaration.name == stem)
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

fn resolved_module_target(
    source: &str,
    submodule_base: SubmoduleBase,
    declaration: &ModuleDeclaration,
    files: &BTreeSet<&str>,
) -> Option<String> {
    match module_target(source, submodule_base, declaration).ok()? {
        ModuleTarget::Exact(path) => files.contains(path.as_str()).then_some(path),
        ModuleTarget::Search { direct, nested } => {
            match (
                files.contains(direct.as_str()),
                files.contains(nested.as_str()),
            ) {
                (true, false) => Some(direct),
                (false, true) => Some(nested),
                _ => None,
            }
        }
    }
}

fn is_mod_rs(file: &RustFileFacts) -> bool {
    PathBuf::from(&file.relative)
        .file_name()
        .is_some_and(|name| name == "mod.rs")
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

#[cfg(test)]
#[path = "test_placement_test.rs"]
mod test_placement_test;
