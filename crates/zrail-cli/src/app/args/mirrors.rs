//! Non-executing test-mirror plan and receipt verification options.

use std::{ffi::OsString, path::PathBuf};

use super::{Command, as_string, parse_format, set_once, value};
use crate::app::{error::CliError, output::OutputFormat};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MirrorsOptions {
    pub(crate) root: PathBuf,
    pub(crate) config: PathBuf,
    pub(crate) format: OutputFormat,
    pub(crate) action: MirrorsAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MirrorsAction {
    Plan,
    Verify { plan: PathBuf },
    Receipts { plan: PathBuf, results: PathBuf },
}

pub(super) fn parse(arguments: &[OsString]) -> Result<Command, CliError> {
    let (action, remaining) = arguments
        .split_first()
        .ok_or_else(|| CliError::new("mirrors requires `plan`, `receipts`, or `verify`"))?;
    let action = as_string(action)?;
    let mut root = PathBuf::from(".");
    let mut config = PathBuf::from("zrail.toml");
    let mut format = OutputFormat::Human;
    let mut plan = None;
    let mut results = None;
    let mut index = 0;
    while index < remaining.len() {
        let flag = as_string(&remaining[index])?;
        match flag.as_str() {
            "--root" => root = value(remaining, &mut index, "--root")?,
            "--config" => config = value(remaining, &mut index, "--config")?,
            "--format" => {
                format = parse_format(&value(remaining, &mut index, "--format")?)?;
            }
            "--plan" => set_once(
                &mut plan,
                value(remaining, &mut index, "--plan")?,
                "mirror plan",
            )?,
            "--results" => set_once(
                &mut results,
                value(remaining, &mut index, "--results")?,
                "mirror results",
            )?,
            _ => return Err(CliError::new(format!("unknown option {flag:?}"))),
        }
        index += 1;
    }
    let action = match action.as_str() {
        "plan" if plan.is_none() && results.is_none() => MirrorsAction::Plan,
        "plan" => {
            return Err(CliError::new(
                "mirrors plan does not accept --plan or --results",
            ));
        }
        "verify" if results.is_none() => MirrorsAction::Verify {
            plan: plan.ok_or_else(|| CliError::new("mirrors verify requires --plan PATH"))?,
        },
        "verify" => return Err(CliError::new("mirrors verify does not accept --results")),
        "receipts" => MirrorsAction::Receipts {
            plan: plan.ok_or_else(|| CliError::new("mirrors receipts requires --plan PATH"))?,
            results: results
                .ok_or_else(|| CliError::new("mirrors receipts requires --results PATH"))?,
        },
        other => {
            return Err(CliError::new(format!(
                "unknown mirrors action {other:?}; expected `plan`, `receipts`, or `verify`"
            )));
        }
    };
    Ok(Command::Mirrors(MirrorsOptions {
        root,
        config,
        format,
        action,
    }))
}

#[cfg(test)]
#[path = "mirrors_test.rs"]
mod mirrors_test;
