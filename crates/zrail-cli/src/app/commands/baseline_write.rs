//! Baseline persistence rolls back the contract if lock replacement fails.

use std::path::Path;

use zrail_core::{LockFile, replace_text};

pub(super) fn write(
    config: &Path,
    lock: &Path,
    original_contract: &str,
    patched_contract: &str,
    candidate: &LockFile,
) -> Result<(), String> {
    let rendered_lock = candidate.render().map_err(|error| error.to_string())?;
    replace_text(config, patched_contract)?;
    if let Err(error) = replace_text(lock, &rendered_lock) {
        return match replace_text(config, original_contract) {
            Ok(()) => Err(error),
            Err(rollback) => Err(format!(
                "{error}; additionally failed to restore {}: {rollback}",
                config.display()
            )),
        };
    }
    Ok(())
}
