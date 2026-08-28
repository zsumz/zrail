//! Literal include targets can extend the bounded Rust source inventory.

use zrail_core::{Contract, read_text_with_limit};

use super::{
    MAX_RUST_FILES, MAX_RUST_SOURCE_BYTES, RepositoryEntryKind, RepositoryInventory,
    RepositoryInventoryError, RustSourceFile, add_source_bytes, classify_path, excluded,
    under_roots,
};

pub(crate) fn load_referenced_source(
    inventory: &mut RepositoryInventory,
    contract: &Contract,
    relative: &str,
    source_bytes: &mut usize,
) -> Result<bool, RepositoryInventoryError> {
    if excluded(contract, relative) || !under_roots(contract, relative) {
        return Ok(false);
    }
    let Some(entry) = inventory
        .entries
        .iter()
        .find(|entry| entry.relative == relative)
    else {
        return Ok(false);
    };
    if entry.kind != RepositoryEntryKind::File {
        return Ok(false);
    }
    if inventory.rust_files.len() == MAX_RUST_FILES {
        return Err(RepositoryInventoryError(format!(
            "repository exceeds the {MAX_RUST_FILES}-Rust-file safety limit"
        )));
    }
    let source = read_text_with_limit(&entry.absolute, MAX_RUST_SOURCE_BYTES)
        .map_err(RepositoryInventoryError)?;
    *source_bytes = add_source_bytes(*source_bytes, source.len())?;
    inventory.rust_files.push(RustSourceFile {
        relative: relative.into(),
        class: classify_path(relative, &contract.source.rust.generated),
        lines: source.lines().count(),
        source,
    });
    inventory
        .rust_files
        .sort_by(|left, right| left.relative.cmp(&right.relative));
    Ok(true)
}
