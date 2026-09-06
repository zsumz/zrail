//! Parse each Rust file once and retain reusable architecture facts.

#[path = "authored_expressions.rs"]
mod authored_expressions;
#[path = "authored_methods.rs"]
mod authored_methods;
#[path = "authored_renames.rs"]
mod authored_renames;
#[path = "parse_facade.rs"]
mod facade;
#[path = "parse_fact_count.rs"]
mod parse_fact_count;
#[path = "parse_fragments.rs"]
mod parse_fragments;
#[path = "parse_items.rs"]
mod parse_items;

use std::collections::BTreeMap;

use zrail_core::{AnalysisQuality, Finding, RustSourceContract};

#[cfg(test)]
use crate::inventory::FileClass;
use crate::inventory::RepositoryInventory;

use super::{
    depth::check_syntax_depth,
    model::{RustFileFacts, SourceIndex, SourceSyntax},
};

pub(crate) use parse_fact_count::fact_count;

pub(super) const MAX_FACTS_PER_FILE: usize = 50_000;

#[cfg(test)]
pub(crate) fn index_rust_source(
    inventory: &RepositoryInventory,
    rust: &RustSourceContract,
) -> SourceIndex {
    index_rust_source_with_hints(inventory, rust, &BTreeMap::new())
}

pub(crate) fn index_rust_source_with_hints(
    inventory: &RepositoryInventory,
    rust: &RustSourceContract,
    syntax_hints: &BTreeMap<String, std::collections::BTreeSet<SourceSyntax>>,
) -> SourceIndex {
    let mut index = SourceIndex::default();
    for source_file in &inventory.rust_files {
        if let Err(error) = check_syntax_depth(&source_file.source) {
            index
                .findings
                .push(analysis_limit(&source_file.relative, error));
            continue;
        }
        let requested = syntax_hints.get(&source_file.relative);
        let syntaxes = requested.map_or_else(
            || vec![None],
            |syntaxes| syntaxes.iter().copied().map(Some).collect::<Vec<_>>(),
        );
        for syntax in syntaxes {
            match parse_source(source_file, rust, syntax) {
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
    }
    index
        .files
        .sort_by(|left, right| (&left.relative, left.syntax).cmp(&(&right.relative, right.syntax)));
    index
}

fn parse_source(
    source_file: &crate::inventory::RustSourceFile,
    rust: &RustSourceContract,
    syntax_hint: Option<SourceSyntax>,
) -> Result<(RustFileFacts, Vec<zrail_core::SourceSpan>), syn::Error> {
    match syntax_hint {
        Some(SourceSyntax::Expression) => return parse_fragments::expression(source_file),
        Some(SourceSyntax::ImplItems) => return parse_fragments::impl_items(source_file),
        Some(SourceSyntax::TraitItems) => return parse_fragments::trait_items(source_file),
        Some(SourceSyntax::Items) => {
            let syntax = syn::parse_file(&source_file.source)?;
            return Ok((
                parse_items::index_file_with_policy(source_file, rust, &syntax),
                super::cfg::cfg_completeness::file(&syntax),
            ));
        }
        None => {}
    }
    match syn::parse_file(&source_file.source) {
        Ok(syntax) => Ok((
            parse_items::index_file_with_policy(source_file, rust, &syntax),
            super::cfg::cfg_completeness::file(&syntax),
        )),
        Err(file_error) => match syn::parse_str::<syn::Expr>(&source_file.source) {
            Ok(expression) => Ok(parse_fragments::parsed_expression(source_file, &expression)),
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
    let mode = matches!(source_file.class, FileClass::Facade | FileClass::EntryPoint)
        .then_some(zrail_core::FacadeMode::Declarative);
    parse_items::index_file_as(source_file, source_file.class, mode, syntax, &[])
}

#[cfg(test)]
fn index_expression(
    source_file: &crate::inventory::RustSourceFile,
    expression: &syn::Expr,
) -> RustFileFacts {
    parse_fragments::parsed_expression(source_file, expression).0
}

#[cfg(test)]
#[path = "parse_test.rs"]
mod parse_test;
