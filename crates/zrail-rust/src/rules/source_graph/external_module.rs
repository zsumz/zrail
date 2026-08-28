//! External module declarations become exact Rust source-graph edges.

use crate::source::{
    ModuleDeclaration, ModuleTarget, ResolvedModuleEdge, SourceSyntax, SubmoduleBase, module_target,
};

use super::{TraversalContext, Walker};

impl Walker<'_> {
    pub(super) fn walk_module(
        &mut self,
        source: &str,
        source_syntax: SourceSyntax,
        submodule_base: SubmoduleBase,
        context: &TraversalContext,
        declaration: &ModuleDeclaration,
    ) {
        let label = format!("module {:?}", declaration.name);
        let Some(target_context) = context.with_guard(&declaration.guard) else {
            return;
        };
        match module_target(source, submodule_base, declaration) {
            Ok(ModuleTarget::Exact(path)) => self.follow_module(
                ResolvedModuleEdge {
                    parent: source.to_owned(),
                    parent_syntax: source_syntax,
                    module_name: declaration.name.clone(),
                    child: path,
                    child_syntax: SourceSyntax::Items,
                    child_base: SubmoduleBase::SourceParent,
                    reachability: target_context.reachability,
                    guard: target_context.guard.clone(),
                    span: declaration.span,
                },
                source_syntax,
                &declaration.lexical_scope,
                declaration.span,
                &label,
                SourceSyntax::Items,
                &target_context,
            ),
            Ok(ModuleTarget::Search { direct, nested }) => {
                let candidates = [
                    (direct, SubmoduleBase::FileStemDirectory),
                    (nested, SubmoduleBase::SourceParent),
                ]
                .into_iter()
                .filter(|(path, _)| self.entries.contains_key(path.as_str()))
                .collect::<Vec<_>>();
                match candidates.as_slice() {
                    [(path, submodule_base)] => self.follow_module(
                        ResolvedModuleEdge {
                            parent: source.to_owned(),
                            parent_syntax: source_syntax,
                            module_name: declaration.name.clone(),
                            child: path.clone(),
                            child_syntax: SourceSyntax::Items,
                            child_base: *submodule_base,
                            reachability: target_context.reachability,
                            guard: target_context.guard.clone(),
                            span: declaration.span,
                        },
                        source_syntax,
                        &declaration.lexical_scope,
                        declaration.span,
                        &label,
                        SourceSyntax::Items,
                        &target_context,
                    ),
                    [] => self.missing(
                        source,
                        declaration.span,
                        format!("{label} has no source file at either Rust module path"),
                    ),
                    _ => self.missing(
                        source,
                        declaration.span,
                        format!("{label} is ambiguous because both Rust module paths exist"),
                    ),
                }
            }
            Err(error) => self.resolution_error(source, declaration.span, &error, &label),
        }
    }
}
