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

fn errors(contract: &crate::Contract) -> String {
    let mut errors = ValidationErrors::new();
    validate(contract, &mut errors);
    errors.finish().join("\n")
}

fn role(path: &str, reason: &str) -> FileRoleContract {
    FileRoleContract {
        path: path.into(),
        role: FileRole::Implementation,
        reason: reason.into(),
    }
}
