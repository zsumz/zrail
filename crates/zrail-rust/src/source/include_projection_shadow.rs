//! Include instances carry lexical generic and value shadows into projection.

use super::super::{
    GuardAvailability, ImplicitPreludeEligibility, ObservedFact,
    include_resolution_state::ResolutionUsage, source_instance::SourceInstance,
};

pub(super) fn eligibility(
    fact: &ObservedFact,
    usage: ResolutionUsage,
    source: &SourceInstance,
) -> ImplicitPreludeEligibility {
    if generic_root(fact, &source.generic_types) {
        return ImplicitPreludeEligibility::GenericShadow;
    }
    inherited_value_shadow(fact, usage, source).unwrap_or(fact.implicit_prelude)
}

fn generic_root(fact: &ObservedFact, generic_types: &[String]) -> bool {
    let Some(written) = fact.written.as_deref() else {
        return false;
    };
    if written.starts_with("::") {
        return false;
    }
    let root = written.split("::").next();
    root.is_some_and(|root| {
        !matches!(root, "crate" | "self" | "super" | "Self")
            && generic_types.iter().any(|generic| {
                generic.strip_prefix("r#").unwrap_or(generic)
                    == root.strip_prefix("r#").unwrap_or(root)
            })
    })
}

fn inherited_value_shadow(
    fact: &ObservedFact,
    usage: ResolutionUsage,
    source: &SourceInstance,
) -> Option<ImplicitPreludeEligibility> {
    if fact.implicit_prelude != ImplicitPreludeEligibility::Eligible
        || !matches!(
            usage,
            ResolutionUsage::Path | ResolutionUsage::Call | ResolutionUsage::ConstructorValue
        )
    {
        return None;
    }
    let written = fact.written.as_deref()?;
    if written.starts_with("::") || written.contains("::") {
        return None;
    }
    let root = written.strip_prefix("r#").unwrap_or(written);
    let context = source.guard.combine(&fact.guard);
    let availability = source
        .prelude_value_shadows
        .iter()
        .filter(|(name, _)| name == root)
        .fold(GuardAvailability::Absent, |current, (_, guard)| {
            merge(
                current,
                guard.availability_for_domain(&context, &source.domain),
            )
        });
    match availability {
        GuardAvailability::Exact => Some(ImplicitPreludeEligibility::LocalShadow),
        GuardAvailability::Possible => Some(ImplicitPreludeEligibility::PossibleShadow),
        GuardAvailability::Absent => None,
    }
}

const fn merge(left: GuardAvailability, right: GuardAvailability) -> GuardAvailability {
    match (left, right) {
        (GuardAvailability::Exact, _) | (_, GuardAvailability::Exact) => GuardAvailability::Exact,
        (GuardAvailability::Possible, _) | (_, GuardAvailability::Possible) => {
            GuardAvailability::Possible
        }
        _ => GuardAvailability::Absent,
    }
}
