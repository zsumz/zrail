//! Lexical item scopes install imports and declaration facts transactionally.

use syn::Item;

use super::super::{FactVisitor, ordinary_bindings, scoped_globs, scoped_imports};
use super::LocalImportScope;

impl FactVisitor<'_> {
    pub(in crate::source) fn with_import_scope<'a>(
        &mut self,
        items: impl Iterator<Item = &'a Item>,
        visit: impl FnOnce(&mut Self),
    ) {
        let items = items.collect::<Vec<_>>();
        let aliases =
            scoped_imports::collect(items.iter().copied(), |path| self.resolve_text(path));
        let globs = scoped_globs::collect(items.iter().copied());
        let enclosing_guard = self.syntax_guard();
        let lexical_scope = self.lexical_scope.clone();
        self.inline_module_scopes
            .extend(items.iter().filter_map(|item| {
                let Item::Mod(module) = item else {
                    return None;
                };
                module
                    .content
                    .as_ref()
                    .map(|_| super::super::fact::source_span(module.ident.span()))
            }));
        self.glob_imports
            .extend(super::super::glob_imports::collect(
                items.iter().copied(),
                &enclosing_guard,
                &lexical_scope,
            ));
        self.import_bindings.extend(ordinary_bindings::collect(
            items.iter().copied(),
            &enclosing_guard,
            &lexical_scope,
        ));
        self.associated_items
            .extend(crate::source::associated_items::collect(
                items.iter().copied(),
                &enclosing_guard,
                &lexical_scope,
            ));
        self.trait_inheritance
            .extend(crate::source::trait_providers::collect(
                items.iter().copied(),
                &enclosing_guard,
                &lexical_scope,
            ));
        self.local_imports.push(LocalImportScope { aliases, globs });
        visit(self);
        self.local_imports.pop();
    }
}
