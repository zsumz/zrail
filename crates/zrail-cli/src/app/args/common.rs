//! Shared read-only repository, output, lock, and diagnostic-limit flags.

use std::{ffi::OsString, path::PathBuf};

use crate::app::error::CliError;

use super::{CommonOptions, as_string, limit, parse_format, value};

impl Default for CommonOptions {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            config: PathBuf::from("zrail.toml"),
            lock: PathBuf::from("zrail.lock"),
            format: crate::app::output::OutputFormat::Human,
            limit: zrail_core::DiagnosticLimit::default(),
        }
    }
}

pub(super) fn parse(arguments: &[OsString], allow_limit: bool) -> Result<CommonOptions, CliError> {
    let mut options = CommonOptions::default();
    let mut limit_set = false;
    let mut index = 0;
    while index < arguments.len() {
        let flag = as_string(&arguments[index])?;
        match flag.as_str() {
            "--root" => options.root = value(arguments, &mut index, "--root")?,
            "--config" => options.config = value(arguments, &mut index, "--config")?,
            "--lock" => options.lock = value(arguments, &mut index, "--lock")?,
            "--format" => {
                options.format = parse_format(&value(arguments, &mut index, "--format")?)?;
            }
            "--max-findings" | "--limit" if allow_limit && !limit_set => {
                options.limit = limit::parse(&value(arguments, &mut index, &flag)?)?;
                limit_set = true;
            }
            "--max-findings" | "--limit" if allow_limit => {
                return Err(CliError::new(
                    "--max-findings may be specified only once (deprecated --limit is an alias)",
                ));
            }
            _ => return Err(CliError::new(format!("unknown option {flag:?}"))),
        }
        index += 1;
    }
    Ok(options)
}
