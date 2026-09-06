//! Collect complete item-file facts once, preserving independent structural and syntax policies.

use syn::visit::Visit;
use zrail_core::RustSourceContract;

use super::{authored_expressions, authored_impls, authored_methods, authored_renames, facade};
use crate::{
    inventory::FileClass,
    source::{
        Reachability, RustFileFacts, SourceSyntax, attributes::has_module_docs, imports::ImportMap,
        modules::module_declarations, visitor::FactVisitor,
    },
};

pub(super) fn index_file_with_policy(
    source_file: &crate::inventory::RustSourceFile,
    rust: &RustSourceContract,
    syntax: &syn::File,
) -> RustFileFacts {
    let effective =
        crate::source_policy::effective_file_role(&source_file.relative, source_file.class, rust)
            .effective;
    let mode =
        crate::source_policy::facade_mode_for(&source_file.relative, source_file.class, rust);
    index_file_as(source_file, effective, mode, syntax, &rust.inventories)
}

pub(super) fn index_file_as(
    source_file: &crate::inventory::RustSourceFile,
    effective: FileClass,
    mode: Option<zrail_core::FacadeMode>,
    syntax: &syn::File,
    inventories: &[zrail_core::RustInventoryRule],
) -> RustFileFacts {
    let imports = ImportMap::from_file(syntax);
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_file(syntax);
    let (type_policy, synthetic_paths) = crate::source::type_policy_index::collect(syntax);
    visitor.paths.extend(synthetic_paths);
    let facade_implementation =
        mode.map_or_else(Vec::new, |mode| facade::items(effective, mode, syntax));
    RustFileFacts {
        relative: source_file.relative.clone(),
        packages: Vec::new(),
        class: source_file.class,
        reachability: Reachability::UNREACHABLE,
        syntax: SourceSyntax::Items,
        lines: source_file.lines,
        module_docs: has_module_docs(&syntax.attrs),
        authored_expressions: inventories
            .iter()
            .any(|rule| {
                matches!(
                    rule.subject,
                    zrail_core::RustInventorySubject::WrittenExpressionPaths { .. }
                )
            })
            .then(|| authored_expressions::collect(syntax, &visitor.paths, &visitor.calls)),
        authored_paths: inventories
            .iter()
            .any(|rule| {
                matches!(
                    rule.subject,
                    zrail_core::RustInventorySubject::WrittenPathsContaining { .. }
                )
            })
            .then(|| authored_expressions::collect_all(syntax, &visitor.paths, &visitor.calls)),
        paths: visitor.paths,
        authored_renames: inventories
            .iter()
            .any(|rule| {
                matches!(
                    rule.subject,
                    zrail_core::RustInventorySubject::WrittenImportRenames { .. }
                )
            })
            .then(|| authored_renames::collect(syntax)),
        calls: visitor.calls,
        call_resolutions: visitor.call_resolutions,
        authored_methods: inventories
            .iter()
            .any(|rule| {
                matches!(
                    rule.subject,
                    zrail_core::RustInventorySubject::WrittenMethods { .. }
                )
            })
            .then(|| authored_methods::collect(syntax, &visitor.methods)),
        methods: visitor.methods,
        operations: visitor.operations,
        macros: visitor.macros,
        macro_imports: imports.macro_imports(),
        macro_expansions: visitor.macro_expansions,
        opaque_macro_inputs: visitor.opaque_macro_inputs,
        macro_definitions: visitor.macro_definitions,
        import_bindings: visitor.import_bindings,
        associated_items: visitor.associated_items,
        trait_declarations: visitor.trait_declarations,
        glob_imports: visitor.glob_imports,
        inline_module_scopes: visitor.inline_module_scopes,
        prelude_directives: crate::source::include_bindings::implicit_prelude::directives(syntax),
        compile_effects: visitor.compile_effects,
        lint_suppressions: visitor.lint_suppressions,
        unsafe_constructs: visitor.unsafe_constructs,
        async_syntax: visitor.async_syntax,
        authored_impls: inventories
            .iter()
            .any(|rule| {
                matches!(
                    rule.subject,
                    zrail_core::RustInventorySubject::WrittenTraitImpls { .. }
                )
            })
            .then(|| authored_impls::collect(syntax, &type_policy)),
        type_policy,
        tests: visitor.tests,
        modules: module_declarations(syntax),
        includes: visitor.includes,
        item_macros: visitor.item_macros,
        opaque_binding_macros: visitor.opaque_binding_macros,
        facade_implementation,
    }
}
