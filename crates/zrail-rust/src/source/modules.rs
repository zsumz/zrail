//! External module declarations retain inline ancestry for exact path resolution.

use syn::{
    Block, ItemMod,
    spanned::Spanned,
    visit::{self, Visit},
};
use zrail_core::SourceSpan;

use super::{
    SyntaxGuard,
    attributes::{cfg_guard, has_conditional_path_attribute, has_path_attribute, path_attribute},
    fact::source_span,
    model::{InlineModulePath, ModuleDeclaration},
    visitor_parts::visitor_context::{
        expr_attrs, foreign_attrs, impl_attrs, item_attrs, trait_attrs,
    },
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ResolvedModuleEdge {
    pub(crate) parent: String,
    pub(crate) module_name: String,
    pub(crate) child: String,
    pub(crate) child_base: super::SubmoduleBase,
    pub(crate) reachability: super::Reachability,
    pub(crate) guard: SyntaxGuard,
    pub(crate) span: Option<SourceSpan>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CompilationModuleEdge {
    pub(crate) parent: String,
    pub(crate) module_name: String,
    pub(crate) child: String,
    pub(crate) domain: super::CompilationDomain,
    pub(crate) guard: SyntaxGuard,
    pub(crate) parent_scope: Vec<SourceSpan>,
    pub(crate) span: Option<SourceSpan>,
}

pub(super) fn module_declarations(file: &syn::File) -> Vec<ModuleDeclaration> {
    let mut collector = ModuleCollector::default();
    collector.visit_file(file);
    collector.declarations
}

#[derive(Default)]
struct ModuleCollector {
    inline_ancestors: Vec<InlineModulePath>,
    lexical_scope: Vec<zrail_core::SourceSpan>,
    declarations: Vec<ModuleDeclaration>,
    guard_context: SyntaxGuard,
}

impl<'ast> Visit<'ast> for ModuleCollector {
    fn visit_file(&mut self, file: &'ast syn::File) {
        self.with_cfg(&file.attrs, |visitor| visit::visit_file(visitor, file));
    }

    fn visit_item(&mut self, item: &'ast syn::Item) {
        self.with_cfg(item_attrs(item), |visitor| visit::visit_item(visitor, item));
    }

    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        self.with_cfg(impl_attrs(item), |visitor| {
            visit::visit_impl_item(visitor, item);
        });
    }

    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        self.with_cfg(trait_attrs(item), |visitor| {
            visit::visit_trait_item(visitor, item);
        });
    }

    fn visit_foreign_item(&mut self, item: &'ast syn::ForeignItem) {
        self.with_cfg(foreign_attrs(item), |visitor| {
            visit::visit_foreign_item(visitor, item);
        });
    }

    fn visit_expr(&mut self, expression: &'ast syn::Expr) {
        self.with_cfg(expr_attrs(expression), |visitor| {
            visit::visit_expr(visitor, expression);
        });
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        self.with_cfg(&local.attrs, |visitor| visit::visit_local(visitor, local));
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        self.with_cfg(&arm.attrs, |visitor| visit::visit_arm(visitor, arm));
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let path = path_attribute(&module.attrs);
        let unresolved_path = has_conditional_path_attribute(&module.attrs)
            || (has_path_attribute(&module.attrs) && path.is_none());
        let guard = self.guard_context.clone();
        let Some((_, items)) = &module.content else {
            self.declarations.push(ModuleDeclaration {
                name: module.ident.to_string(),
                path,
                guard,
                unresolved_path,
                inline_ancestors: self.inline_ancestors.clone(),
                lexical_scope: self.lexical_scope.clone(),
                span: Some(source_span(module.ident.span())),
            });
            return;
        };
        self.inline_ancestors.push(InlineModulePath {
            name: module.ident.to_string(),
            path,
            unresolved_path,
        });
        self.lexical_scope.push(source_span(module.ident.span()));
        for item in items {
            self.visit_item(item);
        }
        self.lexical_scope.pop();
        self.inline_ancestors.pop();
    }

    fn visit_block(&mut self, block: &'ast Block) {
        self.lexical_scope.push(source_span(block.span()));
        visit::visit_block(self, block);
        self.lexical_scope.pop();
    }
}

impl ModuleCollector {
    fn with_cfg(&mut self, attributes: &[syn::Attribute], visit: impl FnOnce(&mut Self)) {
        let previous = self.guard_context.clone();
        self.guard_context = previous.combine(cfg_guard(attributes));
        visit(self);
        self.guard_context = previous;
    }
}

#[cfg(test)]
#[path = "modules_test.rs"]
mod modules_test;
