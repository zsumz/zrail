//! One rejected invocation renders one deterministic, actionable diagnostic.

use std::collections::BTreeSet;

use zrail_core::Finding;
use zrail_core::MacroExpansionAllow;

use crate::source::{MacroExpansionFact, RustFileFacts};

use super::failure::MacroBindingFailure;

pub(super) fn unbound(
    file: &RustFileFacts,
    expansion: &MacroExpansionFact,
    attempted: &[&MacroExpansionAllow],
    reasons: &[MacroBindingFailure],
) -> Finding {
    let summaries = reasons
        .iter()
        .map(MacroBindingFailure::summary)
        .collect::<Vec<_>>()
        .join("; ");
    let helps = reasons
        .iter()
        .map(MacroBindingFailure::help)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    Finding::error(
        "RUST-MACRO-006",
        "rust.macro-binding",
        "source",
        format!(
            "macro allowance(s) {} match invocation {}, but cannot bind: {summaries}",
            attempted
                .iter()
                .map(|allowance| allowance.name.as_str())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .map(|name| format!("{name:?}"))
                .collect::<Vec<_>>()
                .join(", "),
            expansion.name,
        ),
    )
    .at(&file.relative, expansion.span)
    .with_analysis(expansion.quality)
    .with_help(helps.join("; "))
}

pub(super) fn unreviewed(file: &RustFileFacts, expansion: &MacroExpansionFact) -> Finding {
    let preferred = expansion.preferred_policy_name();
    let message = preferred
        .filter(|name| *name != expansion.name)
        .map_or_else(
            || {
                format!(
                    "source invokes unreviewed macro expansion {}",
                    expansion.name
                )
            },
            |name| {
                format!(
                    "source invokes unreviewed macro expansion {} (preferred policy name {name})",
                    expansion.name
                )
            },
        );
    let help = preferred.map_or_else(
        || {
            "remove the macro or add a reasoned source.rust.macros.allow entry after reviewing its expansion boundary".into()
        },
        |name| {
            format!(
                "remove the macro or add source.rust.macros.allow name = {name:?} after reviewing its expansion boundary"
            )
        },
    );
    Finding::error("RUST-MACRO-001", "rust.macro-expansion", "source", message)
        .at(&file.relative, expansion.span)
        .with_analysis(expansion.quality)
        .with_help(help)
}
