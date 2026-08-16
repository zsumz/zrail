//! Lock format and interpretation epochs advance independently and coherently.

use super::LockError;

pub(super) fn validate_epochs(schema: u64, semantics: u64) -> Result<(), LockError> {
    if semantics >= 3 && schema < 3 {
        return Err(LockError(format!(
            "zrail.lock semantics {semantics} require lock schema 3 or newer"
        )));
    }
    if semantics >= 4 && schema < 4 {
        return Err(LockError(format!(
            "zrail.lock semantics {semantics} require lock schema 4 or newer"
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "compatibility_test.rs"]
mod compatibility_test;
