//! Context-specific fragment parsers preserve Rust's include-position grammar.

use syn::{
    parse::{Parse, ParseStream},
    visit::Visit,
};

use crate::source::{
    Reachability,
    cfg::cfg_completeness,
    imports::ImportMap,
    model::{RustFileFacts, SourceSyntax},
    type_policy_model::TypePolicyFacts,
    visitor::FactVisitor,
};

pub(super) fn expression(
    source: &crate::inventory::RustSourceFile,
) -> Result<(RustFileFacts, Vec<zrail_core::SourceSpan>), syn::Error> {
    let expression = syn::parse_str::<syn::Expr>(&source.source)?;
    Ok(parsed_expression(source, &expression))
}

pub(super) fn parsed_expression(
    source: &crate::inventory::RustSourceFile,
    expression: &syn::Expr,
) -> (RustFileFacts, Vec<zrail_core::SourceSpan>) {
    let imports = ImportMap::default();
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_expr(expression);
    (
        finish(source, visitor, SourceSyntax::Expression),
        cfg_completeness::expression(expression),
    )
}

pub(super) fn impl_items(
    source: &crate::inventory::RustSourceFile,
) -> Result<(RustFileFacts, Vec<zrail_core::SourceSpan>), syn::Error> {
    let items = syn::parse_str::<ImplItems>(&source.source)?.0;
    let imports = ImportMap::default();
    let mut visitor = FactVisitor::new(&imports);
    for item in &items {
        visitor.visit_impl_item(item);
    }
    Ok((
        finish(source, visitor, SourceSyntax::ImplItems),
        cfg_completeness::impl_items(&items),
    ))
}

pub(super) fn trait_items(
    source: &crate::inventory::RustSourceFile,
) -> Result<(RustFileFacts, Vec<zrail_core::SourceSpan>), syn::Error> {
    let items = syn::parse_str::<TraitItems>(&source.source)?.0;
    let imports = ImportMap::default();
    let mut visitor = FactVisitor::new(&imports);
    for item in &items {
        visitor.visit_trait_item(item);
    }
    Ok((
        finish(source, visitor, SourceSyntax::TraitItems),
        cfg_completeness::trait_items(&items),
    ))
}

struct ImplItems(Vec<syn::ImplItem>);

impl Parse for ImplItems {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self(items))
    }
}

struct TraitItems(Vec<syn::TraitItem>);

impl Parse for TraitItems {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self(items))
    }
}

fn finish(
    source: &crate::inventory::RustSourceFile,
    visitor: FactVisitor<'_>,
    syntax: SourceSyntax,
) -> RustFileFacts {
    RustFileFacts {
        relative: source.relative.clone(),
        packages: Vec::new(),
        class: source.class,
        reachability: Reachability::UNREACHABLE,
        syntax,
        lines: source.lines,
        module_docs: false,
        paths: visitor.paths,
        calls: visitor.calls,
        call_resolutions: visitor.call_resolutions,
        methods: visitor.methods,
        operations: visitor.operations,
        macros: visitor.macros,
        macro_imports: Vec::new(),
        macro_expansions: visitor.macro_expansions,
        opaque_macro_inputs: visitor.opaque_macro_inputs,
        macro_definitions: visitor.macro_definitions,
        import_bindings: visitor.import_bindings,
        associated_items: visitor.associated_items,
        trait_declarations: visitor.trait_declarations,
        glob_imports: visitor.glob_imports,
        inline_module_scopes: visitor.inline_module_scopes,
        prelude_directives: Vec::new(),
        compile_effects: visitor.compile_effects,
        lint_suppressions: visitor.lint_suppressions,
        unsafe_constructs: visitor.unsafe_constructs,
        async_syntax: visitor.async_syntax,
        type_policy: TypePolicyFacts::default(),
        tests: visitor.tests,
        modules: Vec::new(),
        includes: visitor.includes,
        item_macros: visitor.item_macros,
        opaque_binding_macros: visitor.opaque_binding_macros,
        facade_implementation: Vec::new(),
    }
}
