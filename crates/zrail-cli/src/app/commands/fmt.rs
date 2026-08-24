//! Canonical TOML layout for every exact contract fragment.

use crate::app::{args::FmtOptions, error::CliError};

use super::{CommandResult, config_edit};

pub(crate) fn format_config(options: &FmtOptions) -> Result<CommandResult, CliError> {
    let plan = config_edit::format(&options.root, &options.config)?;
    if options.check && plan.changed() > 0 {
        return Ok(CommandResult::status(
            format!(
                "{} contract source(s) require `zrail fmt`\n",
                plan.changed()
            ),
            1,
        ));
    }
    if !options.check {
        plan.write()?;
    }
    Ok(CommandResult::success(format!(
        "Formatted {} contract source(s)\n",
        plan.changed()
    )))
}
