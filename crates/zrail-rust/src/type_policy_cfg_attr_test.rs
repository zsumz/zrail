//! Type enforcement consumes only cfg-attribute effects active in an exact world.

use super::type_policy_test::{
    FEATURE_WORLD_POLICY, error_count, report, repository, reset, write,
};

#[test]
fn cfg_attr_derive_and_opacity_follow_inactive_and_active_worlds() {
    let inactive = repository(FEATURE_WORLD_POLICY);
    write_fixture(&inactive);
    let inactive_report = report(&inactive);
    assert_eq!(error_count(&inactive_report, "RUST-TYPE-003"), 0);
    assert_eq!(error_count(&inactive_report, "RUST-TYPE-005"), 0);
    reset(&inactive);

    let policy = FEATURE_WORLD_POLICY.replace("features = []", "features = [\"dup\"]");
    let active = repository(&policy);
    write_fixture(&active);
    let active_report = report(&active);
    assert_eq!(error_count(&active_report, "RUST-TYPE-003"), 1);
    assert_eq!(error_count(&active_report, "RUST-TYPE-005"), 1);
    reset(&active);
}

fn write_fixture(root: &std::path::Path) {
    write(
        root,
        "Cargo.toml",
        concat!(
            "[package]\nname = \"policy-app\"\nversion = \"0.0.0\"\n",
            "edition = \"2024\"\n\n[features]\ndup = []\n",
        ),
    );
    write(
        root,
        "src/lib.rs",
        concat!(
            "//! facade\n",
            "#[cfg_attr(feature = \"dup\", derive(Clone))]\n",
            "struct Ticket;\n",
            "#[cfg_attr(feature = \"dup\", reviewed::attribute)]\n",
            "fn generated() {}\n",
        ),
    );
}
