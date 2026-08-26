//! Exact test mirrors require live Cargo reachability and content-bound passed receipts.

#[path = "test_mirror_receipts/fixture.rs"]
mod fixture;

use std::fs;

use fixture::MirrorFixture;
use zrail_core::ReportStatus;
use zrail_rust::{
    render_test_mirror_receipts, test_mirror_plan, verify_test_mirror_plan, verify_test_mirrors,
};

#[test]
fn mirror_plan_scales_execution_without_weakening_exact_receipts() {
    let fixture = MirrorFixture::new("plan");
    fixture.write_valid_receipt("runner 1.2.3", "state_transitions", "passed");
    let root = fixture.path("");
    let plan = test_mirror_plan(&root, std::path::Path::new("zrail.toml"))
        .expect("build exact mirror plan");
    let source = plan.json().expect("render mirror plan");

    assert_eq!(
        verify_test_mirror_plan(&root, std::path::Path::new("zrail.toml"), &source)
            .expect("verify current plan"),
        plan
    );
    let verified = verify_test_mirrors(&root, std::path::Path::new("zrail.toml"), &source)
        .expect("verify exact receipt set");
    assert_eq!(verified.report.status, ReportStatus::Pass);

    fs::write(
        fixture.path("src/state.rs"),
        "//! Changed state behavior.\npub fn transition(value: usize) -> usize { value + 2 }\n",
    )
    .expect("drift production input");
    assert!(
        verify_test_mirror_plan(&root, std::path::Path::new("zrail.toml"), &source)
            .expect_err("stale plan must fail")
            .to_string()
            .contains("differs")
    );
}

#[test]
fn trusted_group_results_render_the_complete_receipt_set_without_execution() {
    let fixture = MirrorFixture::new("bulk-receipts");
    let root = fixture.path("");
    let plan = test_mirror_plan(&root, std::path::Path::new("zrail.toml"))
        .expect("build exact mirror plan");
    let mirror = &plan.mirrors[0];
    let source = plan.json().expect("render mirror plan");
    let results = format!(
        concat!(
            "{{\"schema\":1,\"plan_sha256\":\"{}\",",
            "\"producer\":\"trusted-runner 1.2.3\",\"groups\":[{{",
            "\"execution_group\":\"{}\",\"tests\":[{{",
            "\"policy_id\":\"{}\",\"status\":\"passed\"}}]}}]}}"
        ),
        plan.plan_sha256, mirror.execution_group, mirror.policy_id
    );

    let bundle =
        render_test_mirror_receipts(&root, std::path::Path::new("zrail.toml"), &source, &results)
            .expect("render current receipts");
    assert_eq!(bundle.receipts.len(), 1);
    let artifact = &bundle.receipts[0];
    fs::create_dir_all(fixture.path("evidence")).expect("create receipt directory");
    fs::write(fixture.path(&artifact.path), &artifact.source).expect("write rendered receipt");
    fixture.write_candidate_lock();

    let checked = fixture.check();
    assert_eq!(
        checked.report.status,
        ReportStatus::Pass,
        "{}",
        checked.report.human()
    );
}

#[test]
fn exact_cargo_reachable_mirror_with_passed_receipt_is_authoritative() {
    let fixture = MirrorFixture::new("valid");
    fixture.write_valid_receipt("runner 1.2.3", "state_transitions", "passed");
    let lock = fixture.write_candidate_lock();

    assert_eq!(lock.execution_receipts.len(), 1);
    assert_eq!(lock.execution_receipts[0].production, "src/state.rs");
    assert_eq!(lock.execution_receipts[0].test, "tests/state_test.rs");

    let checked = fixture.check();
    assert_eq!(
        checked.report.status,
        ReportStatus::Pass,
        "{}",
        checked.report.human()
    );
}

#[test]
fn mirror_requires_production_and_cargo_test_reachability_plus_exact_declaration() {
    let fixture = MirrorFixture::new("reachability");
    fs::write(
        fixture.path("src/orphan.rs"),
        "//! Orphan.\npub fn value() {}\n",
    )
    .expect("write orphan production");
    fs::write(
        fixture.path("Cargo.toml"),
        concat!(
            "[package]\nname='mirror-fixture'\nversion='0.0.0'\nedition='2024'\n",
            "autotests=false\n\n[[test]]\nname='active'\npath='tests/active.rs'\n",
        ),
    )
    .expect("restrict Cargo tests");
    fs::write(
        fixture.path("tests/active.rs"),
        "//! Active test target.\n#[test]\nfn active() {}\n",
    )
    .expect("write active test");
    fs::write(
        fixture.path("tests/state_test.rs"),
        "//! Unreachable test source.\n#[test]\nfn different_name() {}\n",
    )
    .expect("change named test");
    fixture.write_contract("src/orphan.rs", "tests/state_test.rs", "state_transitions");
    fixture.write_valid_receipt_for(
        "src/orphan.rs",
        "tests/state_test.rs",
        "runner 1.2.3",
        "state_transitions",
        "passed",
    );
    fixture.write_candidate_lock();

    let checked = fixture.check();
    assert!(MirrorFixture::has(&checked, "MIRROR-002"));
    assert!(MirrorFixture::has(&checked, "MIRROR-004"));
    assert!(MirrorFixture::has(&checked, "MIRROR-010"));
}

#[test]
fn stale_explicit_paths_and_missing_receipts_fail_closed() {
    let fixture = MirrorFixture::new("stale");
    fixture.write_contract(
        "src/missing.rs",
        "tests/missing_test.rs",
        "missing_behavior",
    );
    fixture.write_candidate_lock();

    let checked = fixture.check();
    assert!(MirrorFixture::has(&checked, "MIRROR-001"));
    assert!(MirrorFixture::has(&checked, "MIRROR-003"));
    assert!(MirrorFixture::has(&checked, "RECEIPT-001"));
}

#[test]
fn receipt_schema_digest_pass_status_and_exact_bytes_fail_closed() {
    let fixture = MirrorFixture::new("receipt");
    fixture.write_valid_receipt("runner 1.2.3", "state_transitions", "passed");
    fixture.write_candidate_lock();

    let original = fs::read_to_string(fixture.path("evidence/state.json")).expect("read receipt");
    fs::write(
        fixture.path("evidence/state.json"),
        original.replace("runner 1.2.3", "runner 1.2.4"),
    )
    .expect("change producer bytes");
    let drifted = fixture.check();
    assert!(MirrorFixture::has(&drifted, "LOCK-042"));

    fixture.write_valid_receipt("runner 1.2.4", "state_transitions", "passed");
    let source =
        fs::read_to_string(fixture.path("evidence/state.json")).expect("read exact receipt");
    fs::write(
        fixture.path("evidence/state.json"),
        source.replace("\"id\":\"state_transitions\"", "\"id\":\"other_test\""),
    )
    .expect("replace receipt test id");
    fixture.write_candidate_lock();
    let missing_test = fixture.check();
    assert!(MirrorFixture::has(&missing_test, "RECEIPT-004"));

    fixture.write_valid_receipt("runner 1.2.4", "state_transitions", "failed");
    fixture.write_candidate_lock();
    let failed = fixture.check();
    assert!(MirrorFixture::has(&failed, "RECEIPT-005"));

    fixture.write_receipt("runner", &"0".repeat(64), "state_transitions", "passed");
    fixture.write_candidate_lock();
    let malformed = fixture.check();
    assert!(MirrorFixture::has(&malformed, "RECEIPT-002"));

    fixture.write_receipt(
        "runner 1.2.4",
        &"0".repeat(64),
        "state_transitions",
        "passed",
    );
    fixture.write_candidate_lock();
    let mismatched = fixture.check();
    assert!(MirrorFixture::has(&mismatched, "RECEIPT-003"));
}

#[test]
fn receipt_binds_manifests_lock_shared_inputs_and_execution_identity() {
    let fixture = MirrorFixture::new("context-inputs");
    fixture.write_valid_receipt("runner 1.2.3", "state_transitions", "passed");
    fixture.write_candidate_lock();
    fs::write(
        fixture.path("src/lib.rs"),
        "//! Changed shared module.\npub mod state;\n",
    )
    .expect("change shared input");
    assert!(MirrorFixture::has(&fixture.check(), "RECEIPT-003"));

    fixture.write_valid_receipt("runner 1.2.3", "state_transitions", "passed");
    fixture.write_candidate_lock();
    let manifest = fs::read_to_string(fixture.path("Cargo.toml")).expect("read manifest");
    fs::write(
        fixture.path("Cargo.toml"),
        format!("{manifest}\n# reviewed input changed\n"),
    )
    .expect("change manifest input");
    assert!(MirrorFixture::has(&fixture.check(), "RECEIPT-003"));

    let context = MirrorFixture::new("execution-context");
    context.write_valid_receipt("runner 1.2.3", "state_transitions", "passed");
    let contract = fs::read_to_string(context.path("zrail.toml")).expect("read contract");
    fs::write(
        context.path("zrail.toml"),
        contract.replace(
            "target = \"x86_64-unknown-linux-gnu\"",
            "target = \"aarch64-unknown-linux-gnu\"",
        ),
    )
    .expect("change execution target");
    context.write_candidate_lock();
    let checked = context.check();
    assert!(MirrorFixture::has(&checked, "RECEIPT-003"));
    assert!(MirrorFixture::has(&checked, "RECEIPT-007"));
}

#[test]
fn selected_execution_package_must_own_both_mirror_sources() {
    let fixture = MirrorFixture::new("execution-package");
    fixture.write_valid_receipt("runner 1.2.3", "state_transitions", "passed");
    let contract = fs::read_to_string(fixture.path("zrail.toml")).expect("read contract");
    fs::write(
        fixture.path("zrail.toml"),
        contract.replace(
            "package = \"mirror-fixture\"",
            "package = \"other-package\"",
        ),
    )
    .expect("change execution package");
    fixture.write_candidate_lock();
    let checked = fixture.check();
    assert!(MirrorFixture::has(&checked, "MIRROR-007"));
    assert!(MirrorFixture::has(&checked, "RECEIPT-007"));
}
