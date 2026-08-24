//! Candidate constructors retain derivation, written aliases, and exact policy names.

use super::{MacroCandidate, MacroDerivation, MacroOrigin, ObservedFact};

impl MacroCandidate {
    pub(crate) fn pending(
        observation: ObservedFact,
        local_module: bool,
        derivation: MacroDerivation,
    ) -> Self {
        Self {
            observation,
            origins: vec![MacroOrigin::Pending { local_module }],
            derivation,
            written_alias: matches!(
                derivation,
                MacroDerivation::ExactImport | MacroDerivation::ReExport
            ),
            definition: None,
        }
    }

    pub(crate) fn unresolved(observation: ObservedFact, derivation: MacroDerivation) -> Self {
        Self {
            observation,
            origins: vec![MacroOrigin::Unresolved],
            derivation,
            written_alias: false,
            definition: None,
        }
    }

    pub(crate) fn policy_names(&self) -> impl Iterator<Item = &str> {
        self.observation.policy_names()
    }

    pub(crate) fn allowance_names<'a>(&'a self, written: &'a str) -> Vec<&'a str> {
        let mut names = self.policy_names().collect::<Vec<_>>();
        if self.written_alias
            && names.len() == 1
            && names[0] != written
            && super::valid_path(written)
        {
            names.push(written);
        }
        names
    }
}
