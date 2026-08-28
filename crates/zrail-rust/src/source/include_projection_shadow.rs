//! Include instances carry lexical generic and value shadows into projection.

use super::super::{
    FactNamespace, GenericRootIdentity, GenericRootShadow, GuardAvailability,
    ImplicitPreludeEligibility, ObservedFact, RootLookupNamespace, generic_root_identity,
    identity_for_generic_root, include_resolution_state::ResolutionUsage,
    source_instance::SourceInstance,
};

pub(super) fn eligibility(
    fact: &ObservedFact,
    usage: ResolutionUsage,
    source: &SourceInstance,
) -> ImplicitPreludeEligibility {
    if let Some(identity) = generic_identity(fact, usage, Some(source)) {
        return generic_eligibility(
            identity.shadow,
            fact.written.as_deref().unwrap_or(&fact.name),
        );
    }
    let written = fact.written.as_deref().unwrap_or(&fact.name);
    let lookup = root_lookup(fact.namespace, usage, written);
    inherited_value_shadow(fact, lookup, source).unwrap_or(fact.implicit_prelude)
}

pub(super) fn generic_identity(
    fact: &ObservedFact,
    usage: ResolutionUsage,
    source: Option<&SourceInstance>,
) -> Option<GenericRootIdentity> {
    let written = fact.written.as_deref().unwrap_or(&fact.name);
    if let Some(shadow) = fact.generic_shadow {
        return Some(identity_for_generic_root(written, shadow));
    }
    if !fact.inherits_parent_context {
        return None;
    }
    let source = source?;
    generic_root_identity(
        written,
        generic_root_lookup(fact.namespace, usage, written),
        &source.generic_types,
        &source.generic_values,
    )
    .or_else(|| {
        (written.starts_with("Self::")
            && source.generic_types.iter().any(|generic| generic == "Self"))
        .then(|| identity_for_generic_root(written, GenericRootShadow::TypeParameter))
    })
}

fn generic_root_lookup(
    namespace: FactNamespace,
    usage: ResolutionUsage,
    written: &str,
) -> RootLookupNamespace {
    if written.contains("::") {
        RootLookupNamespace::Type
    } else {
        root_lookup(namespace, usage, written)
    }
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
    if !fact.inherits_parent_context {
        return None;
    }
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
