//! Rust type aliases retain the underlying ADT at every operation owner boundary.

use zrail_core::{AnalysisQuality, OwnerContract, OwnerKind, PolicyReachability};

use super::*;
use crate::source::{SourceIndex, SourceOperationKind};

#[test]
fn local_type_alias_construction_reaches_underlying_owner() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"struct State { epoch: usize }
type Alias = State;
fn mint() -> Alias { Alias { epoch: 0 } }",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_exact_owner(
        &index,
        "src/lib.rs",
        OwnerKind::TypeConstruction,
        "crate::State",
    );
}

#[test]
fn local_type_alias_update_reaches_underlying_field_owner() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"struct State { public: usize, secret: String }
type Alias = State;
fn trespass(previous: State) -> Alias {
    Alias { public: 10, ..previous }
}",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_exact_owner(
        &index,
        "src/lib.rs",
        OwnerKind::FieldRead,
        "crate::State::secret",
    );
    assert!(
        matching(
            &index,
            "src/lib.rs",
            OwnerKind::FieldRead,
            "crate::State::public"
        )
        .is_empty()
    );
}

#[test]
fn type_alias_chain_is_canonical() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"struct Envelope<T> { value: T }
type Alias<T> = Envelope<T>;
type Second<T> = Alias<T>;
fn mint() -> Second<u8> { Second::<u8> { value: 1 } }",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_exact_owner(
        &index,
        "src/lib.rs",
        OwnerKind::TypeConstruction,
        "crate::Envelope",
    );
}

#[test]
fn cfg_partitioned_aliases_are_domain_exact() {
    let mut left = domain();
    left.feature_world = Some("left".into());
    left.active_features.insert("left".into());
    let mut right = domain();
    right.feature_world = Some("right".into());
    let mut index = index([parsed_file(
        "src/lib.rs",
        r#"struct Left { epoch: usize }
struct Right { epoch: usize }
#[cfg(feature = "left")]
type Alias = Left;
#[cfg(not(feature = "left"))]
type Alias = Right;
fn mint() -> Alias { Alias { epoch: 0 } }"#,
    )]);
    let findings = canonicalize_operation_worlds(&mut index, &[left, right], &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    for selector in ["crate::Left", "crate::Right"] {
        let operations = matching(&index, "src/lib.rs", OwnerKind::TypeConstruction, selector);
        assert_eq!(operations.len(), 1, "owner {selector}: {operations:#?}");
        assert_eq!(
            operations[0].identity.quality,
            AnalysisQuality::Conservative
        );
        assert_eq!(operations[0].identity.canonical, ["Left", "Right"]);
    }
}

#[test]
fn cross_module_alias_resolves_to_underlying_adt() {
    let root = parsed_file("src/lib.rs", "mod state;\nmod user;");
    let state = parsed_file(
        "src/state.rs",
        "pub struct State { pub epoch: usize, pub secret: String }\npub type Alias = State;",
    );
    let user = parsed_file(
        "src/user.rs",
        r"use super::state::Alias;
fn update(previous: Alias) -> Alias { Alias { epoch: 0, ..previous } }",
    );
    let compilation = domain();
    let modules = [("state", "src/state.rs"), ("user", "src/user.rs")]
        .into_iter()
        .map(|(name, child)| {
            module_edge(
                "src/lib.rs",
                name,
                child,
                module(&root.modules, name),
                &compilation,
            )
        })
        .collect::<Vec<_>>();
    let mut index = index([root, state, user]);
    let findings = canonicalize_operations(&mut index, &compilation, &modules);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_exact_owner(
        &index,
        "src/user.rs",
        OwnerKind::TypeConstruction,
        "crate::state::State",
    );
    assert_exact_owner(
        &index,
        "src/user.rs",
        OwnerKind::FieldRead,
        "crate::state::State::secret",
    );
}

#[test]
fn relative_alias_target_is_module_canonical() {
    let root = parsed_file("src/lib.rs", "mod state;\nmod user;");
    let state = parsed_file("src/state.rs", "pub struct State { pub epoch: usize }");
    let user = parsed_file(
        "src/user.rs",
        r"type Alias = super::state::State;
fn mint() -> Alias { Alias { epoch: 0 } }",
    );
    let compilation = domain();
    let modules = [("state", "src/state.rs"), ("user", "src/user.rs")]
        .into_iter()
        .map(|(name, child)| {
            module_edge(
                "src/lib.rs",
                name,
                child,
                module(&root.modules, name),
                &compilation,
            )
        })
        .collect::<Vec<_>>();
    let mut index = index([root, state, user]);
    let findings = canonicalize_operations(&mut index, &compilation, &modules);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_exact_owner(
        &index,
        "src/user.rs",
        OwnerKind::TypeConstruction,
        "crate::state::State",
    );
}

#[test]
fn type_alias_cycle_fails_closed() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"type First = Second;
type Second = First;
fn mint() -> First { First { epoch: 0 } }",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(
        findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002")
    );
    let construction = index.files[0]
        .operations
        .iter()
        .find(|operation| operation.kind == SourceOperationKind::TypeConstruction)
        .expect("cycle construction receipt");
    assert_eq!(construction.identity.quality, AnalysisQuality::Unresolved);
}

#[test]
fn opaque_type_alias_fails_closed() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"type Alias = (usize,);
fn mint() -> Alias { Alias { epoch: 0 } }",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(
        findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002")
    );
    let construction = index.files[0]
        .operations
        .iter()
        .find(|operation| operation.kind == SourceOperationKind::TypeConstruction)
        .expect("opaque construction receipt");
    assert_eq!(construction.identity.quality, AnalysisQuality::Unresolved);
}

#[test]
fn external_update_retains_opaque_field_receipt() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        "fn update(previous: External) -> External { External { value: 1, ..previous } }",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert!(index.files[0].operations.iter().any(|operation| {
        operation.kind == SourceOperationKind::FieldRead
            && operation.identity.name == "External::*"
            && operation.identity.quality == AnalysisQuality::Unresolved
    }));
}

fn index(files: impl IntoIterator<Item = crate::source::RustFileFacts>) -> SourceIndex {
    SourceIndex {
        files: files.into_iter().collect(),
        findings: Vec::new(),
        analysis_metrics: crate::source::SourceAnalysisMetrics::default(),
    }
}

fn assert_exact_owner(index: &SourceIndex, file: &str, kind: OwnerKind, selector: &str) {
    let operations = matching(index, file, kind, selector);
    assert_eq!(operations.len(), 1, "owner {selector}: {operations:#?}");
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Exact);
}

fn matching(
    index: &SourceIndex,
    file: &str,
    kind: OwnerKind,
    selector: &str,
) -> Vec<crate::source::SourceOperationFact> {
    let owner = OwnerContract {
        name: "operation-owner".into(),
        kind,
        reachability: PolicyReachability::All,
        within: vec!["src/**".into()],
        selector: selector.into(),
        mutating_methods: Vec::new(),
        allow: vec!["src/owner.rs".into()],
        reason: "operation stays centralized".into(),
    };
    let file = index
        .files
        .iter()
        .find(|candidate| candidate.relative == file)
        .expect("operation file");
    crate::rules::matching_operation_owner_operations(&owner, file)
        .cloned()
        .collect()
}
