//! Mutable facts and effective cfg state owned by one syntax traversal.

use super::{
    CompileEffectFact, MacroExpansionFact,
    imports::ImportMap,
    model::{
        CallResolutionFact, ImportBindingFact, IncludeBoundary, MacroDefinitionFact, ObservedFact,
    },
};

#[derive(Debug)]
pub(in crate::source) struct FactVisitor<'a> {
    pub(in crate::source) imports: &'a ImportMap,
    pub(in crate::source) local_imports: super::visitor_imports::LocalImportScopes,
    pub(in crate::source) guard_context: super::SyntaxGuard,
    pub(in crate::source) lexical_scope: Vec<zrail_core::SourceSpan>,
    pub(in crate::source) generic_types: Vec<String>,
    pub(in crate::source) next_path_namespace: super::FactNamespace,
    pub(in crate::source) paths: Vec<ObservedFact>,
    pub(in crate::source) calls: Vec<ObservedFact>,
    pub(in crate::source) call_resolutions: Vec<CallResolutionFact>,
    pub(in crate::source) methods: Vec<ObservedFact>,
    pub(in crate::source) operations: Vec<super::SourceOperationFact>,
    pub(in crate::source) local_types: Vec<super::operation_model::LocalTypes>,
    pub(in crate::source) local_values: super::visitor_values::LocalValueScopes,
    pub(in crate::source) pattern_inputs: Vec<super::visitor_patterns::PatternInputMode>,
    pub(in crate::source) inline_modules: Vec<String>,
    pub(in crate::source) self_types: Vec<super::operation_model::TypeIdentity>,
    pub(in crate::source) field_read_exclusions: Vec<zrail_core::SourceSpan>,
    pub(in crate::source) constructor_path_exclusions: Vec<zrail_core::SourceSpan>,
    pub(in crate::source) macros: Vec<ObservedFact>,
    pub(in crate::source) macro_expansions: Vec<MacroExpansionFact>,
    pub(in crate::source) opaque_macro_inputs: Vec<MacroExpansionFact>,
    pub(in crate::source) macro_definitions: Vec<MacroDefinitionFact>,
    pub(in crate::source) import_bindings: Vec<ImportBindingFact>,
    pub(in crate::source) glob_imports: Vec<super::GlobImportFact>,
    pub(in crate::source) inline_module_scopes: Vec<zrail_core::SourceSpan>,
    pub(in crate::source) compile_effects: Vec<CompileEffectFact>,
    pub(in crate::source) lint_suppressions: Vec<ObservedFact>,
    pub(in crate::source) unsafe_constructs: Vec<ObservedFact>,
    pub(in crate::source) async_syntax: Vec<super::AsyncSyntaxFact>,
    pub(in crate::source) tests: Vec<ObservedFact>,
    pub(in crate::source) includes: Vec<IncludeBoundary>,
    pub(in crate::source) item_macros: Vec<ObservedFact>,
    pub(in crate::source) opaque_binding_macros: Vec<ObservedFact>,
}

impl FactVisitor<'_> {
    pub(in crate::source) fn with_generics(
        &mut self,
        generics: &syn::Generics,
        include_self: bool,
        visit: impl FnOnce(&mut Self),
    ) {
        let checkpoint = self.generic_types.len();
        if include_self {
            self.generic_types.push("Self".into());
        }
        self.generic_types.extend(
            generics
                .params
                .iter()
                .filter_map(|parameter| match parameter {
                    syn::GenericParam::Type(parameter) => Some(parameter.ident.to_string()),
                    _ => None,
                }),
        );
        visit(self);
        self.generic_types.truncate(checkpoint);
    }

    pub(in crate::source) fn with_fresh_generics(&mut self, visit: impl FnOnce(&mut Self)) {
        let inherited = std::mem::take(&mut self.generic_types);
        visit(self);
        self.generic_types = inherited;
    }
}
