//! Module contracts and declarative facade enforcement.

use zrail_core::{FacadeMode, Finding, FindingSink, ModuleDocsMode};

use crate::{
    inventory::{FileClass, under_root},
    source::{SourceSyntax, join_relative, parent},
};

use super::RuleContext;
use super::count_ratchet::{self, CountRatchetSpec};

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    if context.contract.source.rust.module_docs == ModuleDocsMode::Required {
        count_ratchet::evaluate(
            context,
            CountRatchetSpec {
                rule: "rust.module-docs",
                finding_id: "RUST-DOC-002",
                finding_rule: "rust.module-docs.ratchet",
                category: "source-shape",
                debt: "missing module documentation",
                report_source_lock_drift: false,
            },
            None,
            findings,
            report_missing_module_docs,
        );
    }
    for file in &context.source.files {
        let effective = crate::source_policy::effective_file_role(
            &file.relative,
            file.class,
            &context.contract.source.rust,
        )
        .effective;
        let mode = crate::source_policy::facade_mode_for(
            &file.relative,
            file.class,
            &context.contract.source.rust,
        );
        if let Some(mode) = mode.filter(|mode| *mode != FacadeMode::Allow) {
            if !facade_syntax_allowed(&context.contract.source.rust, file, mode) {
                findings.push(Finding::error(
                    "RUST-FACADE-002", "rust.facades", "source-shape",
                    "facade structure requires a Rust item file; an included expression or item fragment is unsupported",
                ).at(&file.relative, None));
            }
            let (rail, description) = if effective == FileClass::EntryPoint {
                ("rust.entrypoints", "entrypoint")
            } else {
                ("rust.facades", "facade")
            };
            for item in facade_violations(&context.contract.source.rust, file, mode) {
                findings.push(
                    Finding::error(
                        "RUST-FACADE-001",
                        rail,
                        "source-shape",
                        format!(
                            "{description} ({}) contains forbidden item {}",
                            crate::source_policy::facade_mode_name(mode),
                            item.name
                        ),
                    )
                    .at(&file.relative, item.span)
                    .with_analysis(item.quality)
                    .with_help("move implementation behind a named module boundary"),
                );
            }
        }
    }
}

pub(crate) fn facade_syntax_allowed(
    rust: &zrail_core::RustSourceContract,
    file: &crate::source::RustFileFacts,
    mode: FacadeMode,
) -> bool {
    file.syntax == SourceSyntax::Items
        || mode == FacadeMode::Allow
        || mode == FacadeMode::Declarative
            && !rust.file_roles.iter().any(|role| {
                role.path == file.relative && role.role == zrail_core::FileRole::TestFacade
            })
}

fn report_missing_module_docs(file: &crate::source::RustFileFacts, findings: &mut FindingSink) {
    if file.syntax == SourceSyntax::Items && file.class != FileClass::Generated && !file.module_docs
    {
        findings.push(
            Finding::error(
                "RUST-DOC-001",
                "rust.module-docs",
                "source-shape",
                "Rust source is missing its module contract (`//!`)",
            )
            .at(&file.relative, None)
            .with_help("start the file with a concise `//!` responsibility statement"),
        );
    }
}

pub(crate) fn facade_violations<'a>(
    rust: &'a zrail_core::RustSourceContract,
    file: &'a crate::source::RustFileFacts,
    mode: FacadeMode,
) -> impl Iterator<Item = &'a crate::source::ObservedFact> {
    file.facade_implementation.iter().filter(move |item| {
        mode != FacadeMode::Allow
            && !(mode == FacadeMode::Declarative && generated_include(rust, file, item))
    })
}

fn generated_include(
    rust: &zrail_core::RustSourceContract,
    file: &crate::source::RustFileFacts,
    item: &crate::source::ObservedFact,
) -> bool {
    item.name == "include!"
        && file.includes.iter().any(|include| {
            include.span == item.span
                && (literal_generated_include(rust, file, include)
                    || include.out_dir.as_deref().is_some_and(|output| {
                        rust.out_dir.iter().any(|binding| {
                            binding.path == file.relative && binding.output == output
                        })
                    }))
        })
}

fn literal_generated_include(
    rust: &zrail_core::RustSourceContract,
    file: &crate::source::RustFileFacts,
    include: &crate::source::IncludeBoundary,
) -> bool {
    include.path.as_deref().is_some_and(|path| {
        join_relative(&parent(&file.relative), path).is_ok_and(|target| {
            rust.generated
                .iter()
                .any(|generated| under_root(&target, &generated.root))
        })
    })
}
