//! Parse each Rust file once and retain reusable architecture facts.

#[path = "parse_fact_count.rs"]
mod parse_fact_count;

use syn::visit::Visit;
use zrail_core::{AnalysisQuality, Finding, RustSourceContract};

use crate::inventory::{FileClass, RepositoryInventory};

use super::{
    Reachability,
    attributes::has_module_docs,
    depth::check_syntax_depth,
    imports::ImportMap,
    model::{RustFileFacts, SourceIndex, SourceSyntax},
    modules::module_declarations,
    visitor::FactVisitor,
};

pub(crate) use parse_fact_count::fact_count;

pub(super) const MAX_FACTS_PER_FILE: usize = 50_000;

pub(crate) fn index_rust_source(
    inventory: &RepositoryInventory,
    rust: &RustSourceContract,
) -> SourceIndex {
    let mut index = SourceIndex::default();
    for source_file in &inventory.rust_files {
        if let Err(error) = check_syntax_depth(&source_file.source) {
            index
                .findings
                .push(analysis_limit(&source_file.relative, error));
            continue;
        }
        match parse_source(source_file, rust) {
            Ok((facts, incomplete_cfg)) => {
                if !rust.feature_worlds.is_empty() {
                    index.findings.extend(incomplete_cfg.into_iter().map(|span| {
                        Finding::error(
                            "RUST-CFG-001",
                            "rust.source.feature-world",
                            "source",
                            "feature-dependent cfg_attr changes path or test target identity",
                        )
                        .at(&source_file.relative, Some(span))
                        .with_analysis(AnalysisQuality::Unresolved)
                        .with_help(
                            "use direct cfg(feature = ...) items or make the attribute identity unconditional",
                        )
                    }));
                }
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
    rust: &RustSourceContract,
) -> Result<(RustFileFacts, Vec<zrail_core::SourceSpan>), syn::Error> {
    match syn::parse_file(&source_file.source) {
        Ok(syntax) => Ok((
            index_file_with_policy(source_file, rust, &syntax),
            super::cfg::cfg_completeness::file(&syntax),
        )),
        Err(file_error) => match syn::parse_str::<syn::Expr>(&source_file.source) {
            Ok(expression) => Ok((
                index_expression(source_file, &expression),
                super::cfg::cfg_completeness::expression(&expression),
            )),
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

#[cfg(test)]
fn index_file(source_file: &crate::inventory::RustSourceFile, syntax: &syn::File) -> RustFileFacts {
    index_file_as(source_file, source_file.class, syntax)
}

fn index_file_with_policy(
    source_file: &crate::inventory::RustSourceFile,
    rust: &RustSourceContract,
    syntax: &syn::File,
) -> RustFileFacts {
    let effective =
        crate::source_policy::effective_file_role(&source_file.relative, source_file.class, rust)
            .effective;
    index_file_as(source_file, effective, syntax)
}

fn index_file_as(
    source_file: &crate::inventory::RustSourceFile,
    effective: FileClass,
    syntax: &syn::File,
) -> RustFileFacts {
    let imports = ImportMap::from_file(syntax);
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_file(syntax);
    let (type_policy, synthetic_paths) = super::type_policy_index::collect(syntax);
    visitor.paths.extend(synthetic_paths);
    let facade_implementation = if matches!(effective, FileClass::Facade | FileClass::EntryPoint) {
        super::parse_facade::items(&source_file.relative, syntax)
    } else {
        Vec::new()
    };
    RustFileFacts {
        relative: source_file.relative.clone(),
        packages: Vec::new(),
        class: source_file.class,
        reachability: Reachability::UNREACHABLE,
        syntax: SourceSyntax::Items,
        lines: source_file.lines,
        module_docs: has_module_docs(&syntax.attrs),
        paths: visitor.paths,
        calls: visitor.calls,
        call_resolutions: visitor.call_resolutions,
        methods: visitor.methods,
        operations: visitor.operations,
        macros: visitor.macros,
        macro_imports: imports.macro_imports(),
        macro_expansions: visitor.macro_expansions,
        opaque_macro_inputs: visitor.opaque_macro_inputs,
        macro_definitions: visitor.macro_definitions,
        import_bindings: visitor.import_bindings,
        associated_items: visitor.associated_items,
        glob_imports: visitor.glob_imports,
        inline_module_scopes: visitor.inline_module_scopes,
        compile_effects: visitor.compile_effects,
        lint_suppressions: visitor.lint_suppressions,
        unsafe_constructs: visitor.unsafe_constructs,
        async_syntax: visitor.async_syntax,
        type_policy,
        tests: visitor.tests,
        modules: module_declarations(syntax),
        includes: visitor.includes,
        item_macros: visitor.item_macros,
        opaque_binding_macros: visitor.opaque_binding_macros,
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
        reachability: Reachability::UNREACHABLE,
        syntax: SourceSyntax::Expression,
        lines: source_file.lines,
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
        glob_imports: visitor.glob_imports,
        inline_module_scopes: visitor.inline_module_scopes,
        compile_effects: visitor.compile_effects,
        lint_suppressions: visitor.lint_suppressions,
        unsafe_constructs: visitor.unsafe_constructs,
        async_syntax: visitor.async_syntax,
        type_policy: super::type_policy_model::TypePolicyFacts::default(),
        tests: visitor.tests,
        modules: Vec::new(),
        includes: visitor.includes,
        item_macros: visitor.item_macros,
        opaque_binding_macros: visitor.opaque_binding_macros,
        facade_implementation: Vec::new(),
    }
}

#[cfg(test)]
#[path = "parse_test.rs"]
mod parse_test;
