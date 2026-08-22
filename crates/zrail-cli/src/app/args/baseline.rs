//! Baseline adoption keeps rule selection and grant acceptance explicit.

use std::path::PathBuf;

use super::{Command, CommonOptions, as_string, parse_format, set_once, value};
use crate::app::error::CliError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BaselineOptions {
    pub(crate) common: CommonOptions,
    pub(crate) dry_run: bool,
    pub(crate) accept_grants: bool,
    pub(crate) rule: Option<String>,
}

pub(super) fn parse(arguments: &[std::ffi::OsString]) -> Result<Command, CliError> {
    let mut common = CommonOptions::default();
    let mut dry_run = false;
    let mut accept_grants = false;
    let mut rule: Option<PathBuf> = None;
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
            "--rule" => set_once(&mut rule, value(arguments, &mut index, "--rule")?, "--rule")?,
            "--dry-run" if !dry_run => dry_run = true,
            "--dry-run" => return Err(CliError::new("--dry-run may be specified only once")),
            "--accept-grants" if !accept_grants => accept_grants = true,
            "--accept-grants" => {
                return Err(CliError::new("--accept-grants may be specified only once"));
            }
            _ => return Err(CliError::new(format!("unknown option {flag:?}"))),
        }
        index += 1;
    }
    let rule = rule
        .map(|rule| {
            rule.into_os_string()
                .into_string()
                .map_err(|_| CliError::new("--rule is not valid UTF-8"))
        })
        .transpose()?;
    Ok(Command::Baseline(BaselineOptions {
        common,
        dry_run,
        accept_grants,
        rule,
    }))
}
