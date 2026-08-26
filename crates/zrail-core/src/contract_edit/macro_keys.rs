//! Legacy macro-key migration without reserializing surrounding TOML.

use toml_edit::{ImDocument, InlineTable, Item, Key, Table, Value};

use super::{ContractEditError, Replacement};

pub(super) fn migrate_macro_keys(
    document: &ImDocument<String>,
    source: &str,
    replacements: &mut Vec<Replacement>,
) -> Result<(), ContractEditError> {
    let Some(rust) = document
        .get("source")
        .and_then(Item::as_table)
        .and_then(|source| source.get("rust"))
        .and_then(Item::as_table)
    else {
        return Ok(());
    };
    if let Some(allowances) = rust
        .get("macros")
        .and_then(Item::as_table)
        .and_then(|macros| macros.get("allow"))
    {
        if let Some(tables) = allowances.as_array_of_tables() {
            for allowance in tables {
                rename_table(allowance, source, "binding", "resolution", replacements)?;
                rename_table(
                    allowance,
                    source,
                    "bindings",
                    "namespace_effect",
                    replacements,
                )?;
            }
        } else if let Some(values) = allowances.as_array() {
            for allowance in values.iter().filter_map(Value::as_inline_table) {
                rename_inline(allowance, source, "binding", "resolution", replacements)?;
                rename_inline(
                    allowance,
                    source,
                    "bindings",
                    "namespace_effect",
                    replacements,
                )?;
            }
        }
    }
    if let Some(allowances) = rust.get("item_macros") {
        if let Some(tables) = allowances.as_array_of_tables() {
            for allowance in tables {
                rename_table(allowance, source, "binding", "resolution", replacements)?;
            }
        } else if let Some(values) = allowances.as_array() {
            for allowance in values.iter().filter_map(Value::as_inline_table) {
                rename_inline(allowance, source, "binding", "resolution", replacements)?;
            }
        }
    }
    Ok(())
}

fn rename_table(
    table: &Table,
    source: &str,
    old: &str,
    new: &str,
    replacements: &mut Vec<Replacement>,
) -> Result<(), ContractEditError> {
    rename_key(
        (
            table.contains_key(old),
            table.contains_key(new),
            table.get_key_value(old).map(|(key, _)| key),
        ),
        source,
        (old, new),
        replacements,
    )
}

fn rename_inline(
    table: &InlineTable,
    source: &str,
    old: &str,
    new: &str,
    replacements: &mut Vec<Replacement>,
) -> Result<(), ContractEditError> {
    rename_key(
        (
            table.contains_key(old),
            table.contains_key(new),
            table.get_key_value(old).map(|(key, _)| key),
        ),
        source,
        (old, new),
        replacements,
    )
}

fn rename_key(
    state: (bool, bool, Option<&Key>),
    source: &str,
    names: (&str, &str),
    replacements: &mut Vec<Replacement>,
) -> Result<(), ContractEditError> {
    let (has_old, has_new, key) = state;
    let (old, new) = names;
    if has_old && has_new {
        return Err(ContractEditError(format!(
            "contract may not combine legacy {old:?} with schema-2 {new:?}"
        )));
    }
    let Some(key) = key else {
        return Ok(());
    };
    let span = key
        .span()
        .ok_or_else(|| ContractEditError(format!("legacy key {old:?} has no source span")))?;
    let text = key_spelling(&source[span.clone()], new);
    replacements.push(Replacement { span, text });
    Ok(())
}

fn key_spelling(original: &str, replacement: &str) -> String {
    match original.as_bytes().first() {
        Some(b'\'') => format!("'{replacement}'"),
        Some(b'\"') => format!("\"{replacement}\""),
        _ => replacement.into(),
    }
}
