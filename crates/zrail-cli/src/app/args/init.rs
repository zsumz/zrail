//! Initialization accepts one repository path, one preset, and optional debt adoption.

use std::{ffi::OsString, path::PathBuf};

use crate::app::error::CliError;

use super::{Command, as_string, os_value, set_once};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InitOptions {
    pub(crate) root: PathBuf,
    pub(crate) preset: InitPreset,
    pub(crate) baseline: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InitPreset {
    Zsumz,
    Rust,
}

impl InitPreset {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Zsumz => "zsumz",
            Self::Rust => "rust",
        }
    }
}

pub(super) fn parse(arguments: &[OsString]) -> Result<Command, CliError> {
    let mut root = None;
    let mut preset = None;
    let mut baseline = None;
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        match argument.to_str() {
            Some("--preset") => {
                let value = as_string(&os_value(arguments, &mut index, "--preset")?)?;
                set_once(&mut preset, parse_preset(&value)?, "init preset")?;
            }
            Some("--baseline") => set_once(&mut baseline, (), "--baseline")?,
            Some(flag) if flag.starts_with('-') => {
                return Err(CliError::new(format!("unknown option {flag:?}")));
            }
            _ => set_once(
                &mut root,
                PathBuf::from(argument.as_os_str()),
                "init repository path",
            )?,
        }
        index += 1;
    }
    Ok(Command::Init(InitOptions {
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        preset: preset.unwrap_or(InitPreset::Zsumz),
        baseline: baseline.is_some(),
    }))
}

fn parse_preset(value: &str) -> Result<InitPreset, CliError> {
    match value {
        "zsumz" => Ok(InitPreset::Zsumz),
        "rust" => Ok(InitPreset::Rust),
        other => Err(CliError::new(format!(
            "unsupported init preset {other:?}; expected \"zsumz\" or \"rust\""
        ))),
    }
}
