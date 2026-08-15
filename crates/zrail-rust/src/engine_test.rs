//! Doctor readiness is explicit and machine-readable.

use zrail_core::LockFile;

use super::{DoctorReport, doctor_status};

#[test]
fn ready_status_is_ready() {
    assert!(report("ready").is_ready());
}

#[test]
fn stale_status_is_not_ready() {
    assert!(!report("lock-stale").is_ready());
}

#[test]
fn generated_and_ratcheted_state_can_require_a_lock() {
    let lock = LockFile::new("0".repeat(64));

    assert_eq!(doctor_status(true, None, &lock), "lock-missing");
    assert_eq!(doctor_status(false, None, &lock), "ready");
}

fn report(status: &str) -> DoctorReport {
    DoctorReport {
        schema: 1,
        root: ".".into(),
        config: "zrail.toml".into(),
        lock: "zrail.lock".into(),
        contract_sha256: "0".repeat(64),
        packages: 1,
        rust_files: 1,
        contract_sources: 1,
        status: status.into(),
    }
}
