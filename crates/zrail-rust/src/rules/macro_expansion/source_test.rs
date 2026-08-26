//! Cargo.lock macro authority matches exactly one resolved package identity.

use std::fs;

use zrail_core::{
    AnalysisQuality, CrateRootSource, MacroBindingMode, MacroExpansionAllow,
    MacroExpansionBindings, MacroInputMode,
};

use crate::{
    cargo::{DependencySource, ResolvedCargoGraph},
    source::{
        FactNamespace, MacroCandidate, MacroDerivation, MacroOrigin, ObservedFact, SyntaxGuard,
    },
};

use super::failures;

#[test]
fn cargo_lock_source_authorizes_only_the_exact_resolved_occurrence() {
    let root = std::env::temp_dir().join(format!("zrail-macro-lock-source-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir(&root).expect("create fixture");
    fs::write(
        root.join("Cargo.lock"),
        format!(
            "version = 3\n\n[[package]]\nname = \"derive-impl\"\nversion = \"1.2.3\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"{}\"\n",
            "1".repeat(64)
        ),
    )
    .expect("write Cargo.lock");
    let graph = ResolvedCargoGraph::load(&root, &[])
        .expect("parse graph")
        .expect("present graph");
    let allowance = allowance();
    let exact = candidate("1");

    assert!(failures(&exact, &allowance, Some(&graph)).is_empty());

    let wrong_version = candidate("2");
    assert!(!failures(&wrong_version, &allowance, Some(&graph)).is_empty());
    assert!(!failures(&exact, &allowance, None).is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn cargo_lock_source_rejects_same_source_multiversion_ambiguity() {
    let root =
        std::env::temp_dir().join(format!("zrail-macro-lock-ambiguity-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir(&root).expect("create fixture");
    fs::write(root.join("Cargo.lock"), AMBIGUOUS_LOCK).expect("write Cargo.lock");
    let graph = ResolvedCargoGraph::load(&root, &[])
        .expect("parse graph")
        .expect("present graph");

    assert!(!failures(&candidate("1"), &allowance(), Some(&graph)).is_empty());
    let _ = fs::remove_dir_all(root);
}

fn allowance() -> MacroExpansionAllow {
    MacroExpansionAllow {
        name: "derive".into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: MacroExpansionBindings::None,
        async_syntax: zrail_core::MacroAsyncSyntax::Opaque,
        duplication_effect: zrail_core::MacroDuplicationEffect::Opaque,
        definition: None,
        source: Some(CrateRootSource::CargoLock {
            package: "derive-impl".into(),
            version: Some("1.2.3".into()),
            source: None,
        }),
        reason: "exact reviewed implementation".into(),
    }
}

fn candidate(requirement: &str) -> MacroCandidate {
    MacroCandidate {
        observation: ObservedFact {
            name: "derive".into(),
            written: None,
            canonical: vec!["derive".into()],
            span: None,
            quality: AnalysisQuality::Exact,
            guard: SyntaxGuard::Ordinary,
            lexical_scope: Vec::new(),
            namespace: FactNamespace::Unknown,
        },
        origins: vec![MacroOrigin::External {
            package: "derive-impl".into(),
            source: DependencySource::Registry {
                registry: None,
                index: None,
                requirement: requirement.into(),
            },
        }],
        derivation: MacroDerivation::DependencyRoot,
        written_alias: false,
        definition: None,
        definition_name: None,
        definition_sha256: None,
    }
}

const AMBIGUOUS_LOCK: &str = r#"version = 3

[[package]]
name = "derive-impl"
version = "1.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1111111111111111111111111111111111111111111111111111111111111111"

[[package]]
name = "derive-impl"
version = "1.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2222222222222222222222222222222222222222222222222222222222222222"
"#;
