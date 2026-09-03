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
    pub(crate) target: Option<OsString>,
    pub(crate) output: Option<PathBuf>,
    pub(crate) discover_base: bool,
}

pub(super) fn parse(arguments: &[OsString]) -> Result<Command, CliError> {
    let mut root = PathBuf::from(".");
    let mut config = PathBuf::from("zrail.toml");
    let mut lock = PathBuf::from("zrail.lock");
    let mut base = None;
    let mut target = None;
    let mut output = None;
    let mut discover_base = false;
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
            "--target" => set_once(
                &mut target,
                os_value(arguments, &mut index, "--target")?,
                "migration target",
            )?,
            "--output" => set_once(
                &mut output,
                value(arguments, &mut index, "--output")?,
                "migration output",
            )?,
            "--discover-base" if !discover_base => discover_base = true,
            "--discover-base" => {
                return Err(CliError::new(
                    "migration base discovery may be specified only once",
                ));
            }
            _ => return Err(CliError::new(format!("unknown option {flag:?}"))),
        }
        index += 1;
    }
    if discover_base && (base.is_some() || target.is_some() || output.is_some()) {
        return Err(CliError::new(
            "--discover-base cannot be combined with --base, --target, or --output",
        ));
    }
    if !discover_base && output.is_none() {
        return Err(CliError::new("migrate-lock requires --output PATH"));
    }
    Ok(Command::MigrateLock(MigrateLockOptions {
        root,
        config,
        lock,
        base: base.unwrap_or_else(|| OsStr::new("HEAD").to_os_string()),
        target,
        output,
        discover_base,
    }))
}

#[cfg(test)]
#[path = "migrate_lock_test.rs"]
mod migrate_lock_test;
