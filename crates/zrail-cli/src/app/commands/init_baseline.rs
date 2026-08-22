//! Initialization delegates measurable-debt planning to `zrail baseline`.

use std::path::Path;

use zrail_core::replace_text;
use zrail_rust::BaselinePlan;

use super::super::baseline_plan;

pub(super) fn apply(root: &Path, config: &Path) -> Result<BaselinePlan, String> {
    let prepared = baseline_plan::prepare(root, Path::new("zrail.toml"), None)
        .map_err(|error| error.message)?;
    replace_text(config, &prepared.patched_contract)?;
    Ok(BaselinePlan {
        size: None,
        ratchets: prepared.added,
    })
}

#[cfg(test)]
#[path = "init_baseline_test.rs"]
mod init_baseline_test;
