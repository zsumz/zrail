//! Lexically scoped typed values support exact receiver identities without type guessing.

use std::collections::BTreeMap;

use syn::{FnArg, Local, Pat, Signature};

use super::{FactVisitor, operation_model::TypeIdentity};

impl FactVisitor<'_> {
    pub(in crate::source) fn with_value_scope(&mut self, visit: impl FnOnce(&mut Self)) {
        self.local_values.push(BTreeMap::new());
        visit(self);
        self.local_values.pop();
    }

    pub(in crate::source) fn with_signature_values(
        &mut self,
        signature: &Signature,
        visit: impl FnOnce(&mut Self),
    ) {
        let values = signature
            .inputs
            .iter()
            .filter_map(|argument| match argument {
                FnArg::Typed(argument) => {
                    binding_name(&argument.pat).map(|name| (name, self.resolve_type(&argument.ty)))
                }
                FnArg::Receiver(_) => None,
            })
            .collect();
        self.local_values.push(values);
        visit(self);
        self.local_values.pop();
    }

    pub(in crate::source) fn with_closure_values(
        &mut self,
        inputs: &syn::punctuated::Punctuated<Pat, syn::Token![,]>,
        visit: impl FnOnce(&mut Self),
    ) {
        let values = inputs
            .iter()
            .filter_map(|pattern| {
                let Pat::Type(typed) = pattern else {
                    return None;
                };
                binding_name(&typed.pat).map(|name| (name, self.resolve_type(&typed.ty)))
            })
            .collect();
        self.local_values.push(values);
        visit(self);
        self.local_values.pop();
    }

    pub(in crate::source) fn record_typed_local(&mut self, local: &Local) {
        let Pat::Type(typed) = &local.pat else {
            return;
        };
        let Some(name) = binding_name(&typed.pat) else {
            return;
        };
        let identity = self.resolve_type(&typed.ty);
        if let Some(scope) = self.local_values.last_mut() {
            scope.insert(name, identity);
        }
    }

    pub(in crate::source) fn local_value_identity(&self, name: &str) -> Option<TypeIdentity> {
        self.local_values
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }
}

fn binding_name(pattern: &Pat) -> Option<String> {
    match pattern {
        Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        Pat::Reference(reference) => binding_name(&reference.pat),
        Pat::Paren(paren) => binding_name(&paren.pat),
        Pat::Type(typed) => binding_name(&typed.pat),
        _ => None,
    }
}
