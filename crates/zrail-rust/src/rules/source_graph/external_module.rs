//! External module declarations become exact Rust source-graph edges.

use crate::source::{ModuleDeclaration, ModuleTarget, SourceSyntax, SubmoduleBase, module_target};

use super::{TraversalContext, Walker};

impl Walker<'_> {
    pub(super) fn walk_module(
        &mut self,
        source: &str,
        submodule_base: SubmoduleBase,
        context: &TraversalContext,
        declaration: &ModuleDeclaration,
    ) {
        let label = format!("module {:?}", declaration.name);
        let target_context = context.with_test_guard(declaration.cfg_test);
        match module_target(source, submodule_base, declaration) {
            Ok(ModuleTarget::Exact(path)) => self.follow(
                source,
                declaration.span,
                path,
                &label,
                SubmoduleBase::SourceParent,
                SourceSyntax::Items,
                target_context,
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
                    [(path, submodule_base)] => self.follow(
                        source,
                        declaration.span,
                        path.clone(),
                        &label,
                        *submodule_base,
                        SourceSyntax::Items,
                        target_context,
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
