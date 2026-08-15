//! Mutually exclusive parsing for Git-base and explicit-state architecture diffs.

use std::{ffi::OsString, path::PathBuf};

use crate::app::{error::CliError, output::OutputFormat};

use super::{Command, DiffMode, DiffOptions, as_string, os_value, parse_format, set_once, value};

pub(super) fn parse(arguments: &[OsString]) -> Result<Command, CliError> {
    let mut base = None;
    let mut before = None;
    let mut after = None;
    let mut root = PathBuf::from(".");
    let mut root_set = false;
    let mut config = PathBuf::from("zrail.toml");
    let mut lock = PathBuf::from("zrail.lock");
    let mut format = OutputFormat::Human;
    let mut deny_grants = false;
    let mut index = 0;
    while index < arguments.len() {
        let flag = as_string(&arguments[index])?;
        match flag.as_str() {
            "--base" => set_once(
                &mut base,
                os_value(arguments, &mut index, "--base")?,
                "--base",
            )?,
            "--before" => set_once(
                &mut before,
                value(arguments, &mut index, "--before")?,
                "--before",
            )?,
            "--after" => set_once(
                &mut after,
                value(arguments, &mut index, "--after")?,
                "--after",
            )?,
            "--root" if !root_set => {
                root = value(arguments, &mut index, "--root")?;
                root_set = true;
            }
            "--root" => return Err(CliError::new("--root may be specified only once")),
            "--config" => config = value(arguments, &mut index, "--config")?,
            "--lock" => lock = value(arguments, &mut index, "--lock")?,
            "--format" => format = parse_format(&value(arguments, &mut index, "--format")?)?,
            "--deny-grants" if !deny_grants => deny_grants = true,
            "--deny-grants" => {
                return Err(CliError::new("--deny-grants may be specified only once"));
            }
            _ => return Err(CliError::new(format!("unknown option {flag:?}"))),
        }
        index += 1;
    }
    let mode = select_mode(base, before, after, root, root_set)?;
    Ok(Command::Diff(DiffOptions {
        mode,
        config,
        lock,
        format,
        deny_grants,
    }))
}

fn select_mode(
    base: Option<OsString>,
    before: Option<PathBuf>,
    after: Option<PathBuf>,
    root: PathBuf,
    root_set: bool,
) -> Result<DiffMode, CliError> {
    match (base, before, after) {
        (Some(revision), None, None) => Ok(DiffMode::Base { root, revision }),
        (None, Some(before), Some(after)) if !root_set => Ok(DiffMode::Explicit { before, after }),
        (Some(_), _, _) => Err(CliError::new(
            "diff --base cannot be combined with --before or --after",
        )),
        (None, Some(_), Some(_)) => Err(CliError::new("diff --root is only valid with --base")),
        (None, None, None) => Err(CliError::new(
            "diff requires --base <revision> or --before <repository> --after <repository>",
        )),
        (None, _, _) => Err(CliError::new(
            "diff requires both --before <repository> and --after <repository>",
        )),
    }
}
