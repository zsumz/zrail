//! Exclusion matching distinguishes result filtering from proven subtree pruning.

use zrail_core::{Contract, glob_matches};

pub(super) fn excluded(contract: &Contract, relative: &str) -> bool {
    excluded_by(&contract.repository.exclude, relative)
}

pub(super) fn excluded_by(exclusions: &[String], relative: &str) -> bool {
    exclusions.iter().any(|pattern| {
        glob_matches(pattern, relative) || relative.starts_with(&format!("{pattern}/"))
    })
}

pub(super) fn excluded_subtree(exclusions: &[String], directory: &str) -> bool {
    exclusions.iter().any(|pattern| {
        if !pattern.bytes().any(|byte| matches!(byte, b'*' | b'?')) {
            return directory == pattern || directory.starts_with(&format!("{pattern}/"));
        }
        let prefix = pattern.trim_end_matches("/**");
        prefix != pattern && glob_matches(prefix, directory)
    })
}
