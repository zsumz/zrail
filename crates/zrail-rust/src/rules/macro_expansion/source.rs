//! Expansion authority is bound to observed compiler, repository, or dependency origin.

use zrail_core::MacroExpansionAllow;

use crate::{
    cargo::source_matches,
    source::{MacroExpansionFact, MacroOrigin},
};

pub(super) fn bound(expansion: &MacroExpansionFact, allowance: &MacroExpansionAllow) -> bool {
    !expansion.origins.is_empty()
        && expansion.origins.iter().all(|origin| match origin {
            MacroOrigin::CompilerBuiltin | MacroOrigin::Repository { .. } => {
                allowance.source.is_none()
            }
            MacroOrigin::External { source, .. } => allowance
                .source
                .as_ref()
                .is_some_and(|allowed| source_matches(allowed, source)),
            MacroOrigin::Pending { .. } | MacroOrigin::Unresolved => false,
        })
}
