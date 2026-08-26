//! Exact feature worlds remove syntax absent from every governed compilation domain.

use std::collections::{BTreeMap, BTreeSet};

use super::{CompilationDomain, GuardAvailability, MacroExpansionFact, SourceIndex, SyntaxGuard};

pub(crate) fn retain_active_facts(
    source: &mut SourceIndex,
    compilation_domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
    exact_worlds: bool,
) {
    if !exact_worlds {
        return;
    }
    for file in &mut source.files {
        let domains = compilation_domains.get(&file.relative);
        let active = |guard: &SyntaxGuard| active_in_any(guard, domains);
        file.paths.retain(|fact| active(&fact.guard));
        file.calls.retain(|fact| active(&fact.guard));
        file.call_resolutions.retain(|fact| active(&fact.guard));
        file.methods.retain(|fact| active(&fact.guard));
        file.operations.retain(|fact| active(&fact.identity.guard));
        file.macros.retain(|fact| active(&fact.guard));
        file.macro_imports.retain(|fact| active(&fact.guard));
        retain_expansions(&mut file.macro_expansions, domains);
        retain_expansions(&mut file.opaque_macro_inputs, domains);
        file.macro_definitions.retain(|fact| active(&fact.guard));
        file.import_bindings.retain(|fact| active(&fact.guard));
        file.glob_imports.retain(|fact| active(&fact.guard));
        file.compile_effects
            .retain(|fact| active(&fact.invocation.observation.guard));
        file.lint_suppressions.retain(|fact| active(&fact.guard));
        file.unsafe_constructs.retain(|fact| active(&fact.guard));
        file.async_syntax
            .retain(|fact| active(&fact.observation.guard));
        for declaration in &mut file.type_policy.declarations {
            declaration.derives.retain(|fact| active(&fact.guard));
        }
        file.type_policy
            .declarations
            .retain(|fact| active(&fact.guard));
        file.type_policy
            .trait_impls
            .retain(|fact| active(&fact.guard));
        file.type_policy.syntax.retain(|fact| active(&fact.guard));
        file.tests.retain(|fact| active(&fact.guard));
        file.modules.retain(|fact| active(&fact.guard));
        file.includes.retain(|fact| active(&fact.guard));
        file.item_macros.retain(|fact| active(&fact.guard));
        file.opaque_binding_macros
            .retain(|fact| active(&fact.guard));
        file.facade_implementation
            .retain(|fact| active(&fact.guard));
    }
}

fn retain_expansions(
    facts: &mut Vec<MacroExpansionFact>,
    domains: Option<&BTreeSet<CompilationDomain>>,
) {
    facts.retain_mut(|fact| {
        if !active_in_any(&fact.observation.guard, domains) {
            return false;
        }
        fact.candidates
            .retain(|candidate| active_in_any(&candidate.observation.guard, domains));
        fact.refresh_quality();
        true
    });
}

fn active_in_any(guard: &SyntaxGuard, domains: Option<&BTreeSet<CompilationDomain>>) -> bool {
    domains.is_some_and(|domains| {
        domains
            .iter()
            .any(|domain| guard.availability_in_domain(domain) != GuardAvailability::Absent)
    })
}

#[cfg(test)]
#[path = "active_facts_test.rs"]
mod active_facts_test;
