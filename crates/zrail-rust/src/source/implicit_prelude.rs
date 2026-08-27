//! Prelude-affecting attributes retain guarded lexical module scope.

use syn::{
    Attribute, Block, Item, ItemMod,
    spanned::Spanned,
    visit::{self, Visit},
};
use zrail_core::SourceSpan;

use crate::source::{SyntaxGuard, attributes::cfg_guard, fact::source_span};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum PreludeDirectiveKind {
    NoStd,
    NoImplicit,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PreludeDirective {
    pub(crate) kind: PreludeDirectiveKind,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
}

pub(in crate::source) fn directives(file: &syn::File) -> Vec<PreludeDirective> {
    let mut collector = DirectiveCollector::default();
    collector.visit_file(file);
    collector.directives.sort();
    collector.directives.dedup();
    collector.directives
}

#[derive(Default)]
struct DirectiveCollector {
    guard: SyntaxGuard,
    lexical_scope: Vec<SourceSpan>,
    directives: Vec<PreludeDirective>,
}

impl<'ast> Visit<'ast> for DirectiveCollector {
    fn visit_file(&mut self, file: &'ast syn::File) {
        let previous = self.guard.clone();
        self.guard = previous.combine(cfg_guard(&file.attrs));
        self.collect(&file.attrs, &[], true);
        for item in &file.items {
            self.visit_item(item);
        }
        self.guard = previous;
    }

    fn visit_item(&mut self, item: &'ast Item) {
        let previous = self.guard.clone();
        self.guard = previous.combine(cfg_guard(
            crate::source::visitor_parts::visitor_context::item_attrs(item),
        ));
        visit::visit_item(self, item);
        self.guard = previous;
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let span = source_span(module.ident.span());
        let mut module_scope = self.lexical_scope.clone();
        module_scope.push(span);
        self.collect(&module.attrs, &module_scope, false);
        if let Some((_, items)) = &module.content {
            self.lexical_scope.push(span);
            for item in items {
                self.visit_item(item);
            }
            self.lexical_scope.pop();
        }
    }

    fn visit_block(&mut self, block: &'ast Block) {
        self.lexical_scope.push(source_span(block.span()));
        visit::visit_block(self, block);
        self.lexical_scope.pop();
    }
}

impl DirectiveCollector {
    fn collect(&mut self, attributes: &[Attribute], scope: &[SourceSpan], allow_no_std: bool) {
        for attribute in attributes {
            let Ok(effects) = crate::source::cfg::cfg_guards::guarded_attribute_effects(attribute)
            else {
                continue;
            };
            for effect in effects {
                let kind = if effect.meta.path().is_ident("no_implicit_prelude") {
                    Some(PreludeDirectiveKind::NoImplicit)
                } else if allow_no_std && effect.meta.path().is_ident("no_std") {
                    Some(PreludeDirectiveKind::NoStd)
                } else {
                    None
                };
                let Some(kind) = kind else { continue };
                self.directives.push(PreludeDirective {
                    kind,
                    guard: self.guard.combine(effect.guard),
                    lexical_scope: scope.to_vec(),
                });
            }
        }
    }
}

#[cfg(test)]
#[path = "implicit_prelude_test.rs"]
mod implicit_prelude_test;
