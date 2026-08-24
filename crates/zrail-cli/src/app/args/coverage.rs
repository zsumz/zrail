//! Read-only governed-surface coverage options.

use std::{ffi::OsString, path::PathBuf};

use super::{Command, as_string, parse_format, value};
use crate::app::{error::CliError, output::OutputFormat};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CoverageOptions {
    pub(crate) root: PathBuf,
    pub(crate) config: PathBuf,
    pub(crate) format: OutputFormat,
}

pub(super) fn parse(arguments: &[OsString]) -> Result<Command, CliError> {
    let mut options = CoverageOptions {
        root: PathBuf::from("."),
        config: PathBuf::from("zrail.toml"),
        format: OutputFormat::Human,
    };
    let mut index = 0;
    while index < arguments.len() {
        let flag = as_string(&arguments[index])?;
        match flag.as_str() {
            "--root" => options.root = value(arguments, &mut index, "--root")?,
            "--config" => options.config = value(arguments, &mut index, "--config")?,
            "--format" => {
                options.format = parse_format(&value(arguments, &mut index, "--format")?)?;
            }
            _ => return Err(CliError::new(format!("unknown option {flag:?}"))),
        }
        index += 1;
    }
    Ok(Command::Coverage(options))
}

#[cfg(test)]
#[path = "coverage_test.rs"]
mod coverage_test;
