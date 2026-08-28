//! Generic call boundaries are relevant only to matching call-owner authority.

use zrail_core::{OwnerContract, OwnerKind, PolicyReachability};

use crate::{
    inventory::FileClass,
    source::{
        CallResolutionFact, CallResolutionKind, Reachability, ReachabilityKind, RustFileFacts,
        SourceSyntax, SyntaxGuard,
    },
};

use super::owner_relies_on;

#[test]
fn matching_call_owner_relies_on_generic_associated_identity() {
    assert!(owner_relies_on(
        &owner("crate::Factory::ready"),
        &file(),
        &boundary("Choice::ready"),
    ));
}

#[test]
fn unrelated_call_owner_does_not_make_generic_rust_incomplete() {
    assert!(!owner_relies_on(
        &owner("std::process::Command::new"),
        &file(),
        &boundary("D::Error::custom"),
    ));
}

fn owner(selector: &str) -> OwnerContract {
    OwnerContract {
        name: "call-owner".into(),
        kind: OwnerKind::Call,
        reachability: PolicyReachability::All,
        within: vec!["src/**".into()],
        selector: selector.into(),
        mutating_methods: Vec::new(),
        allow: vec!["src/owner.rs".into()],
        reason: "one exact call boundary".into(),
    }
}

fn boundary(written: &str) -> CallResolutionFact {
    CallResolutionFact {
        written: written.into(),
        span: zrail_core::SourceSpan {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 2,
        },
        guard: SyntaxGuard::Ordinary,
        kind: CallResolutionKind::GenericAssociatedItem,
    }
}

fn file() -> RustFileFacts {
    RustFileFacts {
        relative: "src/lib.rs".into(),
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
        prelude_directives: Vec::new(),
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
