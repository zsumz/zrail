//! Lock updates keep immutable-base and grant acceptance authority explicit.

use std::ffi::OsString;

use super::{Command, CommonOptions, as_string, os_value, parse_format, value};
use crate::app::error::CliError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UpdateOptions {
    pub(crate) common: CommonOptions,
    pub(crate) base: OsString,
    pub(crate) accept_grants: bool,
    pub(crate) accept_migration: Option<String>,
}

pub(super) fn parse(arguments: &[OsString]) -> Result<Command, CliError> {
    let mut common = CommonOptions::default();
    let mut base = OsString::from("HEAD");
    let mut base_set = false;
    let mut accept_grants = false;
    let mut accept_migration = None;
    let mut index = 0;
    while index < arguments.len() {
        let flag = as_string(&arguments[index])?;
        match flag.as_str() {
            "--root" => common.root = value(arguments, &mut index, "--root")?,
            "--config" => common.config = value(arguments, &mut index, "--config")?,
            "--lock" => common.lock = value(arguments, &mut index, "--lock")?,
            "--format" => {
                common.format = parse_format(&value(arguments, &mut index, "--format")?)?;
            }
            "--base" if !base_set => {
                base = os_value(arguments, &mut index, "--base")?;
                base_set = true;
            }
            "--base" => return Err(CliError::new("--base may be specified only once")),
            "--accept-grants" if !accept_grants => accept_grants = true,
            "--accept-grants" => {
                return Err(CliError::new("--accept-grants may be specified only once"));
            }
            "--accept-migration" => {
                let value = os_value(arguments, &mut index, "--accept-migration")?;
                let value = as_string(&value)?;
                if accept_migration.replace(value).is_some() {
                    return Err(CliError::new(
                        "--accept-migration may be specified only once",
                    ));
                }
            }
            _ => return Err(CliError::new(format!("unknown option {flag:?}"))),
        }
        index += 1;
    }
    Ok(Command::Update(UpdateOptions {
        common,
        base,
        accept_grants,
        accept_migration,
    }))
}
