//! Immutable-base lock migration report options.

use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

use super::{Command, as_string, os_value, set_once, value};
use crate::app::error::CliError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MigrateLockOptions {
    pub(crate) root: PathBuf,
    pub(crate) config: PathBuf,
    pub(crate) lock: PathBuf,
    pub(crate) base: OsString,
    pub(crate) output: PathBuf,
}

pub(super) fn parse(arguments: &[OsString]) -> Result<Command, CliError> {
    let mut root = PathBuf::from(".");
    let mut config = PathBuf::from("zrail.toml");
    let mut lock = PathBuf::from("zrail.lock");
    let mut base = None;
    let mut output = None;
    let mut index = 0;
    while index < arguments.len() {
        let flag = as_string(&arguments[index])?;
        match flag.as_str() {
            "--root" => root = value(arguments, &mut index, "--root")?,
            "--config" => config = value(arguments, &mut index, "--config")?,
            "--lock" => lock = value(arguments, &mut index, "--lock")?,
            "--base" => set_once(
                &mut base,
                os_value(arguments, &mut index, "--base")?,
                "migration base",
            )?,
            "--output" => set_once(
                &mut output,
                value(arguments, &mut index, "--output")?,
                "migration output",
            )?,
            _ => return Err(CliError::new(format!("unknown option {flag:?}"))),
        }
        index += 1;
    }
    Ok(Command::MigrateLock(MigrateLockOptions {
        root,
        config,
        lock,
        base: base.unwrap_or_else(|| OsStr::new("HEAD").to_os_string()),
        output: output.ok_or_else(|| CliError::new("migrate-lock requires --output PATH"))?,
    }))
}
