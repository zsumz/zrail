//! Fact visitors begin with resolved import declarations and no cfg context.

use zrail_core::AnalysisQuality;

use super::{FactVisitor, imports::ImportMap, model::ObservedFact};

impl<'a> FactVisitor<'a> {
    pub(in crate::source) fn new(imports: &'a ImportMap) -> Self {
        let mut paths = imports
            .declared_paths()
            .into_iter()
            .map(|(path, quality, guard)| ObservedFact {
                name: path.to_owned(),
                written: None,
                implicit_prelude: super::ImplicitPreludeEligibility::Disabled,
                canonical: Vec::new(),
                span: None,
                quality,
                guard,
                lexical_scope: Vec::new(),
                namespace: super::FactNamespace::Unknown,
            })
            .collect::<Vec<_>>();
        paths.extend(
            imports
                .declared_globs()
                .into_iter()
                .map(|(path, guard)| ObservedFact {
                    name: path.to_owned(),
                    written: None,
                    implicit_prelude: super::ImplicitPreludeEligibility::Disabled,
                    canonical: Vec::new(),
                    span: None,
                    quality: AnalysisQuality::Conservative,
                    guard,
                    lexical_scope: Vec::new(),
                    namespace: super::FactNamespace::Unknown,
                }),
        );
        Self {
            imports,
            local_imports: Vec::new(),
            guard_context: super::SyntaxGuard::Ordinary,
            lexical_scope: Vec::new(),
            generic_types: Vec::new(),
            generic_values: Vec::new(),
            next_path_namespace: super::FactNamespace::Unknown,
            paths,
            calls: Vec::new(),
            call_resolutions: Vec::new(),
            methods: Vec::new(),
            operations: Vec::new(),
            local_types: Vec::new(),
            local_values: Vec::new(),
            pattern_inputs: Vec::new(),
            inline_modules: Vec::new(),
            self_types: Vec::new(),
            field_read_exclusions: Vec::new(),
            constructor_path_exclusions: Vec::new(),
            macros: Vec::new(),
            macro_expansions: Vec::new(),
            opaque_macro_inputs: Vec::new(),
            macro_definitions: Vec::new(),
            import_bindings: Vec::new(),
            associated_items: Vec::new(),
            glob_imports: Vec::new(),
            inline_module_scopes: Vec::new(),
            compile_effects: Vec::new(),
            lint_suppressions: Vec::new(),
            unsafe_constructs: Vec::new(),
            async_syntax: Vec::new(),
            tests: Vec::new(),
            includes: Vec::new(),
            item_macros: Vec::new(),
            opaque_binding_macros: Vec::new(),
        }
    }
}
