//! Expansion authority is bound to observed compiler, repository, or dependency origin.

use zrail_core::MacroExpansionAllow;

use crate::{
    cargo::source_matches,
    source::{MacroCandidate, MacroOrigin},
};

pub(super) fn bound(candidate: &MacroCandidate, allowance: &MacroExpansionAllow) -> bool {
    !candidate.origins.is_empty()
        && candidate.origins.iter().all(|origin| match origin {
            MacroOrigin::CompilerBuiltin => {
                allowance.source.is_none() && allowance.definition.is_none()
            }
            MacroOrigin::Repository { .. } => {
                allowance.source.is_none()
                    && (allowance.definition.is_some()
                        || candidate.policy_names().all(|name| name.contains("::")))
            }
            MacroOrigin::External { source, .. } => allowance
                .source
                .as_ref()
                .is_some_and(|allowed| source_matches(allowed, source)),
            MacroOrigin::Pending { .. } | MacroOrigin::Unresolved => false,
        })
}
