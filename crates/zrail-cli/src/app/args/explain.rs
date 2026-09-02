//! Concrete and hypothetical path explanation options.

use std::{ffi::OsString, path::PathBuf};

use crate::app::error::CliError;

use super::{Command, CommonOptions, as_string, parse_format, set_once, value};

pub(super) fn parse(arguments: &[OsString]) -> Result<Command, CliError> {
    let mut common = CommonOptions::default();
    let mut target = None;
    let mut index = 0;
    while index < arguments.len() {
        let argument = as_string(&arguments[index])?;
        match argument.as_str() {
            "--path" => set_once(
                &mut target,
                (value(arguments, &mut index, "--path")?, false),
                "explain path",
            )?,
            "--hypothetical-path" => set_once(
                &mut target,
                (value(arguments, &mut index, "--hypothetical-path")?, true),
                "explain path",
            )?,
            "--root" => common.root = value(arguments, &mut index, "--root")?,
            "--config" => common.config = value(arguments, &mut index, "--config")?,
            "--lock" => common.lock = value(arguments, &mut index, "--lock")?,
            "--format" => {
                common.format = parse_format(&value(arguments, &mut index, "--format")?)?;
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::new(format!("unknown option {flag:?}")));
            }
            _ => set_once(
                &mut target,
                (PathBuf::from(arguments[index].as_os_str()), false),
                "explain path",
            )?,
        }
        index += 1;
    }
    let (path, hypothetical) = target
        .ok_or_else(|| CliError::new("explain requires --path <repository-relative-path>"))?;
    Ok(Command::Explain {
        common,
        path,
        hypothetical,
    })
}

#[cfg(test)]
#[path = "explain_test.rs"]
mod explain_test;
