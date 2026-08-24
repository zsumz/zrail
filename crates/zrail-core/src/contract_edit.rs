//! Canonical formatting and schema-key migration for one contract source.

use std::{error::Error, fmt};

use toml::{Table, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
/// A contract source could not be parsed, transformed, or rendered.
pub struct ContractEditError(String);

impl fmt::Display for ContractEditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for ContractEditError {}

/// Canonically formats one TOML contract source with a trailing newline.
pub fn format_contract_source(source: &str) -> Result<String, ContractEditError> {
    render(&parse(source)?)
}

/// Migrates one contract source to schema-2 keys and canonical formatting.
///
/// `entry` identifies the root source that owns the schema declaration.
/// `exact_imports` replaces an existing imports array after the caller resolves
/// legacy patterns against the complete, repository-bounded contract bundle.
pub fn migrate_contract_source(
    source: &str,
    entry: bool,
    exact_imports: &[String],
) -> Result<String, ContractEditError> {
    let mut value = parse(source)?;
    let table = value
        .as_table_mut()
        .ok_or_else(|| ContractEditError("contract source root must be a TOML table".into()))?;
    if entry {
        table.insert("schema".into(), Value::Integer(2));
    }
    replace_imports(table, exact_imports)?;
    migrate_macro_keys(table)?;
    render(&value)
}

fn parse(source: &str) -> Result<Value, ContractEditError> {
    source
        .parse::<Value>()
        .map_err(|error| ContractEditError(format!("parse contract source: {error}")))
}

fn render(value: &Value) -> Result<String, ContractEditError> {
    let mut text = toml::to_string_pretty(&value)
        .map_err(|error| ContractEditError(format!("render contract source: {error}")))?;
    if !text.ends_with('\n') {
        text.push('\n');
    }
    Ok(text)
}

fn replace_imports(table: &mut Table, exact_imports: &[String]) -> Result<(), ContractEditError> {
    let Some(imports) = table.get_mut("imports") else {
        return Ok(());
    };
    if !imports.is_array() {
        return Err(ContractEditError(
            "contract imports must contain an array".into(),
        ));
    }
    let mut exact = exact_imports.to_vec();
    exact.sort();
    exact.dedup();
    *imports = Value::Array(exact.into_iter().map(Value::String).collect());
    Ok(())
}

fn migrate_macro_keys(table: &mut Table) -> Result<(), ContractEditError> {
    let Some(rust) = table
        .get_mut("source")
        .and_then(Value::as_table_mut)
        .and_then(|source| source.get_mut("rust"))
        .and_then(Value::as_table_mut)
    else {
        return Ok(());
    };
    if let Some(allowances) = rust
        .get_mut("macros")
        .and_then(Value::as_table_mut)
        .and_then(|macros| macros.get_mut("allow"))
        .and_then(Value::as_array_mut)
    {
        for allowance in allowances.iter_mut().filter_map(Value::as_table_mut) {
            rename(allowance, "binding", "resolution")?;
            rename(allowance, "bindings", "namespace_effect")?;
        }
    }
    if let Some(allowances) = rust.get_mut("item_macros").and_then(Value::as_array_mut) {
        for allowance in allowances.iter_mut().filter_map(Value::as_table_mut) {
            rename(allowance, "binding", "resolution")?;
        }
    }
    Ok(())
}

fn rename(table: &mut Table, old: &str, new: &str) -> Result<(), ContractEditError> {
    if table.contains_key(old) && table.contains_key(new) {
        return Err(ContractEditError(format!(
            "contract may not combine legacy {old:?} with schema-2 {new:?}"
        )));
    }
    if let Some(value) = table.remove(old) {
        table.insert(new.into(), value);
    }
    Ok(())
}

#[cfg(test)]
#[path = "contract_edit_test.rs"]
mod contract_edit_test;
