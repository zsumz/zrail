//! Global strict facade selectors continue to protect newly added test modules.

#[path = "strict_facades/fixture.rs"]
mod fixture;

use fixture::{Repository, TEST_FACADE};
use zrail_core::ReportStatus;

#[test]
fn newly_added_test_mod_files_receive_global_wiring_policy() {
    for mode in ["wiring-only", "wiring-reexports"] {
        let repository = Repository::new(mode, TEST_FACADE);
        repository.write("tests/suite/mod.rs", "//! Test wiring.\nmod cases;\n");
        repository.write(
            "tests/suite.rs",
            "//! Test entrypoint.\n#[path = \"suite/mod.rs\"] mod suite;\n",
        );
        repository.lock();
        assert_eq!(repository.check().report.status, ReportStatus::Pass);
        let explanation = repository.explain("tests/suite/mod.rs");
        assert_eq!(explanation.reachability, "test-only");
        assert_eq!(explanation.design_target, Some(320));
        assert_eq!(
            explanation.facade_mode,
            Some(if mode == "wiring-only" {
                zrail_core::FacadeMode::WiringOnly
            } else {
                zrail_core::FacadeMode::WiringReexports
            })
        );
        for violation in ["pub struct Misplaced;", "#[test] fn misplaced() {}"] {
            repository.write(
                "tests/suite/mod.rs",
                &format!("//! Test wiring.\nmod cases;\n{violation}\n"),
            );
            let result = repository.check();
            assert!(result.report.analysis.complete);
            assert!(
                result
                    .report
                    .findings
                    .iter()
                    .any(|finding| finding.id == "RUST-FACADE-001"
                        && finding.path.as_deref() == Some("tests/suite/mod.rs")),
                "{mode} {violation}: {:?}",
                result.report
            );
        }
    }
}

#[test]
fn legacy_modes_preserve_test_source_selection() {
    for mode in ["allow", "declarative"] {
        let repository = Repository::new(mode, TEST_FACADE);
        repository.write(
            "tests/suite/mod.rs",
            "//! Test wiring.\nmod cases;\n#[test] fn scenario() {}\n",
        );
        repository.write(
            "tests/suite.rs",
            "//! Test entrypoint.\n#[path = \"suite/mod.rs\"] mod suite;\n",
        );
        repository.lock();
        assert_eq!(repository.check().report.status, ReportStatus::Pass);
        assert_eq!(repository.explain("tests/suite/mod.rs").facade_mode, None);
    }
}
