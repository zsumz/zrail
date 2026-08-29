//! Generic call boundaries are relevant only to matching call-owner authority.

use zrail_core::{OwnerContract, OwnerKind, PolicyReachability};

use crate::{
    inventory::FileClass,
    source::{
        AssociatedCandidateKind, AssociatedOccurrenceKind, CallResolutionFact, CallResolutionKind,
        GenericAssociatedCandidate, ProjectionIdentity, Reachability, ReachabilityKind,
        RustFileFacts, SourceSyntax, SyntaxGuard,
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

#[test]
fn capability_owner_applies_to_generic_function_reference() {
    assert!(owner_relies_on(
        &capability_owner("crate::Factory::ready"),
        &file(),
        &boundary_for(
            "Choice::ready",
            AssociatedOccurrenceKind::ValueReference,
            &["Factory::ready"],
        ),
    ));
}

#[test]
fn unrelated_same_named_owner_is_not_implicated() {
    assert!(!owner_relies_on(
        &owner("crate::Widget::ready"),
        &file(),
        &boundary("Choice::ready"),
    ));
}

#[test]
fn multiple_candidates_fail_closed_only_for_matching_authority() {
    let boundary = boundary_for(
        "Choice::ready",
        AssociatedOccurrenceKind::DirectCall,
        &["Factory::ready", "Alternative::ready"],
    );
    assert!(owner_relies_on(
        &owner("crate::Alternative::ready"),
        &file(),
        &boundary,
    ));
    assert!(!owner_relies_on(
        &owner("crate::Unrelated::ready"),
        &file(),
        &boundary,
    ));
}

#[test]
fn same_external_root_different_item_is_not_relevant() {
    let boundary = incomplete_boundary(
        "Choice::ready",
        "dependency::Derived::ready",
        crate::source::ProviderAuthority::ExternalRoot("dependency".into()),
    );

    assert!(!owner_relies_on(
        &owner("dependency::Other::shutdown"),
        &file(),
        &boundary,
    ));
}

#[test]
fn same_local_authority_different_item_is_not_relevant() {
    let boundary = incomplete_boundary(
        "Choice::ready",
        "Factory::ready",
        crate::source::ProviderAuthority::LocalCrate,
    );

    assert!(!owner_relies_on(
        &owner("crate::Other::shutdown"),
        &file(),
        &boundary,
    ));
}

#[test]
fn unknown_provider_same_item_remains_fail_closed() {
    let boundary = incomplete_boundary(
        "Choice::ready",
        "Derived::ready",
        crate::source::ProviderAuthority::Unknown,
    );

    assert!(owner_relies_on(
        &owner("crate::Factory::ready"),
        &file(),
        &boundary,
    ));
}

#[test]
fn trait_prefix_owner_remains_conservative() {
    let boundary = incomplete_boundary(
        "Choice::ready",
        "dependency::Parent::ready",
        crate::source::ProviderAuthority::ExternalRoot("dependency".into()),
    );

    assert!(owner_relies_on(
        &owner("dependency::Derived"),
        &file(),
        &boundary,
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

fn capability_owner(selector: &str) -> OwnerContract {
    OwnerContract {
        kind: OwnerKind::Capability,
        ..owner(selector)
    }
}

fn boundary(written: &str) -> CallResolutionFact {
    boundary_for(
        written,
        AssociatedOccurrenceKind::DirectCall,
        &["Factory::ready"],
    )
}

fn boundary_for(
    written: &str,
    occurrence: AssociatedOccurrenceKind,
    candidates: &[&str],
) -> CallResolutionFact {
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
        associated_candidates: candidates
            .iter()
            .map(|candidate| GenericAssociatedCandidate {
                name: (*candidate).into(),
                canonical: Vec::new(),
                quality: zrail_core::AnalysisQuality::Unresolved,
                projection: ProjectionIdentity::default(),
                kind: AssociatedCandidateKind::TraitProvider,
                provider_complete: true,
                provider_authorities: [crate::source::ProviderAuthority::Unknown].into(),
            })
            .collect(),
        occurrence: Some(occurrence),
    }
}

fn incomplete_boundary(
    written: &str,
    candidate: &str,
    authority: crate::source::ProviderAuthority,
) -> CallResolutionFact {
    let mut boundary = boundary_for(written, AssociatedOccurrenceKind::DirectCall, &[candidate]);
    boundary.associated_candidates[0].provider_complete = false;
    boundary.associated_candidates[0].provider_authorities = [authority].into();
    boundary
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
        trait_declarations: Vec::new(),
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
