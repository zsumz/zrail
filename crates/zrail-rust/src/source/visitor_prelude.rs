//! Prelude eligibility follows occurrence-local generic and value shadowing.

use super::{
    CfgPredicate, FactVisitor, GenericRootShadow, ImplicitPreludeEligibility, ObservedFact,
    RootLookupNamespace, SyntaxGuard, generic_root_shadow,
};

impl FactVisitor<'_> {
    pub(in crate::source) fn with_implicit_prelude_scope(
        &self,
        facts: impl IntoIterator<Item = ObservedFact>,
        value_position: bool,
    ) -> Vec<ObservedFact> {
        facts
            .into_iter()
            .flat_map(|fact| self.prelude_scoped_fact(fact, value_position))
            .collect()
    }

    fn prelude_scoped_fact(
        &self,
        mut fact: ObservedFact,
        value_position: bool,
    ) -> Vec<ObservedFact> {
        let Some(written) = fact.written.clone() else {
            return vec![fact];
        };
        let Some(root) = implicit_root(&written).map(str::to_owned) else {
            fact.implicit_prelude = ImplicitPreludeEligibility::Disabled;
            return vec![fact];
        };
        let root_name = root.strip_prefix("r#").unwrap_or(&root);
        let lookup = if value_position && !written.contains("::") {
            RootLookupNamespace::Value
        } else {
            RootLookupNamespace::Type
        };
        if let Some(shadow) =
            generic_root_shadow(&written, lookup, &self.generic_types, &self.generic_values)
        {
            fact.implicit_prelude = generic_eligibility(shadow, &root);
            return vec![fact];
        }
        if !value_position || written.contains("::") {
            return vec![fact];
        }
        split_value_shadow(fact, &self.local_value_shadow_guard(root_name))
    }
}

fn generic_eligibility(shadow: GenericRootShadow, root: &str) -> ImplicitPreludeEligibility {
    match shadow {
        GenericRootShadow::TypeParameter
            if crate::source::include_bindings::known_implicit_prelude_name(root) =>
        {
            ImplicitPreludeEligibility::GenericShadow
        }
        GenericRootShadow::TypeParameter | GenericRootShadow::ConstParameter => {
            ImplicitPreludeEligibility::LocalShadow
        }
    }
}

fn implicit_root(written: &str) -> Option<&str> {
    if written.starts_with("::") {
        return None;
    }
    written
        .split("::")
        .next()
        .filter(|root| !root.is_empty())
        .filter(|root| !matches!(*root, "crate" | "self" | "super" | "Self"))
}

fn split_value_shadow(mut fact: ObservedFact, shadow: &SyntaxGuard) -> Vec<ObservedFact> {
    let shadowed = fact.guard.combine(shadow);
    let unshadowed = fact
        .guard
        .combine(SyntaxGuard::from_predicate(CfgPredicate::not(
            shadow.predicate(),
        )));
    let mut facts = Vec::new();
    if shadowed.predicate().is_satisfiable() != Some(false) {
        let mut local = fact.clone();
        local.guard = shadowed;
        local.implicit_prelude = ImplicitPreludeEligibility::LocalShadow;
        facts.push(local);
    }
    if unshadowed.predicate().is_satisfiable() != Some(false) {
        fact.guard = unshadowed;
        facts.push(fact);
    }
    facts
}
