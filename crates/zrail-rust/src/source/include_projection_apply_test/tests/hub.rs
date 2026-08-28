//! Parent hubs and repeated path mounts retain one exact identity per instance.

use zrail_core::{AnalysisQuality, OwnerContract, OwnerKind, PolicyReachability};

use super::*;
use crate::source::{
    CompilationDomain, CompilationModuleEdge, ModuleDeclaration, SourceOperationKind,
};

#[test]
fn hub_aliases_resolve_across_repeated_child_mounts() {
    let root = parsed_file("src/lib.rs", "mod hub;");
    let hub = parsed_file(
        "src/hub.rs",
        r#"
pub(super) use std::result::Result as Alias;
#[path = "shared.rs"] mod left;
#[path = "shared.rs"] mod right;
"#,
    );
    let child = parsed_file(
        "src/shared.rs",
        r"
use super::Alias;
pub fn value(input: u32) -> Alias<String> { Ok(input.to_string()) }
struct Inner { value: u32 }
struct Local { inner: Inner }
impl Local { fn set(&mut self) { self.inner.value = 1; } }
",
    );
    let domain = domain();
    let modules = vec![
        module_edge("src/lib.rs", "hub", "src/hub.rs", &root.modules[0], &domain),
        module_edge(
            "src/hub.rs",
            "left",
            "src/shared.rs",
            module(&hub.modules, "left"),
            &domain,
        ),
        module_edge(
            "src/hub.rs",
            "right",
            "src/shared.rs",
            module(&hub.modules, "right"),
            &domain,
        ),
    ];
    let mut index = SourceIndex {
        files: vec![root, hub, child],
        findings: Vec::new(),
        analysis_metrics: SourceAnalysisMetrics::default(),
    };
    let bindings = IncludeBindings::collect(
        &index,
        &[CompilationRoot {
            file: "src/lib.rs".into(),
            syntax: SourceSyntax::Items,
            domain: domain.clone(),
        }],
        &modules,
        &[],
        &crate::source::BindingMacroPolicy::default(),
    );

    assert_eq!(
        bindings
            .instances
            .for_source("src/shared.rs", SourceSyntax::Items)
            .len(),
        2
    );
    assert_eq!(bindings.instances.metrics().derived_contexts, 1);
    assert!(bindings.apply(&mut index).is_empty());
    let domains = compilation_domains(&index, &domain);
    crate::source::operation_place_canonical::apply(&mut index, &domains);
    let child = index
        .files
        .iter()
        .find(|file| file.relative == "src/shared.rs")
        .expect("shared child facts");
    assert!(child.paths.iter().any(|fact| {
        fact.name == "std::result::Result" && fact.quality == AnalysisQuality::Exact
    }));
    assert!(
        child
            .paths
            .iter()
            .all(|fact| fact.quality != AnalysisQuality::Unresolved),
        "child path identities: {:?}",
        child.paths
    );
    let write = child
        .operations
        .iter()
        .find(|operation| {
            operation.kind == SourceOperationKind::FieldWrite
                && operation.identity.written.as_deref() == Some("value")
        })
        .expect("nested field write");
    assert_eq!(
        write.identity.canonical,
        ["hub::left::Inner::value", "hub::right::Inner::value"]
    );
    assert_eq!(write.identity.quality, AnalysisQuality::Conservative);
}

#[test]
fn hub_types_repair_nested_self_and_typed_field_places() {
    let root = parsed_file("src/lib.rs", "mod hub;");
    let hub = parsed_file(
        "src/hub.rs",
        concat!(
            "mod ",
            "state;\nmod ",
            "behavior;\npub(super) use state::{Node, PersistentState};\n"
        ),
    );
    let state = parsed_file(
        "src/state.rs",
        r"
pub(super) struct PersistentState { pub(super) current_term: u64 }
pub(super) struct Node { pub(super) persistent: PersistentState }
",
    );
    let behavior = parsed_file(
        "src/behavior.rs",
        r"
use super::Node;
impl Node {
    fn advance(&mut self, typed: &mut Node) {
        self.persistent.current_term += 1;
        typed.persistent.current_term = 2;
        let _ = self.persistent.current_term.saturating_add(1);
    }
}
",
    );
    let domain = domain();
    let modules = vec![
        module_edge("src/lib.rs", "hub", "src/hub.rs", &root.modules[0], &domain),
        module_edge(
            "src/hub.rs",
            "state",
            "src/state.rs",
            module(&hub.modules, "state"),
            &domain,
        ),
        module_edge(
            "src/hub.rs",
            "behavior",
            "src/behavior.rs",
            module(&hub.modules, "behavior"),
            &domain,
        ),
    ];
    let mut index = SourceIndex {
        files: vec![root, hub, state, behavior],
        findings: Vec::new(),
        analysis_metrics: SourceAnalysisMetrics::default(),
    };
    let bindings = IncludeBindings::collect(
        &index,
        &[CompilationRoot {
            file: "src/lib.rs".into(),
            syntax: SourceSyntax::Items,
            domain: domain.clone(),
        }],
        &modules,
        &[],
        &crate::source::BindingMacroPolicy::default(),
    );

    assert!(bindings.apply(&mut index).is_empty());
    let domains = compilation_domains(&index, &domain);
    crate::source::operation_place_canonical::apply(&mut index, &domains);
    let operations = &index
        .files
        .iter()
        .find(|file| file.relative == "src/behavior.rs")
        .expect("behavior facts")
        .operations;
    let identity = "hub::state::PersistentState::current_term";
    let writes = operations
        .iter()
        .filter(|operation| {
            operation.kind == SourceOperationKind::FieldWrite && operation.identity.name == identity
        })
        .collect::<Vec<_>>();
    assert_eq!(writes.len(), 2, "nested writes: {operations:?}");
    assert!(
        writes
            .iter()
            .all(|operation| operation.identity.quality == AnalysisQuality::Exact)
    );
    assert!(operations.iter().any(|operation| {
        operation.kind == SourceOperationKind::FieldReceiverCall
            && operation.identity.name == identity
            && operation.method.as_deref() == Some("saturating_add")
            && operation.identity.quality == AnalysisQuality::Exact
    }));
    let owner = OwnerContract {
        name: "term-mutation".into(),
        kind: OwnerKind::FieldMutation,
        reachability: PolicyReachability::All,
        within: vec!["src/**".into()],
        selector: identity.into(),
        mutating_methods: vec!["saturating_add".into()],
        allow: vec!["src/behavior.rs".into()],
        reason: "term mutation stays bounded".into(),
    };
    let behavior = index
        .files
        .iter()
        .find(|file| file.relative == "src/behavior.rs")
        .expect("behavior facts");
    assert_eq!(
        crate::rules::matching_operation_owner_operations(&owner, behavior).count(),
        3,
        "field-mutation matches typed writes plus the declared receiver method"
    );
}

fn module<'a>(modules: &'a [ModuleDeclaration], name: &str) -> &'a ModuleDeclaration {
    modules
        .iter()
        .find(|module| module.name == name)
        .expect("module declaration")
}

fn module_edge(
    parent: &str,
    module_name: &str,
    child: &str,
    declaration: &ModuleDeclaration,
    domain: &CompilationDomain,
) -> CompilationModuleEdge {
    CompilationModuleEdge {
        parent: parent.into(),
        parent_syntax: SourceSyntax::Items,
        module_name: module_name.into(),
        child: child.into(),
        child_syntax: SourceSyntax::Items,
        domain: domain.clone(),
        guard: declaration.guard.clone(),
        parent_scope: declaration.lexical_scope.clone(),
        span: declaration.span,
    }
}

fn compilation_domains(
    index: &SourceIndex,
    domain: &CompilationDomain,
) -> std::collections::BTreeMap<String, std::collections::BTreeSet<CompilationDomain>> {
    index
        .files
        .iter()
        .map(|file| {
            (
                file.relative.clone(),
                std::collections::BTreeSet::from([domain.clone()]),
            )
        })
        .collect()
}
