//! Deterministic evidence explaining why a named macro allowance did not bind.

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum MacroBindingFailure {
    UnresolvedOrigin {
        candidate: String,
    },
    PendingOrigin {
        candidate: String,
    },
    SourceMismatch {
        allowance: String,
        expected: String,
        observed: Vec<String>,
    },
    DefinitionMismatch {
        allowance: String,
        configured: String,
        observed_packages: Vec<String>,
    },
    CandidateNotCovered {
        candidate: String,
        missing: Vec<String>,
    },
    ConfidenceNotGranted {
        allowance: String,
    },
}

impl MacroBindingFailure {
    pub(super) fn summary(&self) -> String {
        match self {
            Self::UnresolvedOrigin { candidate } => {
                format!("candidate {candidate:?} has an unresolved origin")
            }
            Self::PendingOrigin { candidate } => {
                format!("candidate {candidate:?} has a pending origin")
            }
            Self::SourceMismatch {
                allowance,
                expected,
                observed,
            } => format!(
                "allowance {allowance:?} expects source {expected:?}, observed {}",
                listed(observed)
            ),
            Self::DefinitionMismatch {
                allowance,
                configured,
                observed_packages,
            } => format!(
                "allowance {allowance:?} configures definition {configured:?}, observed implementation package(s) {}",
                listed(observed_packages)
            ),
            Self::CandidateNotCovered { candidate, missing } => format!(
                "candidate {candidate:?} is not covered; missing allowance(s) {}",
                listed(missing)
            ),
            Self::ConfidenceNotGranted { allowance } => {
                format!("allowance {allowance:?} does not grant conservative name-only binding")
            }
        }
    }

    pub(super) fn help(&self) -> &'static str {
        match self {
            Self::UnresolvedOrigin { .. }
            | Self::PendingOrigin { .. }
            | Self::ConfidenceNotGranted { .. } => {
                "qualify or resolve the macro origin, or use binding = \"conservative\" after reviewing the unresolved name-only boundary"
            }
            Self::SourceMismatch { .. } => {
                "bind source to the exact observed external package source"
            }
            Self::DefinitionMismatch { .. } => {
                "bind definition to one macro_rules! definition in the observed repository implementation package"
            }
            Self::CandidateNotCovered { .. } => {
                "add reasoned allowances for every feasible macro candidate"
            }
        }
    }
}

fn listed(values: &[String]) -> String {
    if values.is_empty() {
        "<none>".into()
    } else {
        values
            .iter()
            .map(|value| format!("{value:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
