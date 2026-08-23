//! Mutating commands keep lock output disjoint from every contract source.

use std::{fs, path::Path};

use zrail_core::{ContractBundle, repository_file};

use crate::app::error::CliError;

pub(super) fn reject_lock_contract_overlap(
    root: &Path,
    bundle: &ContractBundle,
    lock: &Path,
) -> Result<(), CliError> {
    let destination = repository_file(root, lock).map_err(CliError::new)?;
    let canonical_destination = fs::canonicalize(&destination).ok();
    for source in &bundle.sources {
        let source_path = repository_file(root, Path::new(&source.path)).map_err(CliError::new)?;
        let same_path = destination == source_path;
        let same_existing_file = canonical_destination.as_ref().is_some_and(|candidate| {
            fs::canonicalize(&source_path).is_ok_and(|source| source == *candidate)
        });
        if same_path || same_existing_file {
            return Err(CliError::new(format!(
                "lock destination {} overlaps contract source {}; choose a distinct lock path",
                lock.display(),
                source.path
            )));
        }
    }
    Ok(())
}
