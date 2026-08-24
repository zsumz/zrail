//! Exclusion inputs become one canonical selection before Cargo discovery.

use std::path::Path;

use zrail_core::{read_text_with_limit, repository_file};
use zrail_rust::RepositorySelection;

use crate::app::{args::InitOptions, error::CliError};

const MAX_EXCLUSION_FILE_BYTES: usize = 1024 * 1024;

pub(super) fn load(root: &Path, options: &InitOptions) -> Result<RepositorySelection, CliError> {
    let mut exclusions = options.exclusions.clone();
    for relative in &options.exclusion_files {
        let path = repository_file(root, relative).map_err(CliError::new)?;
        let source =
            read_text_with_limit(&path, MAX_EXCLUSION_FILE_BYTES).map_err(CliError::new)?;
        exclusions.extend(lines(&source));
    }
    RepositorySelection::new(exclusions).map_err(|error| CliError::new(error.to_string()))
}

fn lines(source: &str) -> impl Iterator<Item = String> + '_ {
    source.lines().filter_map(|line| {
        let line = line.trim();
        (!line.is_empty() && !line.starts_with('#')).then(|| line.to_owned())
    })
}

#[cfg(test)]
#[path = "init_selection_test.rs"]
mod init_selection_test;
