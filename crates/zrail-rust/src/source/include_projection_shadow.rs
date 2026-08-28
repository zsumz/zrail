//! Include instances carry lexical generic and value shadows into projection.

use super::super::{
    FactNamespace, GenericRootShadow, GuardAvailability, ImplicitPreludeEligibility, ObservedFact,
    RootLookupNamespace, generic_root_shadow, include_resolution_state::ResolutionUsage,
    source_instance::SourceInstance,
};

pub(super) fn eligibility(
    fact: &ObservedFact,
    usage: ResolutionUsage,
    source: &SourceInstance,
) -> ImplicitPreludeEligibility {
    let written = fact.written.as_deref().unwrap_or(&fact.name);
    let lookup = root_lookup(fact.namespace, usage, written);
    if let Some(shadow) = generic_root_shadow(
        written,
        lookup,
        &source.generic_types,
        &source.generic_values,
    ) {
        return generic_eligibility(shadow, written);
    }
    inherited_value_shadow(fact, lookup, source).unwrap_or(fact.implicit_prelude)
}

fn generic_eligibility(shadow: GenericRootShadow, written: &str) -> ImplicitPreludeEligibility {
    match shadow {
        GenericRootShadow::TypeParameter
            if written.contains("::")
                || super::super::include_bindings::known_implicit_prelude_name(
                    written.split("::").next().unwrap_or(written),
                ) =>
        {
            ImplicitPreludeEligibility::GenericShadow
        }
        GenericRootShadow::TypeParameter | GenericRootShadow::ConstParameter => {
            ImplicitPreludeEligibility::LocalShadow
        }
    }
}

fn inherited_value_shadow(
    fact: &ObservedFact,
    lookup: RootLookupNamespace,
    source: &SourceInstance,
) -> Option<ImplicitPreludeEligibility> {
    if lookup != RootLookupNamespace::Value {
        return None;
    }
    let written = fact.written.as_deref()?;
    if written.starts_with("::") || written.contains("::") {
        return None;
    }
    let root = written.strip_prefix("r#").unwrap_or(written);
    let availability = source.value_shadow_availability(root, &fact.guard);
    match availability {
        GuardAvailability::Exact => Some(ImplicitPreludeEligibility::LocalShadow),
        GuardAvailability::Possible => Some(ImplicitPreludeEligibility::PossibleShadow),
        GuardAvailability::Absent => None,
    }
}

pub(in crate::source) fn root_lookup(
    namespace: FactNamespace,
    usage: ResolutionUsage,
    written: &str,
) -> RootLookupNamespace {
    match namespace {
        FactNamespace::Type => RootLookupNamespace::Type,
        FactNamespace::Unknown
            if matches!(
                usage,
                ResolutionUsage::Type | ResolutionUsage::OperationType
            ) || written.contains("::") =>
        {
            RootLookupNamespace::Type
        }
        FactNamespace::Value | FactNamespace::Unknown => RootLookupNamespace::Value,
    }
}
