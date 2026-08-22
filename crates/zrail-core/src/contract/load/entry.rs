//! Proposed entry bytes retain the real contract identity without filesystem mutation.

use std::path::Path;

use crate::input::{MAX_INPUT_BYTES, read_text};

use super::{ContractBundle, ContractError};

/// Loads a contract while substituting exact UTF-8 bytes for its entry file.
///
/// The on-disk entry must still be a regular non-symlink file inside `root`.
/// Imports retain their normal containment and safety limits, and the resulting
/// digest uses `config` rather than a temporary path.
pub fn load_contract_with_entry(
    root: &Path,
    config: &Path,
    entry: &str,
) -> Result<ContractBundle, ContractError> {
    super::load_contract_entry(root, config, Some(entry))
}

pub(super) fn validate(path: &Path, entry: &str) -> Result<(), ContractError> {
    read_text(path).map_err(ContractError::one)?;
    if entry.len() > MAX_INPUT_BYTES {
        return Err(ContractError::one(format!(
            "contract entry exceeds the {MAX_INPUT_BYTES}-byte safety limit: {}",
            path.display()
        )));
    }
    Ok(())
}
