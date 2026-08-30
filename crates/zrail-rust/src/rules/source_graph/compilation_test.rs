//! Procedural macro execution and its test harness retain distinct host-side domains.

use super::{CargoTargetKind, CompilationMode, target_domains};

#[test]
fn proc_macro_domains_are_build_reachable_and_only_the_harness_enables_test() {
    let domains = target_domains(CargoTargetKind::ProcMacro);
    assert_eq!(domains.len(), 2);
    for ((mode, reachability), expected) in domains
        .iter()
        .zip([CompilationMode::ProcMacro, CompilationMode::ProcMacroTest])
    {
        assert_eq!(*mode, expected);
        assert!(!reachability.is_production());
        assert_eq!(
            mode.enables_cfg_test(),
            *mode == CompilationMode::ProcMacroTest
        );
        assert!(mode.canonical_name().starts_with("proc-macro"));
    }
}
