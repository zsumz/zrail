//! Pattern-shape helpers enumerate lexical names without assigning invented types.

use std::collections::BTreeSet;

use syn::{Pat, Type, visit::Visit};

pub(super) fn typed_pattern(pattern: &Pat) -> (&Pat, Option<&Type>) {
    match pattern {
        Pat::Type(typed) => (&typed.pat, Some(&typed.ty)),
        _ => (pattern, None),
    }
}

pub(super) fn simple_binding_name(pattern: &Pat) -> Option<String> {
    match pattern {
        Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        Pat::Reference(reference) => simple_binding_name(&reference.pat),
        Pat::Paren(paren) => simple_binding_name(&paren.pat),
        Pat::Type(typed) => simple_binding_name(&typed.pat),
        _ => None,
    }
}

pub(super) fn binding_names(pattern: &Pat) -> BTreeSet<String> {
    let mut collector = BindingNameCollector::default();
    collector.visit_pat(pattern);
    collector.names
}

#[derive(Default)]
struct BindingNameCollector {
    names: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for BindingNameCollector {
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.names.insert(pattern.ident.to_string());
        syn::visit::visit_pat_ident(self, pattern);
    }
}
