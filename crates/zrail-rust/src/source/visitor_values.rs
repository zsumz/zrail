//! Guarded lexical bindings prevent stale receiver identities across shadows and feature worlds.

use std::collections::{BTreeMap, BTreeSet};

use syn::{FnArg, Local, Pat, Signature, visit::Visit};
use zrail_core::AnalysisQuality;

use crate::source::CfgPredicate;

use super::{
    FactVisitor, SyntaxGuard,
    attributes::cfg_guard,
    operation_model::{TypeIdentity, unresolved},
};

pub(in crate::source) type LocalValueScopes = Vec<BTreeMap<String, Vec<GuardedValueBinding>>>;

#[derive(Clone, Debug)]
pub(in crate::source) struct ValueCandidate {
    pub(in crate::source) identity: TypeIdentity,
    pub(in crate::source) guard: SyntaxGuard,
}

#[derive(Clone, Debug)]
pub(in crate::source) struct GuardedValueBinding {
    value: ValueBinding,
    guard: SyntaxGuard,
}

#[derive(Clone, Debug)]
enum ValueBinding {
    Exact(TypeIdentity),
    Candidates(Vec<TypeIdentity>),
    Unresolved(TypeIdentity),
}

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
        self.local_values.push(BTreeMap::new());
        for argument in &signature.inputs {
            if let FnArg::Typed(argument) = argument {
                let guard = self.syntax_guard().combine(cfg_guard(&argument.attrs));
                self.install_pattern(&argument.pat, Some(&argument.ty), &guard);
            }
        }
        visit(self);
        self.local_values.pop();
    }

    pub(in crate::source) fn with_closure_values(
        &mut self,
        inputs: &syn::punctuated::Punctuated<Pat, syn::Token![,]>,
        visit: impl FnOnce(&mut Self),
    ) {
        self.local_values.push(BTreeMap::new());
        for pattern in inputs {
            let (pattern, ty) = typed_pattern(pattern);
            let guard = self.syntax_guard();
            self.install_pattern(pattern, ty, &guard);
        }
        visit(self);
        self.local_values.pop();
    }

    pub(in crate::source) fn with_pattern_values(
        &mut self,
        pattern: &Pat,
        visit: impl FnOnce(&mut Self),
    ) {
        self.local_values.push(BTreeMap::new());
        let (pattern, ty) = typed_pattern(pattern);
        let guard = self.syntax_guard();
        self.install_pattern(pattern, ty, &guard);
        visit(self);
        self.local_values.pop();
    }

    pub(in crate::source) fn record_local_bindings(&mut self, local: &Local) {
        let (pattern, ty) = typed_pattern(&local.pat);
        let guard = self.syntax_guard();
        self.install_pattern(pattern, ty, &guard);
    }

    pub(in crate::source) fn local_value_candidates(&self, name: &str) -> Vec<ValueCandidate> {
        let mut candidates = Vec::new();
        let mut shadowed = CfgPredicate::False;
        for scope in self.local_values.iter().rev() {
            let Some(bindings) = scope.get(name) else {
                continue;
            };
            for binding in bindings.iter().rev() {
                let effective = CfgPredicate::all(vec![
                    binding.guard.predicate(),
                    CfgPredicate::not(shadowed.clone()),
                ]);
                if effective.is_satisfiable() != Some(false) {
                    expand_binding(
                        binding,
                        SyntaxGuard::from_predicate(effective),
                        &mut candidates,
                    );
                }
                shadowed = CfgPredicate::any(vec![shadowed, binding.guard.predicate()]);
            }
        }
        let uncovered = CfgPredicate::not(shadowed);
        if uncovered.is_satisfiable() != Some(false) {
            candidates.push(ValueCandidate {
                identity: unresolved("<unresolved>"),
                guard: SyntaxGuard::from_predicate(uncovered),
            });
        }
        candidates
    }

    fn install_pattern(&mut self, pattern: &Pat, ty: Option<&syn::Type>, guard: &SyntaxGuard) {
        let names = binding_names(pattern);
        let exact_name = ty.and_then(|_| simple_binding_name(pattern));
        let exact = ty.map(|ty| binding_from_identity(self.resolve_type(ty)));
        let Some(scope) = self.local_values.last_mut() else {
            return;
        };
        for name in names {
            let value = if exact_name.as_deref() == Some(name.as_str()) {
                exact
                    .clone()
                    .unwrap_or_else(|| ValueBinding::Unresolved(unresolved("<unresolved>")))
            } else {
                ValueBinding::Unresolved(unresolved("<unresolved>"))
            };
            scope.entry(name).or_default().push(GuardedValueBinding {
                value,
                guard: guard.clone(),
            });
        }
    }
}

fn binding_from_identity(identity: TypeIdentity) -> ValueBinding {
    match identity.quality {
        AnalysisQuality::Exact => ValueBinding::Exact(identity),
        AnalysisQuality::Conservative => ValueBinding::Candidates(vec![identity]),
        AnalysisQuality::Unresolved => ValueBinding::Unresolved(identity),
    }
}

fn expand_binding(
    binding: &GuardedValueBinding,
    guard: SyntaxGuard,
    candidates: &mut Vec<ValueCandidate>,
) {
    match &binding.value {
        ValueBinding::Exact(identity) | ValueBinding::Unresolved(identity) => {
            candidates.push(ValueCandidate {
                identity: identity.clone(),
                guard,
            });
        }
        ValueBinding::Candidates(identities) => {
            candidates.extend(identities.iter().map(|identity| {
                let mut identity = identity.clone();
                identity.quality = identity.quality.max(AnalysisQuality::Conservative);
                ValueCandidate {
                    identity,
                    guard: guard.clone(),
                }
            }));
        }
    }
}

fn typed_pattern(pattern: &Pat) -> (&Pat, Option<&syn::Type>) {
    match pattern {
        Pat::Type(typed) => (&typed.pat, Some(&typed.ty)),
        _ => (pattern, None),
    }
}

fn simple_binding_name(pattern: &Pat) -> Option<String> {
    match pattern {
        Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        Pat::Reference(reference) => simple_binding_name(&reference.pat),
        Pat::Paren(paren) => simple_binding_name(&paren.pat),
        Pat::Type(typed) => simple_binding_name(&typed.pat),
        _ => None,
    }
}

fn binding_names(pattern: &Pat) -> BTreeSet<String> {
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
