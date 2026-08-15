//! External module declarations retain inline ancestry for exact path resolution.

use syn::{
    ItemMod,
    visit::{self, Visit},
};

use super::{
    attributes::{has_conditional_path_attribute, has_path_attribute, is_cfg_test, path_attribute},
    fact::source_span,
    model::{InlineModulePath, ModuleDeclaration},
    visitor_context::{expr_attrs, foreign_attrs, impl_attrs, item_attrs, trait_attrs},
};

pub(super) fn module_declarations(file: &syn::File) -> Vec<ModuleDeclaration> {
    let mut collector = ModuleCollector::default();
    collector.visit_file(file);
    collector.declarations
}

#[derive(Default)]
struct ModuleCollector {
    inline_ancestors: Vec<InlineModulePath>,
    declarations: Vec<ModuleDeclaration>,
    test_only_context: bool,
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
        let cfg_test = self.test_only_context || module.attrs.iter().any(is_cfg_test);
        let Some((_, items)) = &module.content else {
            self.declarations.push(ModuleDeclaration {
                name: module.ident.to_string(),
                path,
                cfg_test,
                unresolved_path,
                inline_ancestors: self.inline_ancestors.clone(),
                span: Some(source_span(module.ident.span())),
            });
            return;
        };
        self.inline_ancestors.push(InlineModulePath {
            name: module.ident.to_string(),
            path,
            unresolved_path,
        });
        let previous_context = self.test_only_context;
        self.test_only_context = cfg_test;
        for item in items {
            visit::visit_item(self, item);
        }
        self.test_only_context = previous_context;
        self.inline_ancestors.pop();
    }
}

impl ModuleCollector {
    fn with_cfg(&mut self, attributes: &[syn::Attribute], visit: impl FnOnce(&mut Self)) {
        let previous = self.test_only_context;
        self.test_only_context |= attributes.iter().any(is_cfg_test);
        visit(self);
        self.test_only_context = previous;
    }
}

#[cfg(test)]
#[path = "modules_test.rs"]
mod modules_test;
