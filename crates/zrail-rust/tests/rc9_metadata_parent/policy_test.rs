//! Bound parent-path proof reuses the original metadata body and existing policy fragment.

#[path = "model.rs"]
mod model;
#[path = "qualification.rs"]
mod qualification;

#[test]
fn metadata_parent_original_body_and_registry_are_unchanged() {
    model::source_binding();
}

#[test]
fn metadata_parent_fixed_paths_preserve_all_twelve_parent_mappings() {
    model::path_proof(&model::project().canonicalize().expect("absolute project"));
    assert!(std::path::Path::new("").parent().is_none());
    assert!(
        model::project()
            .ancestors()
            .last()
            .expect("root")
            .parent()
            .is_none()
    );
}

#[test]
fn metadata_parent_mapping_rejects_changed_license_authority() {
    let policies = model::policies();
    for case in 0..7 {
        let mut changed = policies.clone();
        match case {
            0 => {
                changed.pop();
            }
            1 => {
                changed[0].name = "unrelated-license-copy".into();
            }
            2 => {
                changed[1].include = changed[0].include.clone();
            }
            3 => {
                changed[0].include = vec!["Cargo.toml".into()];
            }
            4 => {
                changed[0].exclude = vec!["LICENSE".into()];
            }
            5 => {
                changed[0].predicate = zrail_core::RepositoryFilePredicate::BytesEqual {
                    other: "other".into(),
                    utf8: true,
                };
            }
            _ => {
                changed[0].predicate = zrail_core::RepositoryFilePredicate::BytesEqual {
                    other: "LICENSE".into(),
                    utf8: false,
                };
            }
        }
        assert!(!model::mapping_valid(&changed), "changed mapping {case}");
    }
}

#[test]
fn metadata_parent_original_and_native_accept_exact_frozen_fixture_inputs() {
    let root = model::project().join("crates/zrail-testkit/tests/fixtures/rc9/metadata/valid");
    model::original_and_native(&root);
}

#[test]
#[ignore = "requires frozen snapshots and a fresh ZRAIL_RC9_METADATA_PARENT_REPORT"]
fn qualify_frozen_metadata_parent() {
    qualification::run();
}
