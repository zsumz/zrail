//! Small explicit CLI parser; every accepted flag is visible here.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use super::{error::CliError, output::OutputFormat};

mod diff;
mod init;

pub(crate) use init::{InitOptions, InitPreset};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Check(CommonOptions),
    Doctor(CommonOptions),
    Update(UpdateOptions),
    Explain {
        common: CommonOptions,
        path: PathBuf,
    },
    Diff(DiffOptions),
    Init(InitOptions),
    Help,
    Version,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommonOptions {
    pub(crate) root: PathBuf,
    pub(crate) config: PathBuf,
    pub(crate) lock: PathBuf,
    pub(crate) format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiffOptions {
    pub(crate) mode: DiffMode,
    pub(crate) config: PathBuf,
    pub(crate) lock: PathBuf,
    pub(crate) format: OutputFormat,
    pub(crate) deny_grants: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DiffMode {
    Base { root: PathBuf, revision: OsString },
    Explicit { before: PathBuf, after: PathBuf },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UpdateOptions {
    pub(crate) common: CommonOptions,
    pub(crate) accept_grants: bool,
}

impl Default for CommonOptions {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            config: PathBuf::from("zrail.toml"),
            lock: PathBuf::from("zrail.lock"),
            format: OutputFormat::Human,
        }
    }
}

pub(crate) fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Command, CliError> {
    let mut arguments = arguments.into_iter();
    let _program = arguments.next();
    let Some(command) = arguments.next() else {
        return Ok(Command::Help);
    };
    let command = command
        .into_string()
        .map_err(|_| CliError::new("command is not valid UTF-8"))?;
    let remaining = arguments.collect::<Vec<_>>();
    match command.as_str() {
        "check" => Ok(Command::Check(parse_common(&remaining)?)),
        "doctor" => Ok(Command::Doctor(parse_common(&remaining)?)),
        "update" => parse_update(&remaining),
        "explain" | "guide" => parse_explain(&remaining),
        "diff" => diff::parse(&remaining),
        "init" => init::parse(&remaining),
        "help" | "--help" | "-h" => Ok(Command::Help),
        "version" | "--version" | "-V" => Ok(Command::Version),
        other => Err(CliError::new(format!("unknown command {other:?}"))
            .with_help("run `zrail help` for the supported command map")),
    }
}

fn parse_common(arguments: &[OsString]) -> Result<CommonOptions, CliError> {
    parse_common_options(arguments, false).map(|(options, _)| options)
}

fn parse_update(arguments: &[OsString]) -> Result<Command, CliError> {
    let (common, accept_grants) = parse_common_options(arguments, true)?;
    Ok(Command::Update(UpdateOptions {
        common,
        accept_grants,
    }))
}

fn parse_common_options(
    arguments: &[OsString],
    allow_accept_grants: bool,
) -> Result<(CommonOptions, bool), CliError> {
    let mut options = CommonOptions::default();
    let mut accept_grants = false;
    let mut index = 0;
    while index < arguments.len() {
        let flag = as_string(&arguments[index])?;
        match flag.as_str() {
            "--root" => options.root = value(arguments, &mut index, "--root")?,
            "--config" => options.config = value(arguments, &mut index, "--config")?,
            "--lock" => options.lock = value(arguments, &mut index, "--lock")?,
            "--format" => {
                let value = value(arguments, &mut index, "--format")?;
                options.format = parse_format(&value)?;
            }
            "--accept-grants" if allow_accept_grants && !accept_grants => accept_grants = true,
            "--accept-grants" if allow_accept_grants => {
                return Err(CliError::new("--accept-grants may be specified only once"));
            }
            _ => return Err(CliError::new(format!("unknown option {flag:?}"))),
        }
        index += 1;
    }
    Ok((options, accept_grants))
}

fn parse_explain(arguments: &[OsString]) -> Result<Command, CliError> {
    let mut common = CommonOptions::default();
    let mut path = None;
    let mut index = 0;
    while index < arguments.len() {
        let argument = as_string(&arguments[index])?;
        match argument.as_str() {
            "--path" => set_once(
                &mut path,
                value(arguments, &mut index, "--path")?,
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
                &mut path,
                PathBuf::from(arguments[index].as_os_str()),
                "explain path",
            )?,
        }
        index += 1;
    }
    Ok(Command::Explain {
        common,
        path: path
            .ok_or_else(|| CliError::new("explain requires --path <repository-relative-path>"))?,
    })
}

pub(super) fn set_once<T>(target: &mut Option<T>, value: T, label: &str) -> Result<(), CliError> {
    if target.replace(value).is_some() {
        Err(CliError::new(format!("{label} may be specified only once")))
    } else {
        Ok(())
    }
}

pub(super) fn value(
    arguments: &[OsString],
    index: &mut usize,
    flag: &str,
) -> Result<PathBuf, CliError> {
    os_value(arguments, index, flag).map(PathBuf::from)
}

pub(super) fn os_value(
    arguments: &[OsString],
    index: &mut usize,
    flag: &str,
) -> Result<OsString, CliError> {
    *index += 1;
    arguments
        .get(*index)
        .cloned()
        .ok_or_else(|| CliError::new(format!("{flag} requires a value")))
}

pub(super) fn parse_format(value: &Path) -> Result<OutputFormat, CliError> {
    match value.to_string_lossy().as_ref() {
        "human" => Ok(OutputFormat::Human),
        "json" => Ok(OutputFormat::Json),
        other => Err(CliError::new(format!(
            "unsupported output format {other:?}"
        ))),
    }
}

pub(super) fn as_string(value: &OsString) -> Result<String, CliError> {
    value
        .to_str()
        .map(str::to_owned)
        .ok_or_else(|| CliError::new("argument is not valid UTF-8"))
}

#[cfg(test)]
#[path = "args_test.rs"]
mod args_test;
