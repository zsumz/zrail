//! Empty physical source facts isolate projection fixture inputs.

use crate::{
    inventory::FileClass,
    source::{ImportBindingFact, ObservedFact, Reachability, RustFileFacts, SourceSyntax},
};

pub(super) fn file(
    relative: &str,
    calls: Vec<ObservedFact>,
    import_bindings: Vec<ImportBindingFact>,
) -> RustFileFacts {
    RustFileFacts {
        relative: relative.into(),
        packages: Vec::new(),
        class: FileClass::Implementation,
        reachability: Reachability::UNREACHABLE,
        syntax: SourceSyntax::Items,
        lines: 1,
        module_docs: true,
        paths: Vec::new(),
        calls,
        call_resolutions: Vec::new(),
        methods: Vec::new(),
        authored_methods: None,
        authored_expressions: None,
        authored_paths: None,
        authored_renames: None,
        operations: Vec::new(),
        macros: Vec::new(),
        macro_imports: Vec::new(),
        macro_expansions: Vec::new(),
        opaque_macro_inputs: Vec::new(),
        macro_definitions: Vec::new(),
        import_bindings,
        associated_items: Vec::new(),
        trait_declarations: Vec::new(),
        glob_imports: Vec::new(),
        inline_module_scopes: Vec::new(),
        prelude_directives: Vec::new(),
        compile_effects: Vec::new(),
        lint_suppressions: Vec::new(),
        unsafe_constructs: Vec::new(),
        async_syntax: Vec::new(),
        type_policy: crate::source::TypePolicyFacts::default(),
        tests: Vec::new(),
        modules: Vec::new(),
        includes: Vec::new(),
        item_macros: Vec::new(),
        opaque_binding_macros: Vec::new(),
        facade_implementation: Vec::new(),
    }
}
