//! Feature-disabled syntax disappears while target-dependent syntax remains conservative.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use zrail_core::{
    FacadeMode, GlobImportMode, HygieneContract, LintSuppressionMode, ModuleDocsMode, PolicyMode,
    RustSourceContract, TestMode,
};

use super::retain_active_facts;
use crate::{
    inventory::{FileClass, RepositoryInventory, RustSourceFile},
    source::{CompilationDomain, CompilationMode, index_rust_source},
};

#[test]
fn exact_worlds_prune_inactive_features_and_retain_target_uncertainty() {
    let source = concat!(
        "#[cfg(feature = \"enabled\")] unsafe fn enabled() {}\n",
        "#[cfg(feature = \"disabled\")] unsafe fn disabled() {}\n",
        "#[cfg(unix)] unsafe fn target_dependent() {}\n",
    );
    let mut index = index(source);
    let domains = BTreeMap::from([(
        "src/lib.rs".into(),
        BTreeSet::from([CompilationDomain {
            package: "app".into(),
            edition: "2024".into(),
            target: "app".into(),
            mode: CompilationMode::Library,
            feature_world: Some("enabled".into()),
            active_features: BTreeSet::from(["enabled".into()]),
        }]),
    )]);

    retain_active_facts(&mut index, &domains, true);

    let guards = index.files[0]
        .unsafe_constructs
        .iter()
        .map(|fact| fact.guard.canonical_name())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        guards,
        BTreeSet::from(["cfg:feature=\"enabled\"".into(), "cfg:opaque(unix)".into(),])
    );
}

#[test]
fn cfg_attr_derive_follows_the_enclosing_guard_in_each_world() {
    let source = concat!(
        "#[cfg(feature = \"outer\")]\n",
        "#[cfg_attr(feature = \"duplicate\", derive(Clone))]\n",
        "struct Ticket;\n",
    );

    let inactive = retained(source, &["outer"]);
    assert!(
        inactive.files[0].type_policy.declarations[0]
            .derives
            .is_empty()
    );
    assert!(inactive.files[0].macro_expansions.is_empty());

    let active = retained(source, &["duplicate", "outer"]);
    let derive = &active.files[0].type_policy.declarations[0].derives[0];
    assert_eq!(
        derive.guard.canonical_name(),
        "cfg:all(feature=\"duplicate\",feature=\"outer\")"
    );
    assert_eq!(active.files[0].macro_expansions.len(), 1);
    assert_eq!(
        active.files[0].macro_expansions[0].guard.canonical_name(),
        derive.guard.canonical_name()
    );
}

#[test]
fn custom_and_nested_cfg_attrs_are_active_only_in_their_exact_worlds() {
    let source = concat!(
        "#[cfg(feature = \"outer\")]\n",
        "#[cfg_attr(feature = \"custom\", reviewed::attribute)]\n",
        "#[cfg_attr(feature = \"first\", ",
        "cfg_attr(feature = \"second\", reviewed::nested))]\n",
        "struct Ticket;\n",
    );

    let inactive = retained(source, &["first", "outer"]);
    assert!(inactive.files[0].macro_expansions.is_empty());
    assert!(inactive.files[0].opaque_binding_macros.is_empty());

    let active = retained(source, &["custom", "first", "outer", "second"]);
    assert_eq!(active.files[0].macro_expansions.len(), 2);
    assert_eq!(active.files[0].opaque_binding_macros.len(), 2);
    let guards = active.files[0]
        .macro_expansions
        .iter()
        .map(|fact| fact.guard.canonical_name())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        guards,
        BTreeSet::from([
            "cfg:all(feature=\"custom\",feature=\"outer\")".into(),
            "cfg:all(feature=\"first\",feature=\"outer\",feature=\"second\")".into(),
        ])
    );
}

#[test]
fn non_macro_attribute_policy_facts_follow_cfg_attr_worlds() {
    let source = concat!(
        "#[cfg(feature = \"outer\")]\n",
        "#[cfg_attr(feature = \"lint\", allow(dead_code))]\n",
        "#[cfg_attr(feature = \"symbol\", unsafe(no_mangle))]\n",
        "fn guarded() {}\n",
    );

    let inactive = retained(source, &["outer"]);
    assert!(inactive.files[0].lint_suppressions.is_empty());
    assert!(inactive.files[0].unsafe_constructs.is_empty());

    let active = retained(source, &["lint", "outer", "symbol"]);
    assert_eq!(active.files[0].lint_suppressions.len(), 1);
    assert_eq!(active.files[0].unsafe_constructs.len(), 1);
    assert_eq!(
        active.files[0].lint_suppressions[0].guard.canonical_name(),
        "cfg:all(feature=\"lint\",feature=\"outer\")"
    );
    assert_eq!(
        active.files[0].unsafe_constructs[0].guard.canonical_name(),
        "cfg:all(feature=\"outer\",feature=\"symbol\")"
    );
}

#[test]
fn legacy_conditional_mode_does_not_prune_physical_facts() {
    let mut index = index(concat!(
        "#[cfg(feature = \"disabled\")] unsafe fn disabled() {}\n",
        "#[cfg_attr(feature = \"disabled\", unsafe(no_mangle))] fn symbol() {}\n",
    ));

    retain_active_facts(&mut index, &BTreeMap::new(), false);

    assert_eq!(index.files[0].unsafe_constructs.len(), 2);
}

fn retained(source: &str, features: &[&str]) -> crate::source::SourceIndex {
    let mut index = index(source);
    let domains = BTreeMap::from([(
        "src/lib.rs".into(),
        BTreeSet::from([CompilationDomain {
            package: "app".into(),
            edition: "2024".into(),
            target: "app".into(),
            mode: CompilationMode::Library,
            feature_world: Some("selected".into()),
            active_features: features.iter().map(|feature| (*feature).into()).collect(),
        }]),
    )]);
    retain_active_facts(&mut index, &domains, true);
    index
}

fn index(source: &str) -> crate::source::SourceIndex {
    let inventory = RepositoryInventory {
        root: PathBuf::from("."),
        entries: Vec::new(),
        rust_files: vec![RustSourceFile {
            relative: "src/lib.rs".into(),
            class: FileClass::Facade,
            source: source.into(),
            lines: source.lines().count(),
        }],
        manifest_paths: Vec::new(),
    };
    index_rust_source(&inventory, &rust_contract())
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
            glob_imports: GlobImportMode::Allow,
        },
        size: None,
    }
}
