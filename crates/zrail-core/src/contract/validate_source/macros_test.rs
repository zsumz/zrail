//! Binding-clean expansion authority requires exact, source-bound review.

use crate::{CrateRootSource, MacroExpansionAllow, MacroExpansionBindings, MacroExpansionMode};

use crate::contract::{validate::validate_contract, validate_fixture_test::minimal_contract};

#[test]
fn no_binding_attestation_requires_provenance() {
    let mut contract = minimal_contract();
    contract.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    contract
        .source
        .rust
        .macros
        .allow
        .push(no_binding_allowance(None));

    let errors = validate_contract(&contract).expect_err("unbound no-binding authority must fail");
    assert!(
        errors
            .to_string()
            .contains("requires source or definition provenance")
    );
}

#[test]
fn source_bound_no_binding_attestation_is_valid() {
    let mut contract = minimal_contract();
    contract.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    contract
        .source
        .rust
        .macros
        .allow
        .push(no_binding_allowance(Some(CrateRootSource::Registry {
            registry: None,
            index: None,
            requirement: "=1.0.0".into(),
        })));

    validate_contract(&contract).expect("source-bound no-binding authority is valid");
}

#[test]
fn git_revision_bound_no_binding_attestation_is_valid() {
    for revision in [GIT_SHA1, GIT_SHA256] {
        let mut contract = minimal_contract();
        contract.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
        contract
            .source
            .rust
            .macros
            .allow
            .push(no_binding_allowance(Some(git_source(
                None,
                None,
                Some(revision),
            ))));

        validate_contract(&contract).expect("revision-bound no-binding authority is valid");
    }
}

#[test]
fn no_binding_attestation_rejects_mutable_external_sources() {
    let sources = [
        ("legacy", CrateRootSource::Legacy),
        (
            "registry range",
            CrateRootSource::Registry {
                registry: None,
                index: None,
                requirement: "1.0.0".into(),
            },
        ),
        (
            "partial registry equality",
            CrateRootSource::Registry {
                registry: None,
                index: None,
                requirement: "=1.0".into(),
            },
        ),
        (
            "registry wildcard",
            CrateRootSource::Registry {
                registry: None,
                index: None,
                requirement: "=1.0.*".into(),
            },
        ),
        ("Git branch", git_source(Some("main"), None, None)),
        ("Git tag", git_source(None, Some("v1"), None)),
        ("unpinned Git", git_source(None, None, None)),
        ("short Git rev", git_source(None, None, Some("abc123"))),
        ("mutable Git revspec", git_source(None, None, Some("main"))),
    ];

    for (case, source) in sources {
        let mut contract = minimal_contract();
        contract.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
        contract
            .source
            .rust
            .macros
            .allow
            .push(no_binding_allowance(Some(source)));

        let Err(errors) = validate_contract(&contract) else {
            panic!("{case} must not grant binding-clean authority");
        };
        assert!(
            errors
                .to_string()
                .contains("requires immutable source provenance"),
            "{case}: {errors}"
        );
    }
}

#[test]
fn no_binding_attestation_rejects_conservative_name_binding() {
    let mut contract = minimal_contract();
    contract.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    let mut allowance = no_binding_allowance(Some(CrateRootSource::Registry {
        registry: None,
        index: None,
        requirement: "=1.0.0".into(),
    }));
    allowance.binding = crate::MacroBindingMode::Conservative;
    contract.source.rust.macros.allow.push(allowance);

    let errors =
        validate_contract(&contract).expect_err("conservative no-binding authority must fail");
    assert!(errors.to_string().contains("requires exact binding"));
}

#[test]
fn no_duplication_attestation_requires_exact_provenance() {
    let mut contract = minimal_contract();
    contract.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    let mut allowance = no_binding_allowance(None);
    allowance.bindings = MacroExpansionBindings::Opaque;
    allowance.duplication_effect = crate::MacroDuplicationEffect::None;
    contract.source.rust.macros.allow.push(allowance);

    let errors = validate_contract(&contract).expect_err("unproven no-duplication claim must fail");
    assert!(
        errors
            .to_string()
            .contains("duplication_effect = \"none\" requires source or definition provenance")
    );
}

#[test]
fn exact_local_no_duplication_attestation_is_valid() {
    let mut contract = minimal_contract();
    contract.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    let mut allowance = no_binding_allowance(None);
    allowance.bindings = MacroExpansionBindings::Opaque;
    allowance.duplication_effect = crate::MacroDuplicationEffect::None;
    allowance.definition = Some("src/macros.rs".into());
    contract.source.rust.macros.allow.push(allowance);

    validate_contract(&contract).expect("exact local no-duplication claim is valid");
}

#[test]
fn no_source_operations_attestation_requires_exact_provenance() {
    let mut contract = minimal_contract();
    contract.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    let mut allowance = no_binding_allowance(None);
    allowance.bindings = MacroExpansionBindings::Opaque;
    allowance.source_operations = crate::MacroSourceOperations::None;
    contract.source.rust.macros.allow.push(allowance);

    let errors =
        validate_contract(&contract).expect_err("unproven no-source-operations claim must fail");
    assert!(
        errors
            .to_string()
            .contains("source_operations = \"none\" requires source or definition provenance")
    );
}

#[test]
fn exact_local_no_source_operations_attestation_is_valid() {
    let mut contract = minimal_contract();
    contract.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    let mut allowance = no_binding_allowance(None);
    allowance.bindings = MacroExpansionBindings::Opaque;
    allowance.source_operations = crate::MacroSourceOperations::None;
    allowance.definition = Some("src/macros.rs".into());
    contract.source.rust.macros.allow.push(allowance);

    validate_contract(&contract).expect("exact local no-source-operations claim is valid");
}

fn no_binding_allowance(source: Option<CrateRootSource>) -> MacroExpansionAllow {
    MacroExpansionAllow {
        name: "derive::Model".into(),
        inputs: crate::MacroInputMode::Inspect,
        binding: crate::MacroBindingMode::Exact,
        bindings: MacroExpansionBindings::None,
        async_syntax: crate::MacroAsyncSyntax::Opaque,
        duplication_effect: crate::MacroDuplicationEffect::Opaque,
        source_operations: crate::MacroSourceOperations::Opaque,
        definition: None,
        source,
        reason: "Reviewed output preserves the ordinary namespace exactly.".into(),
    }
}

fn git_source(branch: Option<&str>, tag: Option<&str>, rev: Option<&str>) -> CrateRootSource {
    CrateRootSource::Git {
        repository: "https://example.invalid/macro".into(),
        branch: branch.map(str::to_owned),
        tag: tag.map(str::to_owned),
        rev: rev.map(str::to_owned),
        requirement: None,
    }
}

const GIT_SHA1: &str = "0123456789abcdef0123456789abcdef01234567";
const GIT_SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
