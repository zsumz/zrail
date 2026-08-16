//! Optional handwritten budgets do not disable explicit generated-source budgets.

use zrail_core::{
    FacadeMode, GeneratedSourceContract, HygieneContract, LintSuppressionMode, ModuleDocsMode,
    PolicyMode, RustSourceContract, TestMode,
};

use crate::{inventory::FileClass, source::Reachability};

use super::budget_for;

#[test]
fn generated_budget_remains_enforced_without_a_handwritten_size_policy() {
    let rust = RustSourceContract {
        module_docs: ModuleDocsMode::Allow,
        facades: FacadeMode::Allow,
        entrypoints: FacadeMode::Allow,
        tests: TestMode::Allow,
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
        macros: zrail_core::MacroExpansionContract::default(),
        hygiene: HygieneContract {
            unsafe_code: PolicyMode::Allow,
            lint_suppressions: LintSuppressionMode::Allow,
            deny_methods: Vec::new(),
            deny_macros: Vec::new(),
        },
        size: None,
    };

    assert_eq!(
        budget_for(
            "src/lib.rs",
            FileClass::Facade,
            Reachability::Production,
            &rust
        ),
        None
    );
    let generated = budget_for(
        "src/generated/model.rs",
        FileClass::Generated,
        Reachability::Production,
        &rust,
    )
    .expect("generated source keeps its declared budget");
    assert_eq!(generated.target, 800);
    assert_eq!(generated.hard, 1_000);
}
