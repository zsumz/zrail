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
        let declarative = match effective {
            FileClass::Facade => context.contract.source.rust.facades == FacadeMode::Declarative,
            FileClass::EntryPoint => {
                context.contract.source.rust.entrypoints == FacadeMode::Declarative
            }
            _ => false,
        };
        if declarative {
            let (rail, description) = if effective == FileClass::EntryPoint {
                ("rust.entrypoints", "declarative entrypoint")
            } else {
                ("rust.facades", "declarative facade")
            };
            for item in &file.facade_implementation {
                if generated_include(context, file, item) {
                    continue;
                }
                findings.push(
                    Finding::error(
                        "RUST-FACADE-001",
                        rail,
                        "source-shape",
                        format!("{description} contains implementation item {}", item.name),
                    )
                    .at(&file.relative, item.span)
                    .with_analysis(item.quality)
                    .with_help("move implementation behind a named module boundary"),
                );
            }
        }
    }
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

fn generated_include(
    context: &RuleContext<'_>,
    file: &crate::source::RustFileFacts,
    item: &crate::source::ObservedFact,
) -> bool {
    item.name == "include!"
        && file.includes.iter().any(|include| {
            include.span == item.span
                && (literal_generated_include(context, file, include)
                    || include.out_dir.as_deref().is_some_and(|output| {
                        context.contract.source.rust.out_dir.iter().any(|binding| {
                            binding.path == file.relative && binding.output == output
                        })
                    }))
        })
}

fn literal_generated_include(
    context: &RuleContext<'_>,
    file: &crate::source::RustFileFacts,
    include: &crate::source::IncludeBoundary,
) -> bool {
    include.path.as_deref().is_some_and(|path| {
        join_relative(&parent(&file.relative), path).is_ok_and(|target| {
            context
                .contract
                .source
                .rust
                .generated
                .iter()
                .any(|generated| under_root(&target, &generated.root))
        })
    })
}
