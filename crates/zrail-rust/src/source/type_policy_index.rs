//! One syntax traversal extracts type shape plus written duplication boundaries.

#[path = "type_policy_mounts.rs"]
mod mounts;
pub(crate) use mounts::inherit_replacing_mounts;

use std::collections::{BTreeMap, BTreeSet};

use syn::{
    Block, ItemImpl, ItemMod, ItemStruct, ItemUse,
    spanned::Spanned,
    visit::{self, Visit},
};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    FactNamespace, ObservedFact, SyntaxGuard,
    attributes::cfg_guard,
    fact::{source_span, written_fact},
    ordinary_binding_facts::replacement_macros,
    type_policy_model::{
        DuplicationSyntaxFact, DuplicationSyntaxKind, TraitImplFact, TraitImplPolarity,
        TypeDeclarationFact, TypeDeclarationKind, TypePolicyFacts,
    },
    type_policy_syntax::{
        collect_tokens, collect_use, derives, duplication_trait, last_segment, named_fields,
        nominal_type_span, visibility,
    },
    visitor_parts::visitor_context::item_attrs,
};

pub(super) fn collect(syntax: &syn::File) -> (TypePolicyFacts, Vec<ObservedFact>) {
    let mut collector = Collector {
        guard: cfg_guard(&syntax.attrs),
        replacement_macros: replacement_macros(&syntax.attrs, &cfg_guard(&syntax.attrs), &[]),
        ..Collector::default()
    };
    collector.visit_file(syntax);
    for declaration in &mut collector.facts.declarations {
        declaration.child_module_guards = collector
            .child_modules
            .get(&declaration.lexical_scope)
            .cloned()
            .unwrap_or_default();
    }
    collector.synthetic_paths.sort_by_key(|fact| fact.span);
    (collector.facts, collector.synthetic_paths)
}

#[derive(Default)]
struct Collector {
    facts: TypePolicyFacts,
    synthetic_paths: Vec<ObservedFact>,
    child_modules: BTreeMap<Vec<SourceSpan>, Vec<SyntaxGuard>>,
    lexical_scope: Vec<SourceSpan>,
    guard: SyntaxGuard,
    replacement_macros: Vec<super::macro_binding_policy::MacroOccurrence>,
}

impl<'ast> Visit<'ast> for Collector {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let previous = self.guard.clone();
        self.guard = previous.combine(cfg_guard(item_attrs(item)));
        let previous_macros = self.replacement_macros.len();
        self.replacement_macros.extend(replacement_macros(
            item_attrs(item),
            &self.guard,
            &self.lexical_scope,
        ));
        visit::visit_item(self, item);
        self.replacement_macros.truncate(previous_macros);
        self.guard = previous;
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.child_modules
            .entry(self.lexical_scope.clone())
            .or_default()
            .push(self.guard.clone());
        let Some((_, items)) = &module.content else {
            return;
        };
        self.lexical_scope.push(source_span(module.ident.span()));
        for item in items {
            self.visit_item(item);
        }
        self.lexical_scope.pop();
    }

    fn visit_block(&mut self, block: &'ast Block) {
        self.lexical_scope.push(source_span(block.span()));
        visit::visit_block(self, block);
        self.lexical_scope.pop();
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        let identity_span = source_span(item.ident.span());
        let mut identity = written_fact(
            item.ident.to_string(),
            item.ident.to_string(),
            item.ident.span(),
            AnalysisQuality::Exact,
            &self.lexical_scope,
        );
        identity.namespace = FactNamespace::Type;
        identity.guard = self.guard.clone();
        self.synthetic_paths.push(identity);
        self.facts.declarations.push(TypeDeclarationFact {
            identity_span,
            kind: if matches!(item.fields, syn::Fields::Named(_)) {
                TypeDeclarationKind::NamedStruct
            } else {
                TypeDeclarationKind::Other
            },
            visibility: visibility(&item.vis),
            fields: named_fields(&item.fields, &self.guard),
            derives: derives(&item.attrs, &self.guard),
            guard: self.guard.clone(),
            lexical_scope: self.lexical_scope.clone(),
            child_module_guards: Vec::new(),
            replacement_macros: self.replacement_macros.clone(),
            replacing_mounts: BTreeSet::new(),
        });
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        let identity_span = source_span(item.ident.span());
        let mut identity = written_fact(
            item.ident.to_string(),
            item.ident.to_string(),
            item.ident.span(),
            AnalysisQuality::Exact,
            &self.lexical_scope,
        );
        identity.namespace = FactNamespace::Type;
        identity.guard = self.guard.clone();
        self.synthetic_paths.push(identity);
        self.facts.declarations.push(TypeDeclarationFact {
            identity_span,
            kind: TypeDeclarationKind::Other,
            visibility: visibility(&item.vis),
            fields: None,
            derives: derives(&item.attrs, &self.guard),
            guard: self.guard.clone(),
            lexical_scope: self.lexical_scope.clone(),
            child_module_guards: Vec::new(),
            replacement_macros: self.replacement_macros.clone(),
            replacing_mounts: BTreeSet::new(),
        });
        visit::visit_item_enum(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        if let Some((negative, trait_path, _)) = &item.trait_ {
            self.facts.trait_impls.push(TraitImplFact {
                trait_span: source_span(trait_path.span()),
                trait_hint: last_segment(trait_path),
                type_span: nominal_type_span(&item.self_ty),
                polarity: if negative.is_some() {
                    TraitImplPolarity::Negative
                } else {
                    TraitImplPolarity::Positive
                },
                guard: self.guard.clone(),
                lexical_scope: self.lexical_scope.clone(),
            });
        }
        visit::visit_item_impl(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        collect_use(
            &item.tree,
            &self.guard,
            &self.lexical_scope,
            &mut self.facts.syntax,
        );
        visit::visit_item_use(self, item);
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        for ident in std::iter::once(&item.ident).chain(item.rename.as_ref().map(|(_, name)| name))
        {
            if let Some(trait_name) = duplication_trait(&ident.to_string()) {
                self.facts.syntax.push(DuplicationSyntaxFact {
                    kind: DuplicationSyntaxKind::Import,
                    trait_name,
                    span: source_span(ident.span()),
                    guard: self.guard.clone(),
                    lexical_scope: self.lexical_scope.clone(),
                });
            }
        }
    }

    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        collect_tokens(
            invocation.tokens.clone(),
            &self.guard,
            &self.lexical_scope,
            &mut self.facts.syntax,
        );
        visit::visit_macro(self, invocation);
    }
}

#[cfg(test)]
#[path = "type_policy_index_test.rs"]
mod type_policy_index_test;
