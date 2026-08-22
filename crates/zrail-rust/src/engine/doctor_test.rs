//! Doctor readiness is explicit and machine-readable.

use zrail_core::{LOCK_SCHEMA, LockFile};

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

#[test]
fn unsupported_required_lock_schema_is_not_ready() {
    let candidate = LockFile::new("0".repeat(64));
    let mut future = candidate.clone();
    future.schema = LOCK_SCHEMA + 1;

    assert_eq!(
        doctor_status(true, Some(&future), &candidate),
        "lock-schema-mismatch"
    );
    assert!(!report("lock-schema-mismatch").is_ready());
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
