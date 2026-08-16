//! Mutable facts and effective cfg state owned by one syntax traversal.

use super::{
    imports::ImportMap,
    model::{
        CompileEffectFact, IncludeBoundary, MacroDefinitionFact, MacroExpansionFact, ObservedFact,
    },
};

#[derive(Debug)]
pub(super) struct FactVisitor<'a> {
    pub(super) imports: &'a ImportMap,
    pub(super) local_imports: super::visitor_imports::LocalImportScopes,
    pub(super) test_only_context: bool,
    pub(super) paths: Vec<ObservedFact>,
    pub(super) calls: Vec<ObservedFact>,
    pub(super) methods: Vec<ObservedFact>,
    pub(super) macros: Vec<ObservedFact>,
    pub(super) macro_expansions: Vec<MacroExpansionFact>,
    pub(super) opaque_macro_inputs: Vec<MacroExpansionFact>,
    pub(super) macro_definitions: Vec<MacroDefinitionFact>,
    pub(super) compile_effects: Vec<CompileEffectFact>,
    pub(super) lint_suppressions: Vec<ObservedFact>,
    pub(super) unsafe_constructs: Vec<ObservedFact>,
    pub(super) tests: Vec<ObservedFact>,
    pub(super) includes: Vec<IncludeBoundary>,
    pub(super) item_macros: Vec<ObservedFact>,
}
