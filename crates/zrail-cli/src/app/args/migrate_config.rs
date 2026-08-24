//! Explicit schema migration never hides its repository mutation.

use std::{ffi::OsString, path::PathBuf};

use super::{Command, as_string, value};
use crate::app::error::CliError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MigrateConfigOptions {
    pub(crate) root: PathBuf,
    pub(crate) config: PathBuf,
    pub(crate) write: bool,
}

pub(super) fn parse(arguments: &[OsString]) -> Result<Command, CliError> {
    let mut options = MigrateConfigOptions {
        root: PathBuf::from("."),
        config: PathBuf::from("zrail.toml"),
        write: false,
    };
    let mut index = 0;
    while index < arguments.len() {
        let flag = as_string(&arguments[index])?;
        match flag.as_str() {
            "--root" => options.root = value(arguments, &mut index, "--root")?,
            "--config" => options.config = value(arguments, &mut index, "--config")?,
            "--write" if !options.write => options.write = true,
            "--write" => return Err(CliError::new("--write may be specified only once")),
            _ => return Err(CliError::new(format!("unknown option {flag:?}"))),
        }
        index += 1;
    }
    Ok(Command::MigrateConfig(options))
}
