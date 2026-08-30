//! Exact shape and coverage must describe each actual compilation-domain representation.

use super::type_policy_test::{error_count, report, repository, reset, write};

#[test]
fn complementary_fields_cannot_pass_as_one_combined_shape() {
    let policy = format!(
        "{}\n{SHAPE}\n{}",
        worlds(&[false, true]),
        field("other", "u64")
    );
    let root = fixture(
        &policy,
        r#"struct Permit {
        #[cfg(feature = "extra")] epoch: u64,
        #[cfg(not(feature = "extra"))] other: u64,
    }"#,
    );
    let report = report(&root);
    assert_eq!(error_count(&report, "RUST-TYPE-002"), 2, "{report}");
    assert!(report.contains("feature-world=off"), "{report}");
    assert!(report.contains("feature-world=on"), "{report}");
    reset(&root);
}

#[test]
fn inactive_fields_are_absent_from_enforcement_and_coverage() {
    let root = fixture(
        &format!("{}\n{SHAPE}", worlds(&[false])),
        r#"struct Permit {
        epoch: u64,
        #[cfg(feature = "extra")] other: UnknownInactiveType,
    }"#,
    );
    let report = report(&root);
    assert_eq!(error_count(&report, "RUST-TYPE-002"), 0, "{report}");
    let coverage = super::governed_surface_report(&root, "zrail.toml".as_ref()).unwrap();
    let observation = &coverage.type_policies[0].observations[0];
    assert!(observation.allowed);
    assert_eq!(observation.quality, zrail_core::AnalysisQuality::Exact);
    assert_eq!(observation.fields.as_ref().unwrap().len(), 1);
    assert_eq!(observation.compilation_domains.len(), 1);
    reset(&root);
}

#[test]
fn target_predicate_field_is_unresolved_not_present() {
    let root = fixture(
        &format!("{}\n{SHAPE}", worlds(&[false])),
        "struct Permit { #[cfg(unix)] epoch: u64 }",
    );
    let report = report(&root);
    assert_eq!(error_count(&report, "RUST-TYPE-002"), 1, "{report}");
    assert!(
        report.contains("field \"epoch\" availability is unresolved"),
        "{report}"
    );
    assert!(
        report.contains("mode=library;feature-world=off"),
        "{report}"
    );
    assert!(report.contains("analysis: unresolved"), "{report}");
    reset(&root);
}

#[test]
fn every_world_must_match_and_guard_only_changes_are_detected() {
    let source = r#"struct Permit {
        #[cfg(not(feature = "extra"))] epoch: u64,
        #[cfg(feature = "extra")] epoch: u32,
    }"#;
    let root = fixture(&format!("{}\n{SHAPE}", worlds(&[false, true])), source);
    let first = report(&root);
    assert_eq!(error_count(&first, "RUST-TYPE-002"), 1, "{first}");
    assert!(first.contains("feature-world=on"), "{first}");
    let coverage = super::governed_surface_report(&root, "zrail.toml".as_ref()).unwrap();
    let observations = &coverage.type_policies[0].observations;
    assert_eq!(observations.len(), 2);
    assert_eq!(
        observations
            .iter()
            .filter(|observation| observation.allowed)
            .count(),
        1
    );
    write(
        &root,
        "src/lib.rs",
        &source.replace("not(feature = \"extra\")", "any()"),
    );
    let changed = report(&root);
    assert_eq!(error_count(&changed, "RUST-TYPE-002"), 2, "{changed}");
    reset(&root);
}

#[test]
fn test_mode_shape_is_governed_only_when_selected() {
    let source = "struct Permit { epoch: u64, #[cfg(test)] other: u64 }";
    let root = fixture(SHAPE, source);
    assert_eq!(error_count(&report(&root), "RUST-TYPE-002"), 0);
    let root_all = fixture(&SHAPE.replace("production", "all"), source);
    let report = report(&root_all);
    assert_eq!(error_count(&report, "RUST-TYPE-002"), 1, "{report}");
    assert!(report.contains("mode=library-test"), "{report}");
    reset(&root);
    reset(&root_all);
}

#[test]
fn leaf_module_shape_uses_domain_active_child_modules() {
    let policy = SHAPE.replace(
        "visibility = \"private\"\nreason",
        "visibility = \"private\"\nleaf_module = true\nreason",
    );
    let root = fixture(
        &format!("{}\n{policy}", worlds(&[false])),
        "struct Permit { epoch: u64 }\n#[cfg(feature = \"extra\")] mod child {}\n",
    );
    assert_eq!(error_count(&report(&root), "RUST-TYPE-002"), 0);
    write(
        &root,
        "src/lib.rs",
        "struct Permit { epoch: u64 }\n#[cfg(unix)] mod child {}\n",
    );
    let report = report(&root);
    assert_eq!(error_count(&report, "RUST-TYPE-002"), 1, "{report}");
    assert!(report.contains("child-module availability is unresolved"));
    reset(&root);
}

pub(crate) fn fixture(policy: &str, source: &str) -> std::path::PathBuf {
    let root = repository(policy);
    write(
        &root,
        "Cargo.toml",
        concat!(
            "[package]\nname = \"policy-app\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
            "[features]\nextra = []\n"
        ),
    );
    write(&root, "src/lib.rs", source);
    root
}

pub(crate) fn worlds(selected: &[bool]) -> String {
    selected.iter().map(|enabled| format!(
        "[[source.rust.feature_worlds]]\nname = {:?}\nreason = \"Reviewed shape world.\"\n\
         [[source.rust.feature_worlds.packages]]\npackage = \"policy-app\"\ndefault_features = false\nfeatures = {}\n",
        if *enabled { "on" } else { "off" }, if *enabled { "[\"extra\"]" } else { "[]" }
    )).collect::<Vec<_>>().join("\n")
}

fn field(name: &str, ty: &str) -> String {
    format!(
        "[[source.rust.types.fields]]\nname = {name:?}\ntype = {ty:?}\nvisibility = \"private\"\n"
    )
}

pub(crate) const SHAPE: &str = r#"[[source.rust.types]]
name = "permit-shape"
match = "crate::Permit"
path = "src/lib.rs"
reachability = "production"
visibility = "private"
reason = "The authority representation is exact in each compilation domain."
[[source.rust.types.fields]]
name = "epoch"
type = "u64"
visibility = "private"
"#;
