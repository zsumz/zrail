//! Syntax-preserving formatting and schema-key migration for one contract source.

mod macro_keys;

use std::{error::Error, fmt, ops::Range};

use toml_edit::{ImDocument, Item, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
/// A contract source could not be parsed, transformed, or rendered.
pub struct ContractEditError(String);

impl fmt::Display for ContractEditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for ContractEditError {}

#[derive(Debug)]
pub(super) struct Replacement {
    span: Range<usize>,
    text: String,
}

/// Validates one TOML contract source without rewriting authored layout.
///
/// Comments, blank lines, key order, indentation, quoting, and markers remain
/// byte-for-byte identical. A missing final newline is the only formatting
/// change.
pub fn format_contract_source(source: &str) -> Result<String, ContractEditError> {
    parse(source)?;
    Ok(with_trailing_newline(source.to_owned()))
}

/// Migrates one contract source to schema-2 keys without reserializing it.
///
/// `entry` identifies the root source that owns the schema declaration.
/// `exact_imports` contains the repository-bounded paths resolved by the
/// caller. Legacy patterns are expanded in place and every other authored byte
/// is preserved.
pub fn migrate_contract_source(
    source: &str,
    entry: bool,
    exact_imports: &[String],
) -> Result<String, ContractEditError> {
    let document = parse(source)?;
    let mut replacements = Vec::new();
    if entry {
        migrate_schema(&document, &mut replacements)?;
    }
    migrate_imports(&document, exact_imports, &mut replacements)?;
    macro_keys::migrate_macro_keys(&document, source, &mut replacements)?;
    let rendered = apply(source, replacements)?;
    parse(&rendered)?;
    Ok(with_trailing_newline(rendered))
}

fn parse(source: &str) -> Result<ImDocument<String>, ContractEditError> {
    ImDocument::parse(source.to_owned())
        .map_err(|error| ContractEditError(format!("parse contract source: {error}")))
}

fn migrate_schema(
    document: &ImDocument<String>,
    replacements: &mut Vec<Replacement>,
) -> Result<(), ContractEditError> {
    let schema = document
        .get("schema")
        .and_then(Item::as_value)
        .ok_or_else(|| ContractEditError("contract entry must declare schema".into()))?;
    let value = schema
        .as_integer()
        .ok_or_else(|| ContractEditError("contract schema must be an integer".into()))?;
    if value != 2 {
        replace_value(schema, "2", replacements)?;
    }
    Ok(())
}

fn migrate_imports(
    document: &ImDocument<String>,
    exact_imports: &[String],
    replacements: &mut Vec<Replacement>,
) -> Result<(), ContractEditError> {
    let Some(imports) = document.get("imports") else {
        return Ok(());
    };
    let imports = imports
        .as_array()
        .ok_or_else(|| ContractEditError("contract imports must contain an array".into()))?;
    for import in imports {
        let pattern = import
            .as_str()
            .ok_or_else(|| ContractEditError("contract imports must contain strings".into()))?;
        if !has_wildcard(pattern) {
            continue;
        }
        let mut matches = exact_imports
            .iter()
            .filter(|path| crate::glob_matches(pattern, path))
            .cloned()
            .collect::<Vec<_>>();
        matches.sort();
        matches.dedup();
        if matches.is_empty() {
            return Err(ContractEditError(format!(
                "contract import {pattern:?} matched no exact paths"
            )));
        }
        let text = matches
            .iter()
            .map(|path| Value::from(path.as_str()).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        replace_value(import, &text, replacements)?;
    }
    Ok(())
}

fn replace_value(
    value: &Value,
    text: &str,
    replacements: &mut Vec<Replacement>,
) -> Result<(), ContractEditError> {
    let span = value
        .span()
        .ok_or_else(|| ContractEditError("contract value has no source span".into()))?;
    replacements.push(Replacement {
        span,
        text: text.into(),
    });
    Ok(())
}

fn apply(source: &str, mut replacements: Vec<Replacement>) -> Result<String, ContractEditError> {
    replacements.sort_by_key(|replacement| replacement.span.start);
    if replacements
        .windows(2)
        .any(|pair| pair[0].span.end > pair[1].span.start)
    {
        return Err(ContractEditError(
            "contract migration produced overlapping edits".into(),
        ));
    }
    let mut rendered = source.to_owned();
    for replacement in replacements.into_iter().rev() {
        rendered.replace_range(replacement.span, &replacement.text);
    }
    Ok(rendered)
}

fn has_wildcard(value: &str) -> bool {
    value.bytes().any(|byte| matches!(byte, b'*' | b'?'))
}

fn with_trailing_newline(mut source: String) -> String {
    if !source.ends_with('\n') {
        source.push('\n');
    }
    source
}

#[cfg(test)]
#[path = "contract_edit_test.rs"]
mod contract_edit_test;
