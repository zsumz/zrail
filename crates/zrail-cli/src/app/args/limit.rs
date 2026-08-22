//! Diagnostic payload-limit parsing.

use std::path::Path;

use zrail_core::DiagnosticLimit;

use crate::app::error::CliError;

pub(super) fn parse(value: &Path) -> Result<DiagnosticLimit, CliError> {
    match value.to_string_lossy().as_ref() {
        "all" => Ok(DiagnosticLimit::All),
        value => value
            .parse::<usize>()
            .map(DiagnosticLimit::Bounded)
            .map_err(|_| {
                CliError::new(format!("unsupported diagnostic limit {value:?}"))
                    .with_help("use a non-negative integer or `all`")
            }),
    }
}

#[cfg(test)]
#[path = "limit_test.rs"]
mod limit_test;
