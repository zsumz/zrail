//! Locked execution-receipt canonicalization coverage.

use crate::{LockFile, LockedExecutionReceipt};

#[test]
fn canonicalization_orders_receipts_and_rejects_reused_test_files() {
    let mut ordered = LockFile::new("0".repeat(64));
    ordered.execution_receipts = vec![
        receipt("src/z.rs", "tests/z.rs"),
        receipt("src/a.rs", "tests/a.rs"),
    ];
    ordered.canonicalize().expect("canonical receipt lock");
    assert_eq!(ordered.execution_receipts[0].production, "src/a.rs");

    let mut reused = LockFile::new("0".repeat(64));
    reused.execution_receipts = vec![
        receipt("src/a.rs", "tests/shared.rs"),
        receipt("src/b.rs", "tests/shared.rs"),
    ];
    assert!(
        reused
            .canonicalize()
            .expect_err("reused test file must fail")
            .to_string()
            .contains("duplicate locked execution receipt test")
    );
}

fn receipt(production: &str, test: &str) -> LockedExecutionReceipt {
    LockedExecutionReceipt {
        production: production.into(),
        test: test.into(),
        name: "covers_behavior".into(),
        receipt: format!("evidence/{}.json", production.replace(['/', '.'], "-")),
        sha256: "1".repeat(64),
        input_sha256: "2".repeat(64),
        producer: "runner 1.2.3".into(),
    }
}
