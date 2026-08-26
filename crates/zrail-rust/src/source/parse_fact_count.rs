//! Fact counting bounds reusable parse output before projection expands it.

use super::super::MacroExpansionFact;
use super::RustFileFacts;

pub(crate) fn fact_count(file: &RustFileFacts) -> usize {
    file.paths.len()
        + file.calls.len()
        + file.call_resolutions.len()
        + file.methods.len()
        + file.operations.len()
        + file.macros.len()
        + file.macro_imports.len()
        + candidate_count(&file.macro_expansions)
        + candidate_count(&file.opaque_macro_inputs)
        + file.macro_definitions.len()
        + file.import_bindings.len()
        + file.glob_imports.len()
        + file.inline_module_scopes.len()
        + file
            .compile_effects
            .iter()
            .map(|effect| effect.invocation.candidates.len())
            .sum::<usize>()
        + file.lint_suppressions.len()
        + file.unsafe_constructs.len()
        + file.async_syntax.len()
        + file.type_policy.syntax.len()
        + file.type_policy.trait_impls.len()
        + file
            .type_policy
            .declarations
            .iter()
            .map(|declaration| {
                1 + declaration.derives.len() + declaration.fields.as_ref().map_or(0, Vec::len)
            })
            .sum::<usize>()
        + file.tests.len()
        + file.modules.len()
        + file.includes.len()
        + file.item_macros.len()
        + file.opaque_binding_macros.len()
        + file.facade_implementation.len()
}

fn candidate_count(expansions: &[MacroExpansionFact]) -> usize {
    expansions
        .iter()
        .map(|expansion| expansion.candidates.len())
        .sum()
}
