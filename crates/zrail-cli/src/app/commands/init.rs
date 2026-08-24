//! Create a conservative local contract and optionally its first exact lock.

#[path = "init_baseline.rs"]
mod init_baseline;
#[path = "init_selection.rs"]
mod init_selection;

use std::{fs, path::Path};

use zrail_core::{ReportStatus, create_text, load_contract, repository_file};
use zrail_rust::{
    BaselinePlan, build_lock, check_repository_with_lock, discover_source_roots_with_selection,
};

use crate::app::{args::InitOptions, error::CliError};

use super::{CommandResult, init_preset, init_template};

pub(crate) fn init(options: &InitOptions) -> Result<CommandResult, CliError> {
    let root = fs::canonicalize(&options.root).map_err(|error| {
        CliError::new(format!(
            "open repository {}: {error}",
            options.root.display()
        ))
    })?;
    let config = repository_file(&root, Path::new("zrail.toml")).map_err(CliError::new)?;
    let lock = repository_file(&root, Path::new("zrail.lock")).map_err(CliError::new)?;
    ensure_vacant(&config, &lock)?;
    let selection = init_selection::load(&root, options)?;
    let roots = discover_source_roots_with_selection(&root, &selection)
        .map_err(|error| CliError::new(error.to_string()))?;
    let mut baseline = BaselinePlan::empty();
    let template = init_template::render(&roots, selection.exclusions(), options.preset, &baseline);
    create_text(&config, &template).map_err(CliError::new)?;
    if let Err(error) = load_contract(&root, Path::new("zrail.toml")) {
        return Err(rollback_error(&config, &error.to_string()));
    }
    if !options.baseline {
        return Ok(contract_only_result(
            &root,
            options,
            &roots,
            selection.exclusions().len(),
        ));
    }
    baseline = match init_baseline::apply(&root, &config) {
        Ok(baseline) => baseline,
        Err(error) => return Err(rollback_error(&config, &error)),
    };
    let candidate = match build_lock(&root, Path::new("zrail.toml")) {
        Ok(candidate) => candidate,
        Err(error) => {
            return Err(rollback_error(&config, &error.to_string()));
        }
    };
    let checked = match check_repository_with_lock(&root, Path::new("zrail.toml"), &candidate) {
        Ok(checked) => checked,
        Err(error) => {
            return Err(rollback_error(&config, &error.to_string()));
        }
    };
    if checked.report.status != ReportStatus::Pass {
        remove_created(&config)?;
        return Ok(CommandResult::status(
            format!(
                "zrail init refused to write a lock for a failing {} preset\n\n{}",
                options.preset.name(),
                checked.report.human()
            ),
            1,
        ));
    }
    let rendered = match candidate.render() {
        Ok(rendered) => rendered,
        Err(error) => {
            return Err(rollback_error(&config, &error.to_string()));
        }
    };
    if let Err(error) = create_text(&lock, &rendered) {
        return Err(rollback_error(&config, &error));
    }
    Ok(CommandResult::success(format!(
        concat!(
            "Initialized {}\n",
            "Created zrail.toml and zrail.lock\n",
            "Preset: {}\n",
            "Adoption: {}\n",
            "Recorded debt: {} ratchets\n",
            "Source roots: {}\n",
            "Next: run `zrail check`\n",
        ),
        root.display(),
        options.preset.name(),
        init_preset::adoption_name(options.baseline),
        baseline.ratchets.len(),
        roots.join(", ")
    )))
}

fn contract_only_result(
    root: &Path,
    options: &InitOptions,
    roots: &[String],
    exclusions: usize,
) -> CommandResult {
    CommandResult::success(format!(
        concat!(
            "Initialized {}\n",
            "Created zrail.toml\n",
            "Preset: {}\n",
            "Adoption: contract only\n",
            "Exclusions: {}\n",
            "Source roots: {}\n",
            "Next: review zrail.toml, then run `zrail baseline --dry-run`\n",
        ),
        root.display(),
        options.preset.name(),
        exclusions,
        roots.join(", ")
    ))
}

fn ensure_vacant(config: &Path, lock: &Path) -> Result<(), CliError> {
    if fs::symlink_metadata(config).is_ok() || fs::symlink_metadata(lock).is_ok() {
        return Err(CliError::new(
            "zrail.toml or zrail.lock already exists; init never overwrites architecture",
        ));
    }
    Ok(())
}

fn remove_created(path: &Path) -> Result<(), CliError> {
    fs::remove_file(path)
        .map_err(|error| CliError::new(format!("remove partial {}: {error}", path.display())))
}

fn rollback_error(path: &Path, original: &str) -> CliError {
    match remove_created(path) {
        Ok(()) => CliError::new(original),
        Err(cleanup) => CliError::new(format!("{original}; {}", cleanup.message)),
    }
}

#[cfg(test)]
#[path = "init_test.rs"]
mod init_test;
