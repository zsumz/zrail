//! Mutable facts and effective cfg state owned by one syntax traversal.

use super::{
    CompileEffectFact, MacroExpansionFact,
    imports::ImportMap,
    model::{
        CallResolutionFact, ImportBindingFact, IncludeBoundary, MacroDefinitionFact, ObservedFact,
    },
};

#[derive(Debug)]
pub(super) struct FactVisitor<'a> {
    pub(super) imports: &'a ImportMap,
    pub(super) local_imports: super::visitor_imports::LocalImportScopes,
    pub(super) guard_context: super::SyntaxGuard,
    pub(super) lexical_scope: Vec<zrail_core::SourceSpan>,
    pub(super) generic_types: Vec<String>,
    pub(super) next_path_namespace: super::FactNamespace,
    pub(super) paths: Vec<ObservedFact>,
    pub(super) calls: Vec<ObservedFact>,
    pub(super) call_resolutions: Vec<CallResolutionFact>,
    pub(super) methods: Vec<ObservedFact>,
    pub(super) operations: Vec<super::SourceOperationFact>,
    pub(super) local_types: Vec<super::operation_model::LocalTypes>,
    pub(super) inline_modules: Vec<String>,
    pub(super) self_types: Vec<super::operation_model::TypeIdentity>,
    pub(super) field_read_exclusions: Vec<zrail_core::SourceSpan>,
    pub(super) macros: Vec<ObservedFact>,
    pub(super) macro_expansions: Vec<MacroExpansionFact>,
    pub(super) opaque_macro_inputs: Vec<MacroExpansionFact>,
    pub(super) macro_definitions: Vec<MacroDefinitionFact>,
    pub(super) import_bindings: Vec<ImportBindingFact>,
    pub(super) inline_module_scopes: Vec<zrail_core::SourceSpan>,
    pub(super) compile_effects: Vec<CompileEffectFact>,
    pub(super) lint_suppressions: Vec<ObservedFact>,
    pub(super) unsafe_constructs: Vec<ObservedFact>,
    pub(super) tests: Vec<ObservedFact>,
    pub(super) includes: Vec<IncludeBoundary>,
    pub(super) item_macros: Vec<ObservedFact>,
    pub(super) opaque_binding_macros: Vec<ObservedFact>,
}

impl FactVisitor<'_> {
    pub(super) fn with_generics(
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

    pub(super) fn with_fresh_generics(&mut self, visit: impl FnOnce(&mut Self)) {
        let inherited = std::mem::take(&mut self.generic_types);
        visit(self);
        self.generic_types = inherited;
    }
}
