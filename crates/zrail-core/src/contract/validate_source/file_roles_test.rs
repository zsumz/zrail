//! File-role override contract validation.

use crate::{FileRole, FileRoleContract};

use crate::contract::validate_fixture_test::minimal_contract;

use super::{ValidationErrors, validate};

#[test]
fn overrides_require_exact_unique_rust_paths_and_reasons() {
    let mut contract = minimal_contract();
    contract.source.rust.file_roles = vec![
        role("src/../lib.rs", "reviewed"),
        role("src/plain.txt", "reviewed"),
        role("src/plain.txt", ""),
    ];
    let errors = errors(&contract);

    assert!(errors.contains("not canonical"), "{errors}");
    assert!(errors.contains("must name .rs"), "{errors}");
    assert!(errors.contains("duplicate file-role"), "{errors}");
    assert!(errors.contains("requires a reason"), "{errors}");
}

#[test]
fn generated_source_cannot_be_reclassified() {
    let mut contract = minimal_contract();
    contract
        .source
        .rust
        .generated
        .push(crate::GeneratedSourceContract {
            root: "src/generated".into(),
            manifest: "src/generated/MANIFEST.json".into(),
            inputs: Vec::new(),
            target: 100,
            hard: 200,
            reason: "compiler-owned output".into(),
            auxiliary: Vec::new(),
        });
    contract.source.rust.file_roles = vec![role("src/generated/model.rs", "reviewed")];

    assert!(
        errors(&contract).contains("generated source may not have"),
        "generated override should fail"
    );
}

#[test]
fn test_facades_require_structure_and_implementation_cannot_select_it() {
    let mut contract = minimal_contract();
    let mut selected = role("tests/suite.rs", "Reviewed test wiring.");
    selected.role = FileRole::TestFacade;
    contract.source.rust.file_roles = vec![selected];
    assert!(errors(&contract).contains("requires an enforced facade mode"));
    contract.source.rust.file_roles[0].mode = Some(crate::FacadeMode::Allow);
    assert!(errors(&contract).contains("requires an enforced facade mode"));
    contract.source.rust.file_roles[0].mode = Some(crate::FacadeMode::WiringOnly);
    assert!(errors(&contract).is_empty());
    contract.source.rust.file_roles[0].role = FileRole::Implementation;
    assert!(errors(&contract).contains("cannot set facade mode"));
}

#[test]
fn facade_modes_and_exact_role_fields_parse_strictly() {
    for spelling in ["declarative", "wiring-only", "wiring-reexports"] {
        let text = format!(
            "path = 'tests/suite.rs'\nrole = 'test-facade'\nmode = '{spelling}'\nreason = 'Reviewed.'\n"
        );
        assert!(toml::from_str::<FileRoleContract>(&text).is_ok());
        assert!(toml::from_str::<FileRoleContract>(&format!("{text}unknown = true\n")).is_err());
        assert!(toml::from_str::<FileRoleContract>(&text.replace(spelling, "wiring-ish")).is_err());
    }
}

fn errors(contract: &crate::Contract) -> String {
    let mut errors = ValidationErrors::new();
    validate(contract, &mut errors);
    errors.finish().join("\n")
}

fn role(path: &str, reason: &str) -> FileRoleContract {
    FileRoleContract {
        path: path.into(),
        role: FileRole::Implementation,
        mode: None,
        reason: reason.into(),
    }
}
