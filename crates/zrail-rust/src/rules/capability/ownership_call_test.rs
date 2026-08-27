//! An authorized owner cannot turn unresolved macro-opaque call identity into authority.

use zrail_core::{AnalysisQuality, FindingSink, OwnerContract, OwnerKind, PolicyReachability};

use super::check;
use crate::{
    inventory::FileClass,
    source::{
        FactNamespace, ObservedFact, Reachability, ReachabilityKind, RustFileFacts, SourceSyntax,
        SyntaxGuard,
    },
};

#[test]
fn unresolved_direct_call_fails_closed_inside_its_allowed_owner() {
    let owner = OwnerContract {
        name: "danger-call".into(),
        kind: OwnerKind::Call,
        reachability: PolicyReachability::All,
        within: vec!["src/**".into()],
        selector: "danger".into(),
        mutating_methods: Vec::new(),
        allow: vec!["src/owner.rs".into()],
        reason: "one exact invocation owner".into(),
    };
    let mut file = empty_file();
    file.calls.push(ObservedFact {
        name: "danger".into(),
        written: Some("danger".into()),
        canonical: Vec::new(),
        span: None,
        quality: AnalysisQuality::Unresolved,
        guard: SyntaxGuard::Ordinary,
        lexical_scope: Vec::new(),
        namespace: FactNamespace::Value,
    });
    let mut findings = FindingSink::default();

    assert!(check(&owner, &file, &mut findings));
    let finding = findings.iter().next().expect("fail-closed owner finding");
    assert_eq!(finding.id, "OWN-005");
    assert_eq!(finding.analysis, AnalysisQuality::Unresolved);
}

fn empty_file() -> RustFileFacts {
    RustFileFacts {
        relative: "src/owner.rs".into(),
        packages: vec!["app".into()],
        class: FileClass::Implementation,
        reachability: Reachability::from_kind(ReachabilityKind::Production),
        syntax: SourceSyntax::Items,
        lines: 1,
        module_docs: true,
        paths: Vec::new(),
        calls: Vec::new(),
        call_resolutions: Vec::new(),
        methods: Vec::new(),
        operations: Vec::new(),
        macros: Vec::new(),
        macro_imports: Vec::new(),
        macro_expansions: Vec::new(),
        opaque_macro_inputs: Vec::new(),
        macro_definitions: Vec::new(),
        import_bindings: Vec::new(),
        associated_items: Vec::new(),
        glob_imports: Vec::new(),
        inline_module_scopes: Vec::new(),
        compile_effects: Vec::new(),
        lint_suppressions: Vec::new(),
        unsafe_constructs: Vec::new(),
        async_syntax: Vec::new(),
        type_policy: crate::source::TypePolicyFacts::default(),
        tests: Vec::new(),
        modules: Vec::new(),
        includes: Vec::new(),
        item_macros: Vec::new(),
        opaque_binding_macros: Vec::new(),
        facade_implementation: Vec::new(),
    }
}
