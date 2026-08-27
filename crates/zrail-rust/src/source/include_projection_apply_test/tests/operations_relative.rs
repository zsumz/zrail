//! Relative construction spellings reach the same owner as their canonical type.

use zrail_core::{AnalysisQuality, OwnerContract, OwnerKind, PolicyReachability};

use super::*;
use crate::source::{SourceIndex, SourceOperationKind};

#[test]
fn direct_super_path_reaches_type_construction_owner() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"pub struct State { epoch: usize }
mod rogue {
    fn mint() -> super::State { super::State { epoch: 0 } }
}",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_exact_owner(&index, "src/lib.rs", "crate::State");
}

#[test]
fn direct_self_path_reaches_type_construction_owner() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"mod rogue {
    pub struct State { epoch: usize }
    fn mint() -> self::State { self::State { epoch: 0 } }
}",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_exact_owner(&index, "src/lib.rs", "crate::rogue::State");
}

#[test]
fn repeated_super_path_reaches_type_construction_owner() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"pub struct State { epoch: usize }
mod outer {
    mod rogue {
        fn mint() -> super::super::State {
            super::super::State { epoch: 0 }
        }
    }
}",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_exact_owner(&index, "src/lib.rs", "crate::State");
}

#[test]
fn relative_import_alias_reaches_type_construction_owner() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"pub struct State { epoch: usize }
mod rogue {
    use super::State as Alias;
    fn mint() -> Alias { Alias { epoch: 0 } }
}",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_exact_owner(&index, "src/lib.rs", "crate::State");
}

#[test]
fn external_module_relative_path_is_canonical() {
    let root = parsed_file(
        "src/lib.rs",
        "pub struct State { epoch: usize }\nmod rogue;",
    );
    let child = parsed_file(
        "src/rogue.rs",
        "fn mint() -> super::State { super::State { epoch: 0 } }",
    );
    let compilation = domain();
    let modules = vec![module_edge(
        "src/lib.rs",
        "rogue",
        "src/rogue.rs",
        module(&root.modules, "rogue"),
        &compilation,
    )];
    let mut index = index([root, child]);
    let findings = canonicalize_operations(&mut index, &compilation, &modules);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_exact_owner(&index, "src/rogue.rs", "crate::State");
}

#[test]
fn ambiguous_module_attachment_fails_closed() {
    let root = parsed_file(
        "src/lib.rs",
        "#[path = \"shared.rs\"] mod left;\n#[path = \"shared.rs\"] mod right;",
    );
    let shared = parsed_file(
        "src/shared.rs",
        r"pub struct Local { epoch: usize }
fn mint() -> self::Local { self::Local { epoch: 0 } }",
    );
    let compilation = domain();
    let modules = ["left", "right"]
        .into_iter()
        .map(|name| {
            module_edge(
                "src/lib.rs",
                name,
                "src/shared.rs",
                module(&root.modules, name),
                &compilation,
            )
        })
        .collect::<Vec<_>>();
    let mut index = index([root, shared]);
    let findings = canonicalize_operations(&mut index, &compilation, &modules);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    let operation = matching(&index, "src/shared.rs", "crate::left::Local")
        .into_iter()
        .next()
        .expect("left attachment reaches its owner");
    assert_eq!(operation.identity.quality, AnalysisQuality::Conservative);
    assert_eq!(
        operation.identity.canonical,
        ["left::Local", "right::Local"]
    );
}

fn index(files: impl IntoIterator<Item = crate::source::RustFileFacts>) -> SourceIndex {
    SourceIndex {
        files: files.into_iter().collect(),
        findings: Vec::new(),
        analysis_metrics: crate::source::SourceAnalysisMetrics::default(),
    }
}

fn assert_exact_owner(index: &SourceIndex, file: &str, selector: &str) {
    let operations = matching(index, file, selector);
    assert_eq!(operations.len(), 1, "owner {selector}: {operations:#?}");
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Exact);
}

fn matching(
    index: &SourceIndex,
    file: &str,
    selector: &str,
) -> Vec<crate::source::SourceOperationFact> {
    let owner = OwnerContract {
        name: "construction-owner".into(),
        kind: OwnerKind::TypeConstruction,
        reachability: PolicyReachability::All,
        within: vec!["src/**".into()],
        selector: selector.into(),
        mutating_methods: Vec::new(),
        allow: vec!["src/owner.rs".into()],
        reason: "construction stays centralized".into(),
    };
    let file = index
        .files
        .iter()
        .find(|candidate| candidate.relative == file)
        .expect("operation file");
    crate::rules::matching_operation_owner_operations(&owner, file)
        .filter(|operation| operation.kind == SourceOperationKind::TypeConstruction)
        .cloned()
        .collect()
}
