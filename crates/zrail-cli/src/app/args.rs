//! Small explicit CLI parser; every accepted flag is visible here.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use zrail_core::DiagnosticLimit;

use super::{error::CliError, output::OutputFormat};

mod baseline;
mod common;
mod coverage;
mod diff;
mod fmt;
mod init;
mod limit;
mod migrate_config;
mod migrate_lock;
mod mirrors;
mod review;
mod update;

use limit::parse as parse_limit;

pub(crate) use baseline::BaselineOptions;
pub(crate) use coverage::CoverageOptions;
pub(crate) use fmt::FmtOptions;
pub(crate) use init::{InitOptions, InitPreset};
pub(crate) use migrate_config::MigrateConfigOptions;
pub(crate) use migrate_lock::MigrateLockOptions;
pub(crate) use mirrors::{MirrorsAction, MirrorsOptions};
pub(crate) use update::UpdateOptions;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Check(CommonOptions),
    Coverage(CoverageOptions),
    Doctor(CommonOptions),
    Baseline(BaselineOptions),
    Update(UpdateOptions),
    Explain {
        common: CommonOptions,
        path: PathBuf,
    },
    Diff(DiffOptions),
    Review(ReviewOptions),
    Init(InitOptions),
    MigrateConfig(MigrateConfigOptions),
    MigrateLock(MigrateLockOptions),
    Mirrors(MirrorsOptions),
    Fmt(FmtOptions),
    Help,
    Version,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommonOptions {
    pub(crate) root: PathBuf,
    pub(crate) config: PathBuf,
    pub(crate) lock: PathBuf,
    pub(crate) format: OutputFormat,
    pub(crate) limit: DiagnosticLimit,
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
pub(crate) struct ReviewOptions {
    pub(crate) common: CommonOptions,
    pub(crate) authority_root: PathBuf,
    pub(crate) base: OsString,
    pub(crate) allow_grants: bool,
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
        "check" => Ok(Command::Check(common::parse(&remaining, true)?)),
        "coverage" => coverage::parse(&remaining),
        "doctor" => Ok(Command::Doctor(common::parse(&remaining, false)?)),
        "baseline" => baseline::parse(&remaining),
        "update" => update::parse(&remaining),
        "explain" | "guide" => parse_explain(&remaining),
        "diff" => diff::parse(&remaining),
        "review" => review::parse(&remaining),
        "init" => init::parse(&remaining),
        "migrate-config" => migrate_config::parse(&remaining),
        "migrate-lock" => migrate_lock::parse(&remaining),
        "mirrors" => mirrors::parse(&remaining),
        "fmt" => fmt::parse(&remaining),
        "help" | "--help" | "-h" => Ok(Command::Help),
        "version" | "--version" | "-V" => Ok(Command::Version),
        other => Err(CliError::new(format!("unknown command {other:?}"))
            .with_help("run `zrail help` for the supported command map")),
    }
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
