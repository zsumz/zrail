//! Protected review has an explicit trusted repository and untrusted proposal.

use std::{ffi::OsString, path::PathBuf};

use super::{Command, CommonOptions, ReviewOptions, as_string, os_value, parse_format, value};
use crate::app::error::CliError;

pub(super) fn parse(arguments: &[OsString]) -> Result<Command, CliError> {
    let mut common = CommonOptions::default();
    let mut authority_root = PathBuf::from(".");
    let mut base = OsString::from("HEAD");
    let mut authority_set = false;
    let mut base_set = false;
    let mut allow_grants = false;
    let mut index = 0;
    while index < arguments.len() {
        let flag = as_string(&arguments[index])?;
        match flag.as_str() {
            "--root" => common.root = value(arguments, &mut index, "--root")?,
            "--authority-root" if !authority_set => {
                authority_root = value(arguments, &mut index, "--authority-root")?;
                authority_set = true;
            }
            "--authority-root" => {
                return Err(CliError::new("--authority-root may be specified only once"));
            }
            "--base" if !base_set => {
                base = os_value(arguments, &mut index, "--base")?;
                base_set = true;
            }
            "--base" => return Err(CliError::new("--base may be specified only once")),
            "--config" => common.config = value(arguments, &mut index, "--config")?,
            "--lock" => common.lock = value(arguments, &mut index, "--lock")?,
            "--format" => {
                common.format = parse_format(&value(arguments, &mut index, "--format")?)?;
            }
            "--allow-grants" if !allow_grants => allow_grants = true,
            "--allow-grants" => {
                return Err(CliError::new("--allow-grants may be specified only once"));
            }
            _ => return Err(CliError::new(format!("unknown option {flag:?}"))),
        }
        index += 1;
    }
    Ok(Command::Review(ReviewOptions {
        common,
        authority_root,
        base,
        allow_grants,
    }))
}
