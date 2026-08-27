//! Guarded value bindings expand into conservative receiver candidates.

use zrail_core::AnalysisQuality;

use super::{GuardedValueBinding, SyntaxGuard, TypeIdentity, ValueBinding, ValueCandidate};

pub(super) fn binding_from_identity(identity: TypeIdentity) -> ValueBinding {
    match identity.quality {
        AnalysisQuality::Exact => ValueBinding::Exact(identity),
        AnalysisQuality::Conservative => ValueBinding::Candidates(vec![identity]),
        AnalysisQuality::Unresolved => ValueBinding::Unresolved(identity),
    }
}

pub(super) fn expand_binding(
    binding: &GuardedValueBinding,
    guard: SyntaxGuard,
    candidates: &mut Vec<ValueCandidate>,
) {
    match &binding.value {
        ValueBinding::Exact(identity) | ValueBinding::Unresolved(identity) => {
            candidates.push(ValueCandidate {
                identity: identity.clone(),
                guard,
                input: binding.input,
            });
        }
        ValueBinding::Candidates(identities) => {
            candidates.extend(identities.iter().map(|identity| {
                let mut identity = identity.clone();
                identity.quality = identity.quality.max(AnalysisQuality::Conservative);
                ValueCandidate {
                    identity,
                    guard: guard.clone(),
                    input: binding.input,
                }
            }));
        }
    }
}
