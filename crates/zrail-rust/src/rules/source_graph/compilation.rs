//! Cargo roots, modules, and includes retain exact compilation entry edges.

use zrail_core::SourceSpan;

use crate::{
    cargo::CargoTargetKind,
    source::{
        CompilationIncludeEdge, CompilationMode, CompilationModuleEdge, CompilationRoot,
        IncludeBoundary, Reachability, ReachabilityKind, ResolvedModuleEdge, SourceSyntax,
        SubmoduleBase,
    },
};

use super::{TraversalContext, Walker};

impl Walker<'_> {
    pub(super) fn follow_root(
        &mut self,
        origin: &str,
        span: Option<SourceSpan>,
        target: String,
        label: &str,
        submodule_base: SubmoduleBase,
        expected_syntax: SourceSyntax,
        context: TraversalContext,
    ) {
        let root = CompilationRoot {
            file: target.clone(),
            syntax: expected_syntax,
            domain: context.domain.clone(),
        };
        if self.follow_resolved(
            origin,
            span,
            target,
            label,
            submodule_base,
            expected_syntax,
            context,
        ) {
            self.compilation_roots.insert(root);
        }
    }

    pub(super) fn follow_module(
        &mut self,
        edge: ResolvedModuleEdge,
        parent_syntax: SourceSyntax,
        parent_scope: &[SourceSpan],
        span: Option<SourceSpan>,
        label: &str,
        expected_syntax: SourceSyntax,
        context: &TraversalContext,
    ) {
        if self.follow_resolved(
            &edge.parent,
            span,
            edge.child.clone(),
            label,
            edge.child_base,
            expected_syntax,
            context.clone(),
        ) {
            self.compilation_edges.insert(CompilationModuleEdge {
                parent: edge.parent.clone(),
                parent_syntax,
                module_name: edge.module_name.clone(),
                child: edge.child.clone(),
                child_syntax: expected_syntax,
                domain: context.domain.clone(),
                guard: context.guard.clone(),
                parent_scope: parent_scope.to_vec(),
                span: edge.span,
            });
            self.module_edges.insert(edge);
        }
    }

    pub(super) fn follow_include(
        &mut self,
        parent: &str,
        parent_syntax: SourceSyntax,
        target: String,
        label: &str,
        expected_syntax: SourceSyntax,
        include: &IncludeBoundary,
        context: &TraversalContext,
    ) {
        if self.follow_resolved(
            parent,
            include.span,
            target.clone(),
            label,
            SubmoduleBase::SourceParent,
            expected_syntax,
            context.clone(),
        ) {
            self.compilation_includes.insert(CompilationIncludeEdge {
                parent: parent.into(),
                parent_syntax,
                child: target,
                child_syntax: expected_syntax,
                domain: context.domain.clone(),
                guard: context.guard.clone(),
                context: include.context,
                parent_scope: include.lexical_scope.clone(),
                generic_types: include.generic_types.clone(),
                generic_values: include.generic_values.clone(),
                trait_bounds: include.trait_bounds.clone(),
                current_self: include.current_self.clone(),
                inherits_parent_context: include.inherits_parent_context,
                value_shadows: include.value_shadows.clone(),
                include_span: include.occurrence.span(),
                occurrence: include.occurrence,
            });
        }
    }
}

pub(super) fn target_domains(kind: CargoTargetKind) -> Vec<(CompilationMode, Reachability)> {
    let reachability = target_reachability(kind);
    match kind {
        CargoTargetKind::Library => vec![
            (CompilationMode::Library, reachability),
            (CompilationMode::LibraryTest, reachability),
        ],
        CargoTargetKind::ProcMacro => vec![
            (CompilationMode::ProcMacro, reachability),
            (CompilationMode::ProcMacroTest, reachability),
        ],
        CargoTargetKind::Binary => vec![
            (CompilationMode::Binary, reachability),
            (CompilationMode::BinaryTest, reachability),
        ],
        CargoTargetKind::Test => vec![(CompilationMode::IntegrationTest, reachability)],
        CargoTargetKind::Benchmark => vec![(CompilationMode::Benchmark, reachability)],
        CargoTargetKind::Example => vec![
            (CompilationMode::Example, reachability),
            (CompilationMode::ExampleTest, reachability),
        ],
        CargoTargetKind::BuildScript => vec![(CompilationMode::BuildScript, reachability)],
    }
}

const fn target_reachability(kind: CargoTargetKind) -> Reachability {
    let kind = match kind {
        CargoTargetKind::Library | CargoTargetKind::Binary => ReachabilityKind::Production,
        CargoTargetKind::Test => ReachabilityKind::Test,
        CargoTargetKind::Benchmark => ReachabilityKind::Benchmark,
        CargoTargetKind::Example => ReachabilityKind::Example,
        CargoTargetKind::BuildScript | CargoTargetKind::ProcMacro => ReachabilityKind::Build,
    };
    Reachability::from_kind(kind)
}

#[cfg(test)]
#[path = "compilation_test.rs"]
mod compilation_test;
