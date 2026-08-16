//! Unexpanded Rust syntax is an explicit, reasoned trust boundary.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, Finding, FindingSink, MacroExpansionMode};

use crate::source::{ObservedFact, Reachability};

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    if context.contract.source.rust.macros.mode == MacroExpansionMode::Allow {
        return;
    }
    let allowed = context
        .contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .map(|allowed| (allowed.name.as_str(), allowed.reason.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut used = BTreeSet::new();
    for file in context
        .source
        .files
        .iter()
        .filter(|file| file.reachability != Reachability::Unreachable)
    {
        for expansion in &file.macro_expansions {
            if directly_inspected(expansion) {
                continue;
            }
            let matched = reviewed_names(expansion, &allowed);
            if matched.is_empty() {
                findings.push(
                    Finding::error(
                        "RUST-MACRO-001",
                        "rust.macro-expansion",
                        "source",
                        format!(
                            "source invokes unreviewed macro expansion {}",
                            expansion.name
                        ),
                    )
                    .at(&file.relative, expansion.span)
                    .with_analysis(expansion.quality)
                    .with_help(
                        "remove the macro or add a reasoned source.rust.macros.allow entry after reviewing its expansion boundary",
                    ),
                );
            } else {
                used.extend(matched);
            }
        }
    }
    for (name, reason) in allowed {
        if !used.contains(name) {
            findings.push(
                Finding::error(
                    "RUST-MACRO-002",
                    "rust.macro-expansion",
                    "source",
                    format!("allowed macro expansion {name:?} matches no reachable invocation"),
                )
                .because(reason)
                .with_help("remove stale macro expansion authority"),
            );
        }
    }
}

fn reviewed_names<'a>(expansion: &'a ObservedFact, allowed: &BTreeMap<&str, &str>) -> Vec<&'a str> {
    if expansion.quality == AnalysisQuality::Unresolved {
        return Vec::new();
    }
    let names = expansion.policy_names().collect::<Vec<_>>();
    if names.iter().all(|name| allowed.contains_key(name)) {
        names
    } else {
        Vec::new()
    }
}

fn directly_inspected(expansion: &ObservedFact) -> bool {
    if expansion.quality != AnalysisQuality::Exact {
        return false;
    }
    expansion.policy_names().all(|name| {
        let Some(leaf) = (!name.contains("::")).then_some(name) else {
            return false;
        };
        let intrinsic = matches!(
            leaf,
            "cfg"
                | "column"
                | "concat"
                | "concat_bytes"
                | "env"
                | "file"
                | "include"
                | "include_bytes"
                | "include_str"
                | "line"
                | "module_path"
                | "option_env"
                | "stringify"
        );
        intrinsic
    })
}

#[cfg(test)]
#[path = "macro_expansion_test.rs"]
mod macro_expansion_test;
