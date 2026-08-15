//! Preset selection and debt adoption remain distinct onboarding vocabulary.

pub(super) const fn adoption_name(baseline: bool) -> &'static str {
    if baseline { "baseline" } else { "strict" }
}

#[cfg(test)]
#[path = "init_preset_test.rs"]
mod init_preset_test;
