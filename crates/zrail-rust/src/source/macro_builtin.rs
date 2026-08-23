//! Compiler-derived identities normalize only after every feasible origin resolves.

use super::{MacroExpansionFact, MacroOrigin};

pub(super) fn normalize_derive(expansion: &mut MacroExpansionFact) {
    if !expansion.has_builtin_derive_syntax()
        || !expansion
            .candidates
            .iter()
            .any(|candidate| candidate.origins.contains(&MacroOrigin::CompilerBuiltin))
    {
        return;
    }
    expansion
        .candidates
        .retain(|candidate| !candidate.origins.contains(&MacroOrigin::CompilerBuiltin));
    expansion
        .candidates
        .extend(MacroExpansionFact::compiler_builtin(expansion.observation.clone()).candidates);
    expansion.refresh_quality();
}
