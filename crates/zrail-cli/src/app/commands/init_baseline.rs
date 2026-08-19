//! Baseline preparation replaces only the newly created provisional contract.

use std::path::Path;

use zrail_core::replace_text;
use zrail_rust::{BaselinePlan, discover_baseline};

use crate::app::args::InitPreset;

use super::init_template;

pub(super) fn apply(
    root: &Path,
    config: &Path,
    roots: &[String],
    preset: InitPreset,
) -> Result<BaselinePlan, String> {
    let baseline =
        discover_baseline(root, Path::new("zrail.toml")).map_err(|error| error.to_string())?;
    replace_text(config, &init_template::render(roots, preset, &baseline))?;
    Ok(baseline)
}

#[cfg(test)]
#[path = "init_baseline_test.rs"]
mod init_baseline_test;
