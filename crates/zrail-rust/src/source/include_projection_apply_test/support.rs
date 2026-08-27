//! Shared fixture identities keep the projection tests focused on behavior.

use zrail_core::{OwnerContract, OwnerKind, PolicyReachability, SourceSpan};

use crate::{
    inventory::{FileClass, RustSourceFile},
    source::{
        CompilationDomain, CompilationMode, CompilationModuleEdge, CompilationRoot,
        ModuleDeclaration, Reachability, RustFileFacts, SourceIndex, SourceSyntax,
        imports::ImportMap, include_bindings::IncludeBindings, visitor::FactVisitor,
    },
};
use syn::visit::Visit;

pub(super) fn projected_call_count(index: &SourceIndex) -> usize {
    named_call_count(index, "std::process::Command::new")
}

pub(super) fn named_call_count(index: &SourceIndex, name: &str) -> usize {
    index
        .files
        .iter()
        .flat_map(|file| &file.calls)
        .filter(|fact| fact.name == name)
        .count()
}

pub(super) fn observed_names(index: &SourceIndex) -> Vec<(String, Vec<String>)> {
    let mut observed = index
        .files
        .iter()
        .map(|file| {
            let mut names = file
                .calls
                .iter()
                .map(|fact| fact.name.clone())
                .collect::<Vec<_>>();
            names.sort();
            (file.relative.clone(), names)
        })
        .collect::<Vec<_>>();
    observed.sort();
    observed
}

pub(super) fn domain() -> CompilationDomain {
    CompilationDomain {
        package: "fixture".into(),
        edition: "2024".into(),
        target: "fixture".into(),
        mode: CompilationMode::Library,
        feature_world: None,
        active_features: std::collections::BTreeSet::default(),
    }
}

pub(super) const fn span() -> SourceSpan {
    SourceSpan {
        line: 1,
        column: 0,
        end_line: 1,
        end_column: 1,
    }
}

pub(super) fn parsed_file(relative: &str, source: &str) -> RustFileFacts {
    let source_file = RustSourceFile {
        relative: relative.into(),
        class: FileClass::Implementation,
        source: source.into(),
        lines: source.lines().count(),
    };
    let syntax = syn::parse_file(source).expect("parse source fixture");
    let imports = ImportMap::from_file(&syntax);
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_file(&syntax);
    let (type_policy, synthetic_paths) = crate::source::type_policy_index::collect(&syntax);
    visitor.paths.extend(synthetic_paths);
    RustFileFacts {
        relative: source_file.relative,
        packages: Vec::new(),
        class: source_file.class,
        reachability: Reachability::UNREACHABLE,
        syntax: SourceSyntax::Items,
        lines: source_file.lines,
        module_docs: false,
        paths: visitor.paths,
        calls: visitor.calls,
        call_resolutions: visitor.call_resolutions,
        methods: visitor.methods,
        operations: visitor.operations,
        macros: visitor.macros,
        macro_imports: imports.macro_imports(),
        macro_expansions: visitor.macro_expansions,
        opaque_macro_inputs: visitor.opaque_macro_inputs,
        macro_definitions: visitor.macro_definitions,
        import_bindings: visitor.import_bindings,
        associated_items: visitor.associated_items,
        glob_imports: visitor.glob_imports,
        inline_module_scopes: visitor.inline_module_scopes,
        compile_effects: visitor.compile_effects,
        lint_suppressions: visitor.lint_suppressions,
        unsafe_constructs: visitor.unsafe_constructs,
        async_syntax: visitor.async_syntax,
        type_policy,
        tests: visitor.tests,
        modules: crate::source::modules::module_declarations(&syntax),
        includes: visitor.includes,
        item_macros: visitor.item_macros,
        opaque_binding_macros: visitor.opaque_binding_macros,
        facade_implementation: Vec::new(),
    }
}

pub(super) fn module<'a>(modules: &'a [ModuleDeclaration], name: &str) -> &'a ModuleDeclaration {
    modules
        .iter()
        .find(|module| module.name == name)
        .expect("module declaration")
}

pub(super) fn module_edge(
    parent: &str,
    module_name: &str,
    child: &str,
    declaration: &ModuleDeclaration,
    domain: &CompilationDomain,
) -> CompilationModuleEdge {
    CompilationModuleEdge {
        parent: parent.into(),
        module_name: module_name.into(),
        child: child.into(),
        domain: domain.clone(),
        guard: declaration.guard.clone(),
        parent_scope: declaration.lexical_scope.clone(),
        span: declaration.span,
    }
}

pub(super) fn canonicalize_operations(
    index: &mut SourceIndex,
    domain: &CompilationDomain,
    modules: &[CompilationModuleEdge],
) -> Vec<zrail_core::Finding> {
    canonicalize_operation_worlds(index, std::slice::from_ref(domain), modules)
}

pub(super) fn canonicalize_operations_with_external(
    index: &mut SourceIndex,
    domain: &CompilationDomain,
    modules: &[CompilationModuleEdge],
    external: &str,
) -> Vec<zrail_core::Finding> {
    let bindings = IncludeBindings::collect_with_extern_roots(
        index,
        &[CompilationRoot {
            file: "src/lib.rs".into(),
            domain: domain.clone(),
        }],
        modules,
        &[],
        &crate::source::BindingMacroPolicy::default(),
        None,
        std::collections::BTreeMap::from([(
            domain.package.clone(),
            std::collections::BTreeSet::from([external.into()]),
        )]),
    );
    let mut findings = bindings.apply(index);
    let domains = index
        .files
        .iter()
        .map(|file| {
            (
                file.relative.clone(),
                std::collections::BTreeSet::from([domain.clone()]),
            )
        })
        .collect();
    findings.extend(crate::source::operation_canonical::apply(
        index,
        &bindings,
        &domains,
        &zrail_core::AnalysisLimits::default(),
    ));
    crate::source::operation_place_canonical::apply(index, &domains);
    findings
}

pub(super) fn canonicalize_operation_worlds(
    index: &mut SourceIndex,
    domains: &[CompilationDomain],
    modules: &[CompilationModuleEdge],
) -> Vec<zrail_core::Finding> {
    let bindings = IncludeBindings::collect(
        index,
        &domains
            .iter()
            .map(|domain| CompilationRoot {
                file: "src/lib.rs".into(),
                domain: domain.clone(),
            })
            .collect::<Vec<_>>(),
        modules,
        &[],
        &crate::source::BindingMacroPolicy::default(),
    );
    let mut findings = bindings.apply(index);
    let domains = index
        .files
        .iter()
        .map(|file| (file.relative.clone(), domains.iter().cloned().collect()))
        .collect();
    findings.extend(crate::source::operation_canonical::apply(
        index,
        &bindings,
        &domains,
        &zrail_core::AnalysisLimits::default(),
    ));
    crate::source::operation_place_canonical::apply(index, &domains);
    findings
}

pub(super) fn matching_operations(
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
