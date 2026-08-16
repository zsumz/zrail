//! Compile-time file embedding stays literal and inside inventoried repository input.

use std::collections::BTreeMap;

use zrail_core::{AnalysisQuality, Finding, FindingSink};

use crate::{
    inventory::RepositoryEntryKind,
    source::{join_relative, parent},
};

use super::super::RuleContext;

pub(super) fn check_paths(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let entries = context
        .inventory
        .entries
        .iter()
        .map(|entry| (entry.relative.as_str(), entry.kind))
        .collect::<BTreeMap<_, _>>();
    for file in &context.source.files {
        for boundary in &file.compile_effects {
            let name = boundary
                .invocation
                .name
                .rsplit("::")
                .next()
                .unwrap_or(&boundary.invocation.name);
            if boundary.invocation.quality != AnalysisQuality::Exact
                || !boundary.invocation.is_compiler_builtin()
                || !matches!(name, "include" | "include_str" | "include_bytes")
            {
                continue;
            }
            if boundary.opaque_input && name == "include" {
                findings.push(invalid(
                    file,
                    boundary,
                    "source inclusion inside opaque macro input cannot be traversed exactly",
                ));
                continue;
            }
            let Some(target) = boundary.target.as_deref() else {
                if name != "include" {
                    findings.push(invalid(file, boundary, "file path is not one literal"));
                }
                continue;
            };
            let resolved = join_relative(&parent(&file.relative), target);
            let Ok(resolved) = resolved else {
                findings.push(invalid(
                    file,
                    boundary,
                    "file path escapes or is not portable within the repository",
                ));
                continue;
            };
            if entries.get(resolved.as_str()) != Some(&RepositoryEntryKind::File) {
                findings.push(invalid(
                    file,
                    boundary,
                    "file path does not identify an inventoried repository file",
                ));
            }
        }
    }
}

fn invalid(
    file: &crate::source::RustFileFacts,
    boundary: &crate::source::CompileEffectFact,
    reason: &str,
) -> Finding {
    Finding::error(
        "RUST-COMPILE-001",
        "rust.compile-filesystem",
        "source",
        format!("{}! {reason}", boundary.invocation.name),
    )
    .at(&file.relative, boundary.invocation.span)
    .with_analysis(AnalysisQuality::Unresolved)
    .with_help("use one literal path to an inventoried file inside the repository")
}
