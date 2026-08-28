//! Lexical type context shared by ordinary facts and include instances.

use std::collections::BTreeSet;

use zrail_core::AnalysisQuality;

use super::SyntaxGuard;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum AssociatedOccurrenceKind {
    DirectCall,
    ValueReference,
    TypeReference,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct GenericAssociatedCandidate {
    pub(crate) name: String,
    pub(crate) canonical: Vec<String>,
    pub(crate) quality: AnalysisQuality,
    pub(crate) projection: Vec<String>,
    pub(crate) provider_complete: bool,
    pub(crate) provider_authorities: BTreeSet<ProviderAuthority>,
}

impl GenericAssociatedCandidate {
    pub(crate) fn policy_names(&self) -> impl Iterator<Item = &str> {
        self.canonical
            .iter()
            .filter(|_| self.projection.is_empty())
            .map(String::as_str)
            .chain(
                (self.projection.is_empty() && self.canonical.is_empty())
                    .then_some(self.name.as_str()),
            )
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ProviderAuthority {
    LocalCrate,
    ExternalRoot(String),
    Unknown,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct TraitBoundFact {
    pub(crate) subject: super::BoundSubject,
    pub(crate) providers: Vec<String>,
    pub(crate) quality: AnalysisQuality,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<zrail_core::SourceSpan>,
    pub(crate) span: zrail_core::SourceSpan,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct LexicalSelfIdentity {
    pub(crate) name: String,
    pub(crate) quality: AnalysisQuality,
    pub(crate) file_local: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TraitDeclarationFact {
    pub(crate) trait_path: String,
    pub(crate) bounds: Vec<TraitBoundFact>,
    pub(crate) quality: AnalysisQuality,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<zrail_core::SourceSpan>,
    pub(crate) span: zrail_core::SourceSpan,
}
