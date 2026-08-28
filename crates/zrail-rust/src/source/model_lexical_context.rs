//! Lexical type context shared by ordinary facts and include instances.

use zrail_core::AnalysisQuality;

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
}

impl GenericAssociatedCandidate {
    pub(crate) fn policy_names(&self) -> impl Iterator<Item = &str> {
        self.canonical
            .iter()
            .map(String::as_str)
            .chain(self.canonical.is_empty().then_some(self.name.as_str()))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct GenericParameterBounds {
    pub(crate) parameter: String,
    pub(crate) traits: Vec<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct LexicalSelfIdentity {
    pub(crate) name: String,
    pub(crate) quality: AnalysisQuality,
    pub(crate) file_local: bool,
}
