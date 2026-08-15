//! Initialization accepts one repository path and one explicit onboarding mode.

use std::{ffi::OsString, path::PathBuf};

use crate::app::error::CliError;

use super::{Command, set_once};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InitOptions {
    pub(crate) root: PathBuf,
    pub(crate) mode: InitMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InitMode {
    Strict,
    Baseline,
}

pub(super) fn parse(arguments: &[OsString]) -> Result<Command, CliError> {
    let mut root = None;
    let mut mode = None;
    for argument in arguments {
        match argument.to_str() {
            Some("--strict") => set_once(&mut mode, InitMode::Strict, "init mode")?,
            Some("--baseline") => set_once(&mut mode, InitMode::Baseline, "init mode")?,
            Some(flag) if flag.starts_with('-') => {
                return Err(CliError::new(format!("unknown option {flag:?}")));
            }
            _ => set_once(
                &mut root,
                PathBuf::from(argument.as_os_str()),
                "init repository path",
            )?,
        }
    }
    Ok(Command::Init(InitOptions {
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        mode: mode.unwrap_or(InitMode::Strict),
    }))
}
