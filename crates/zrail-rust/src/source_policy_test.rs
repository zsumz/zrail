//! Optional handwritten budgets do not disable explicit generated-source budgets.

use zrail_core::{
    FacadeMode, FileRole, FileRoleContract, GeneratedSourceContract, HygieneContract,
    LintSuppressionMode, ModuleDocsMode, PolicyMode, RustSourceContract, TestMode,
};

use crate::{
    inventory::FileClass,
    source::{Reachability, ReachabilityKind},
};

use super::{budget_for, effective_file_role};

#[test]
fn generated_budget_remains_enforced_without_a_handwritten_size_policy() {
    let rust = RustSourceContract {
        module_docs: ModuleDocsMode::Allow,
        facades: FacadeMode::Allow,
        entrypoints: FacadeMode::Allow,
        tests: TestMode::Allow,
        file_roles: Vec::new(),
        generated: vec![GeneratedSourceContract {
            root: "src/generated".into(),
            manifest: "src/generated/MANIFEST.json".into(),
            inputs: vec!["schema/**".into()],
            target: 800,
            hard: 1_000,
            reason: "compiler output".into(),
            auxiliary: Vec::new(),
        }],
        out_dir: Vec::new(),
        item_macros: Vec::new(),
        test_mirrors: Vec::new(),
        feature_worlds: Vec::new(),
        macros: zrail_core::MacroExpansionContract::default(),
        duplication: zrail_core::RustDuplicationContract::default(),
        types: Vec::new(),
        hygiene: HygieneContract {
            unsafe_code: PolicyMode::Allow,
            lint_suppressions: LintSuppressionMode::Allow,
            deny_methods: Vec::new(),
            deny_macros: Vec::new(),
            glob_imports: zrail_core::GlobImportMode::Allow,
        },
        size: None,
    };

    assert_eq!(
        budget_for(
            "src/lib.rs",
            FileClass::Facade,
            Reachability::from_kind(ReachabilityKind::Production),
            &rust
        ),
        None
    );
    let generated = budget_for(
        "src/generated/model.rs",
        FileClass::Generated,
        Reachability::from_kind(ReachabilityKind::Production),
        &rust,
    )
    .expect("generated source keeps its declared budget");
    assert_eq!(generated.target, 800);
    assert_eq!(generated.hard, 1_000);
}

#[test]
fn exact_overrides_change_only_facade_and_implementation_roles() {
    let mut rust = rust_contract();
    rust.file_roles = vec![FileRoleContract {
        path: "src/api.rs".into(),
        role: FileRole::Facade,
        reason: "public module surface".into(),
    }];

    let api = effective_file_role("src/api.rs", FileClass::Implementation, &rust);
    let test = effective_file_role("tests/api.rs", FileClass::Test, &rust);

    assert_eq!(api.effective, FileClass::Facade);
    assert_eq!(api.reason, Some("public module surface"));
    assert_eq!(test.effective, FileClass::Test);
}

#[test]
fn exact_implementation_override_reclassifies_an_entrypoint() {
    let mut rust = rust_contract();
    rust.file_roles = vec![FileRoleContract {
        path: "src/main.rs".into(),
        role: FileRole::Implementation,
        reason: "single-file binary".into(),
    }];

    let main = effective_file_role("src/main.rs", FileClass::EntryPoint, &rust);

    assert_eq!(main.inferred, FileClass::EntryPoint);
    assert_eq!(main.effective, FileClass::Implementation);
    assert_eq!(main.reason, Some("single-file binary"));
}

fn rust_contract() -> RustSourceContract {
    RustSourceContract {
        module_docs: ModuleDocsMode::Allow,
        facades: FacadeMode::Allow,
        entrypoints: FacadeMode::Allow,
        tests: TestMode::Allow,
        file_roles: Vec::new(),
        generated: Vec::new(),
        out_dir: Vec::new(),
        item_macros: Vec::new(),
        test_mirrors: Vec::new(),
        feature_worlds: Vec::new(),
        macros: zrail_core::MacroExpansionContract::default(),
        duplication: zrail_core::RustDuplicationContract::default(),
        types: Vec::new(),
        hygiene: HygieneContract {
            unsafe_code: PolicyMode::Allow,
            lint_suppressions: LintSuppressionMode::Allow,
            deny_methods: Vec::new(),
            deny_macros: Vec::new(),
            glob_imports: zrail_core::GlobImportMode::Allow,
        },
        size: None,
    }
}
