//! Parse each Rust file once and retain reusable architecture facts.

use syn::{Item, spanned::Spanned, visit::Visit};
use zrail_core::{AnalysisQuality, Finding};

use crate::inventory::{FileClass, RepositoryInventory};

use super::{
    attributes::has_module_docs,
    depth::check_syntax_depth,
    fact::fact,
    imports::ImportMap,
    model::{ObservedFact, Reachability, RustFileFacts, SourceIndex, SourceSyntax},
    modules::module_declarations,
    visitor::FactVisitor,
};

const MAX_FACTS_PER_FILE: usize = 50_000;
const MAX_TOTAL_SOURCE_FACTS: usize = 1_000_000;

pub(crate) fn index_rust_source(inventory: &RepositoryInventory) -> SourceIndex {
    let mut index = SourceIndex::default();
    let mut total_facts = 0_usize;
    for source_file in &inventory.rust_files {
        if let Err(error) = check_syntax_depth(&source_file.source) {
            index
                .findings
                .push(analysis_limit(&source_file.relative, error));
            continue;
        }
        match parse_source(source_file) {
            Ok(facts) => {
                let count = fact_count(&facts);
                if count > MAX_FACTS_PER_FILE {
                    index.findings.push(analysis_limit(
                        &source_file.relative,
                        format!(
                            "Rust source exceeds the {MAX_FACTS_PER_FILE}-fact per-file safety limit"
                        ),
                    ));
                    continue;
                }
                total_facts = total_facts.saturating_add(count);
                if total_facts > MAX_TOTAL_SOURCE_FACTS {
                    index.findings.push(analysis_limit(
                        &source_file.relative,
                        format!(
                            "repository exceeds the {MAX_TOTAL_SOURCE_FACTS}-fact Rust analysis safety limit"
                        ),
                    ));
                    break;
                }
                index.files.push(facts);
            }
            Err(error) => index.findings.push(
                Finding::error(
                    "RUST-PARSE-001",
                    "rust.parse",
                    "source",
                    format!("Rust source could not be parsed: {error}"),
                )
                .at(&source_file.relative, None)
                .with_analysis(AnalysisQuality::Unresolved)
                .with_help("fix the syntax error before trusting architecture analysis"),
            ),
        }
    }
    index
        .files
        .sort_by(|left, right| left.relative.cmp(&right.relative));
    index
}

fn parse_source(
    source_file: &crate::inventory::RustSourceFile,
) -> Result<RustFileFacts, syn::Error> {
    match syn::parse_file(&source_file.source) {
        Ok(syntax) => Ok(index_file(source_file, &syntax)),
        Err(file_error) => match syn::parse_str::<syn::Expr>(&source_file.source) {
            Ok(expression) => Ok(index_expression(source_file, &expression)),
            Err(_) => Err(file_error),
        },
    }
}

fn analysis_limit(path: &str, message: String) -> Finding {
    Finding::error("RUST-PARSE-002", "rust.parse.limit", "source", message)
        .at(path, None)
        .with_analysis(AnalysisQuality::Unresolved)
        .with_help("reduce the source input before trusting architecture analysis")
}

fn fact_count(file: &RustFileFacts) -> usize {
    file.paths.len()
        + file.calls.len()
        + file.methods.len()
        + file.macros.len()
        + file.lint_suppressions.len()
        + file.unsafe_constructs.len()
        + file.tests.len()
        + file.modules.len()
        + file.includes.len()
        + file.item_macros.len()
        + file.facade_implementation.len()
}

fn index_file(source_file: &crate::inventory::RustSourceFile, syntax: &syn::File) -> RustFileFacts {
    let imports = ImportMap::from_file(syntax);
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_file(syntax);
    let facade_implementation =
        if matches!(source_file.class, FileClass::Facade | FileClass::EntryPoint) {
            facade_items(&source_file.relative, syntax)
        } else {
            Vec::new()
        };
    RustFileFacts {
        relative: source_file.relative.clone(),
        packages: Vec::new(),
        class: source_file.class,
        reachability: Reachability::Unreachable,
        syntax: SourceSyntax::Items,
        lines: source_file.lines,
        module_docs: has_module_docs(&syntax.attrs),
        paths: visitor.paths,
        calls: visitor.calls,
        methods: visitor.methods,
        macros: visitor.macros,
        lint_suppressions: visitor.lint_suppressions,
        unsafe_constructs: visitor.unsafe_constructs,
        tests: visitor.tests,
        modules: module_declarations(syntax),
        includes: visitor.includes,
        item_macros: visitor.item_macros,
        facade_implementation,
    }
}

fn index_expression(
    source_file: &crate::inventory::RustSourceFile,
    expression: &syn::Expr,
) -> RustFileFacts {
    let imports = ImportMap::default();
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_expr(expression);
    RustFileFacts {
        relative: source_file.relative.clone(),
        packages: Vec::new(),
        class: source_file.class,
        reachability: Reachability::Unreachable,
        syntax: SourceSyntax::Expression,
        lines: source_file.lines,
        module_docs: false,
        paths: visitor.paths,
        calls: visitor.calls,
        methods: visitor.methods,
        macros: visitor.macros,
        lint_suppressions: visitor.lint_suppressions,
        unsafe_constructs: visitor.unsafe_constructs,
        tests: visitor.tests,
        modules: Vec::new(),
        includes: visitor.includes,
        item_macros: visitor.item_macros,
        facade_implementation: Vec::new(),
    }
}

fn facade_items(relative: &str, syntax: &syn::File) -> Vec<ObservedFact> {
    syntax
        .items
        .iter()
        .filter_map(|item| {
            if declarative_item(relative, item) {
                None
            } else {
                let span = match item {
                    Item::Macro(item_macro) => item_macro.mac.span(),
                    _ => item.span(),
                };
                Some(fact(item_kind(item), span, AnalysisQuality::Exact))
            }
        })
        .collect()
}

fn declarative_item(relative: &str, item: &Item) -> bool {
    match item {
        Item::Mod(module) if module.content.is_none() => true,
        Item::Use(_) | Item::ExternCrate(_) => true,
        Item::Fn(function) => relative.ends_with("/main.rs") && function.sig.ident == "main",
        _ => false,
    }
}

fn item_kind(item: &Item) -> String {
    match item {
        Item::Const(_) => "const".into(),
        Item::Enum(_) => "enum".into(),
        Item::Fn(_) => "function".into(),
        Item::Impl(_) => "impl".into(),
        Item::Macro(item_macro) => item_macro
            .mac
            .path
            .segments
            .last()
            .map_or_else(|| "macro".into(), |segment| format!("{}!", segment.ident)),
        Item::Static(_) => "static".into(),
        Item::Struct(_) => "struct".into(),
        Item::Trait(_) => "trait".into(),
        Item::Type(_) => "type".into(),
        _ => "item".into(),
    }
}

#[cfg(test)]
#[path = "parse_test.rs"]
mod parse_test;
