//! Schema-1 contracts migrate to exact schema-2 syntax through an explicit write.

use std::fmt::Write as _;

use crate::app::{args::MigrateConfigOptions, error::CliError};

use super::{CommandResult, config_edit};

pub(crate) fn migrate_config(options: &MigrateConfigOptions) -> Result<CommandResult, CliError> {
    let plan = config_edit::migration(&options.root, &options.config)?;
    if options.write {
        plan.write()?;
    }
    let action = if options.write {
        "Migrated"
    } else {
        "Would migrate"
    };
    let mut text = format!(
        "{action} {} contract source(s) to schema 2\n",
        plan.changed()
    );
    for path in plan.changed_paths() {
        let _ = writeln!(text, "  {path}");
    }
    if !options.write && plan.changed() > 0 {
        text.push_str("Rerun with `--write` to apply this deterministic migration.\n");
    }
    Ok(CommandResult::success(text))
}
