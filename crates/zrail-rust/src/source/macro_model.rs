//! One macro invocation owns every bounded candidate identity and origin.

use zrail_core::{AnalysisQuality, Effect};

use crate::cargo::DependencySource;

use super::model::ObservedFact;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum MacroOrigin {
    Pending {
        local_module: bool,
    },
    CompilerBuiltin,
    Repository {
        package: String,
        directory: String,
    },
    External {
        package: String,
        source: DependencySource,
    },
    Unresolved,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum MacroDerivation {
    Written,
    ExactImport,
    GlobImport,
    ReExport,
    LocalDefinition,
    DependencyRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MacroCandidate {
    pub(crate) observation: ObservedFact,
    pub(crate) origins: Vec<MacroOrigin>,
    pub(crate) derivation: MacroDerivation,
    /// Whether the invocation spelling is an exact lexical alias for this candidate.
    pub(crate) written_alias: bool,
}

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
        }
    }

    pub(crate) fn unresolved(observation: ObservedFact, derivation: MacroDerivation) -> Self {
        Self {
            observation,
            origins: vec![MacroOrigin::Unresolved],
            derivation,
            written_alias: false,
        }
    }

    pub(crate) fn policy_names(&self) -> impl Iterator<Item = &str> {
        self.observation.policy_names()
    }

    pub(crate) fn allowance_names<'a>(&'a self, written: &'a str) -> Vec<&'a str> {
        let mut names = self.policy_names().collect::<Vec<_>>();
        if self.written_alias && names.len() == 1 && names[0] != written && valid_path(written) {
            names.push(written);
        }
        names
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MacroExpansionFact {
    /// The spelling present at the invocation site.
    pub(crate) observation: ObservedFact,
    /// Every statically feasible policy identity for this one invocation.
    pub(crate) candidates: Vec<MacroCandidate>,
}

impl MacroExpansionFact {
    #[cfg(test)]
    pub(crate) fn pending(observation: ObservedFact, local_module: bool) -> Self {
        let derivation = if local_module {
            MacroDerivation::LocalDefinition
        } else {
            MacroDerivation::Written
        };
        let candidate = MacroCandidate::pending(observation.clone(), local_module, derivation);
        Self {
            observation,
            candidates: vec![candidate],
        }
    }

    pub(crate) fn unresolved(observation: ObservedFact) -> Self {
        let candidate = MacroCandidate::unresolved(observation.clone(), MacroDerivation::Written);
        Self {
            observation,
            candidates: vec![candidate],
        }
    }

    pub(crate) fn compiler_builtin(observation: ObservedFact) -> Self {
        let candidate = MacroCandidate {
            observation: observation.clone(),
            origins: vec![MacroOrigin::CompilerBuiltin],
            derivation: MacroDerivation::Written,
            written_alias: false,
        };
        Self {
            observation,
            candidates: vec![candidate],
        }
    }

    pub(crate) fn with_candidates(
        mut observation: ObservedFact,
        candidates: Vec<MacroCandidate>,
    ) -> Self {
        observation.quality = candidates
            .iter()
            .map(|candidate| candidate.observation.quality)
            .max()
            .unwrap_or(AnalysisQuality::Unresolved);
        Self {
            observation,
            candidates,
        }
    }

    pub(crate) fn refresh_quality(&mut self) {
        self.observation.quality = self
            .candidates
            .iter()
            .map(|candidate| candidate.observation.quality)
            .max()
            .unwrap_or(AnalysisQuality::Unresolved);
    }

    pub(super) fn mark_test_only(&mut self) {
        self.observation.mark_test_only();
        for candidate in &mut self.candidates {
            candidate.observation.mark_test_only();
        }
    }

    pub(crate) fn preferred_policy_name(&self) -> Option<&str> {
        let names = self
            .candidates
            .iter()
            .flat_map(MacroCandidate::policy_names)
            .filter(|name| valid_path(name))
            .collect::<std::collections::BTreeSet<_>>();
        if names.len() == 1 {
            names.into_iter().next()
        } else {
            valid_path(&self.name).then_some(self.name.as_str())
        }
    }

    pub(crate) fn names_covered_by(&self, allowed: &std::collections::BTreeSet<&str>) -> bool {
        self.candidates.iter().all(|candidate| {
            let names = candidate.policy_names().collect::<Vec<_>>();
            names.iter().all(|name| allowed.contains(name))
                || (names.len() == 1
                    && candidate.written_alias
                    && allowed.contains(self.name.as_str()))
        })
    }

    pub(crate) fn origins(&self) -> impl Iterator<Item = &MacroOrigin> {
        self.candidates
            .iter()
            .flat_map(|candidate| &candidate.origins)
    }

    pub(crate) fn is_compiler_builtin(&self) -> bool {
        self.candidates.len() == 1
            && self.candidates[0].origins.as_slice() == [MacroOrigin::CompilerBuiltin]
    }
}

fn valid_path(name: &str) -> bool {
    syn::parse_str::<syn::Path>(name).is_ok()
}

impl std::ops::Deref for MacroExpansionFact {
    type Target = ObservedFact;

    fn deref(&self) -> &Self::Target {
        &self.observation
    }
}

impl std::ops::DerefMut for MacroExpansionFact {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.observation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompileEffectFact {
    pub(crate) effect: Effect,
    pub(crate) invocation: MacroExpansionFact,
    pub(crate) target: Option<String>,
    pub(crate) opaque_input: bool,
}
