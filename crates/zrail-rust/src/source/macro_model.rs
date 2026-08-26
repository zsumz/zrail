//! One macro invocation owns every bounded candidate identity and origin.

mod candidate;

use zrail_core::AnalysisQuality;

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
    /// Exact repository definition site when textual lookup selected one.
    pub(crate) definition: Option<String>,
    /// Exact local macro identifier selected at the definition site.
    pub(crate) definition_name: Option<String>,
    /// SHA-256 of the selected local macro definition token stream.
    pub(crate) definition_sha256: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MacroExpansionFact {
    /// The spelling present at the invocation site.
    pub(crate) observation: ObservedFact,
    /// Every statically feasible policy identity for this one invocation.
    pub(crate) candidates: Vec<MacroCandidate>,
    pub(crate) lexical_scope: Vec<zrail_core::SourceSpan>,
    /// Digest of the canonical invocation token stream.
    pub(crate) input_sha256: String,
    builtin_derive_syntax: bool,
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
            lexical_scope: Vec::new(),
            input_sha256: zrail_core::sha256_hex(b""),
            builtin_derive_syntax: false,
        }
    }

    pub(crate) fn unresolved(observation: ObservedFact) -> Self {
        let candidate = MacroCandidate::unresolved(observation.clone(), MacroDerivation::Written);
        Self {
            observation,
            candidates: vec![candidate],
            lexical_scope: Vec::new(),
            input_sha256: zrail_core::sha256_hex(b""),
            builtin_derive_syntax: false,
        }
    }

    pub(crate) fn compiler_builtin(observation: ObservedFact) -> Self {
        let candidate = MacroCandidate {
            observation: observation.clone(),
            origins: vec![MacroOrigin::CompilerBuiltin],
            derivation: MacroDerivation::Written,
            written_alias: false,
            definition: None,
            definition_name: None,
            definition_sha256: None,
        };
        Self {
            observation,
            candidates: vec![candidate],
            lexical_scope: Vec::new(),
            input_sha256: zrail_core::sha256_hex(b""),
            builtin_derive_syntax: true,
        }
    }

    pub(crate) fn bind_compiler_candidate(&mut self, resolved: &str) {
        self.candidates
            .retain(|candidate| candidate.observation.name != resolved);
        self.candidates
            .extend(Self::compiler_builtin(self.observation.clone()).candidates);
        self.refresh_quality();
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
            lexical_scope: Vec::new(),
            input_sha256: zrail_core::sha256_hex(b""),
            builtin_derive_syntax: false,
        }
    }

    pub(crate) fn mark_builtin_derive_syntax(&mut self) {
        self.builtin_derive_syntax = true;
    }

    pub(crate) fn with_lexical_scope(mut self, scope: &[zrail_core::SourceSpan]) -> Self {
        self.lexical_scope = scope.to_vec();
        self
    }

    pub(crate) fn with_input_tokens(mut self, tokens: &proc_macro2::TokenStream) -> Self {
        self.input_sha256 = zrail_core::sha256_hex(tokens.to_string().as_bytes());
        self
    }

    pub(crate) const fn has_builtin_derive_syntax(&self) -> bool {
        self.builtin_derive_syntax
    }

    pub(crate) fn refresh_quality(&mut self) {
        self.observation.quality = self
            .candidates
            .iter()
            .map(|candidate| candidate.observation.quality)
            .max()
            .unwrap_or(AnalysisQuality::Unresolved);
    }

    pub(super) fn apply_guard(&mut self, guard: &super::SyntaxGuard) {
        self.observation.apply_guard(guard);
        for candidate in &mut self.candidates {
            candidate.observation.apply_guard(guard);
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
