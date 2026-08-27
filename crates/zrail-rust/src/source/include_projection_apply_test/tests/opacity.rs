//! Macro opacity and target cfg uncertainty remain conservative without impersonating resolver failure.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::*;

#[test]
fn explicit_and_local_bindings_win_over_opaque_item_namespaces() {
    let file = parsed_file(
        "src/lib.rs",
        concat!(
            "unknown_items!(); use external::Message; struct Local; ",
            "fn accepts(_: Local) -> Message { loop {} }",
        ),
    );
    let mut index = index(file);
    let bindings = root_bindings(&index, false);

    let findings = bindings.apply(&mut index);

    assert!(
        findings.is_empty(),
        "{findings:#?}\n{:#?}",
        index.files[0].paths
    );
    for name in ["Local", "Message"] {
        assert!(index.files[0].paths.iter().any(|fact| {
            fact.written.as_deref() == Some(name) && fact.quality == AnalysisQuality::Exact
        }));
    }
}

#[test]
fn opaque_item_namespace_does_not_block_ordinary_binding_completeness() {
    let file = parsed_file("src/lib.rs", "unknown_items!(); fn accepts(_: String) {}");
    let mut index = index(file);
    let bindings = root_bindings(&index, true);

    let findings = bindings.apply(&mut index);

    assert!(findings.is_empty());
    assert!(index.files[0].paths.iter().any(|fact| {
        fact.written.as_deref() == Some("String") && fact.quality == AnalysisQuality::Unresolved
    }));
}

#[test]
fn unreviewed_item_namespace_still_blocks_completeness() {
    let file = parsed_file("src/lib.rs", "unknown_items!(); fn accepts(_: String) {}");
    let mut index = index(file);
    let bindings = root_bindings(&index, false);

    let findings = bindings.apply(&mut index);

    assert!(
        findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002")
    );
}

#[test]
fn authorized_opaque_namespace_keeps_struct_update_complete_and_fail_closed() {
    let file = parsed_file(
        "src/lib.rs",
        "#[unknown] struct State { value: u64 } fn update(base: State) { let _ = State { ..base }; }",
    );
    let mut index = index(file);
    let bindings = root_bindings(&index, true);

    let findings = canonicalize_operations(&mut index, &bindings);

    assert!(findings.is_empty(), "{findings:#?}");
    assert!(
        index.files[0].operations.iter().any(|operation| {
            operation.kind == crate::source::SourceOperationKind::FieldRead
                && operation.identity.name.ends_with("State::value")
                && operation.identity.quality == AnalysisQuality::Unresolved
        }),
        "{:#?}",
        index.files[0].operations
    );
}

#[test]
fn unreviewed_opaque_namespace_blocks_struct_update_completeness() {
    let file = parsed_file(
        "src/lib.rs",
        "#[unknown] struct State { value: u64 } fn update(base: State) { let _ = State { ..base }; }",
    );
    let mut index = index(file);
    let bindings = root_bindings(&index, false);

    let findings = canonicalize_operations(&mut index, &bindings);

    assert!(
        findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002")
    );
}

#[test]
fn target_cfg_alias_is_conservative_without_blocking_completeness() {
    let file = parsed_file(
        "src/lib.rs",
        "#[cfg(unix)] use platform::Thing; fn accepts(_: Thing) {}",
    );
    let mut index = index(file);
    let bindings = root_bindings(&index, false);

    let findings = bindings.apply(&mut index);

    assert!(findings.is_empty());
    assert!(index.files[0].paths.iter().any(|fact| {
        fact.written.as_deref() == Some("Thing") && fact.quality == AnalysisQuality::Unresolved
    }));
}

#[test]
fn mutually_exclusive_target_cfg_bindings_are_not_ambiguous() {
    let file = parsed_file(
        "src/lib.rs",
        concat!(
            "#[cfg(unix)] fn platform() {} ",
            "#[cfg(not(unix))] fn platform() {} ",
            "fn invoke() { platform(); }",
        ),
    );
    let mut index = index(file);
    let bindings = root_bindings(&index, false);

    let findings = bindings.apply(&mut index);

    assert!(findings.is_empty(), "{findings:#?}");
    assert!(index.files[0].calls.iter().any(|fact| {
        fact.written.as_deref() == Some("platform") && fact.quality != AnalysisQuality::Unresolved
    }));
}

#[test]
fn exhaustive_single_value_target_partition_is_not_ambiguous() {
    let file = parsed_file(
        "src/lib.rs",
        concat!(
            "#[cfg(target_os = \"linux\")] const TOOL: &str = \"linux\"; ",
            "#[cfg(target_os = \"macos\")] const TOOL: &str = \"macos\"; ",
            "#[cfg(not(any(target_os = \"linux\", target_os = \"macos\")))] ",
            "const TOOL: &str = \"other\"; fn selected() -> &'static str { TOOL }",
        ),
    );
    let mut index = index(file);
    let bindings = root_bindings(&index, false);

    let findings = bindings.apply(&mut index);

    assert!(findings.is_empty(), "{findings:#?}");
    assert!(index.files[0].paths.iter().any(|fact| {
        fact.written.as_deref() == Some("TOOL") && fact.quality != AnalysisQuality::Unresolved
    }));
}

#[test]
fn bare_self_receiver_is_an_exact_local_value_under_glob_imports() {
    let file = parsed_file(
        "src/lib.rs",
        concat!(
            "use external::*; struct State; ",
            "impl State { fn observe(&self) { let _ = self; } }",
        ),
    );
    let mut index = index(file);
    let bindings = root_bindings(&index, false);

    let findings = bindings.apply(&mut index);

    assert!(findings.is_empty(), "{findings:#?}");
    assert!(index.files[0].paths.iter().any(|fact| {
        fact.written.as_deref() == Some("self") && fact.quality == AnalysisQuality::Exact
    }));
}

#[test]
fn true_alias_cycle_still_blocks_completeness() {
    let file = parsed_file("src/lib.rs", "use B as A; use A as B; fn accepts(_: A) {}");
    let mut index = index(file);
    let bindings = root_bindings(&index, false);

    let findings = bindings.apply(&mut index);

    assert!(
        findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002")
    );
}

fn index(file: RustFileFacts) -> SourceIndex {
    SourceIndex {
        files: vec![file],
        findings: Vec::new(),
        analysis_metrics: SourceAnalysisMetrics::default(),
    }
}

fn root_bindings(index: &SourceIndex, authorize_opacity: bool) -> IncludeBindings {
    let mut policy = crate::source::BindingMacroPolicy::default();
    if authorize_opacity {
        for expansion in &index.files[0].macro_expansions {
            policy.accept_opaque("src/lib.rs", expansion);
        }
    }
    IncludeBindings::collect_with_extern_roots(
        index,
        &[CompilationRoot {
            file: "src/lib.rs".into(),
            domain: domain(),
        }],
        &[],
        &[],
        &policy,
        None,
        BTreeMap::from([("fixture".into(), BTreeSet::from(["external".into()]))]),
    )
}

fn canonicalize_operations(
    index: &mut SourceIndex,
    bindings: &IncludeBindings,
) -> Vec<zrail_core::Finding> {
    let mut findings = bindings.apply(index);
    findings.extend(crate::source::operation_canonical::apply(
        index,
        bindings,
        &BTreeMap::from([("src/lib.rs".into(), BTreeSet::from([domain()]))]),
        &zrail_core::AnalysisLimits::default(),
    ));
    findings
}
