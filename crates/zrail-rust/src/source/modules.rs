//! External module declarations retain inline ancestry for exact path resolution.

use syn::{
    ItemMod,
    visit::{self, Visit},
};

use super::{
    attributes::{has_conditional_path_attribute, has_path_attribute, is_cfg_test, path_attribute},
    fact::source_span,
    model::{InlineModulePath, ModuleDeclaration},
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
}

impl<'ast> Visit<'ast> for ModuleCollector {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let path = path_attribute(&module.attrs);
        let unresolved_path = has_conditional_path_attribute(&module.attrs)
            || (has_path_attribute(&module.attrs) && path.is_none());
        let Some((_, items)) = &module.content else {
            self.declarations.push(ModuleDeclaration {
                name: module.ident.to_string(),
                path,
                cfg_test: module.attrs.iter().any(is_cfg_test),
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
        for item in items {
            visit::visit_item(self, item);
        }
        self.inline_ancestors.pop();
    }
}

#[cfg(test)]
#[path = "modules_test.rs"]
mod modules_test;
