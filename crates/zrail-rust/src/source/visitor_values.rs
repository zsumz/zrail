//! Guarded lexical bindings prevent stale receiver identities across shadows and feature worlds.

use std::collections::BTreeMap;

use syn::{FnArg, Local, Pat, Signature};

use crate::source::CfgPredicate;

use super::{
    FactVisitor, SyntaxGuard,
    attributes::cfg_guard,
    operation_model::{TypeIdentity, unresolved},
    visitor_patterns::{PatternInputMode, binding_input_modes},
};

#[path = "visitor_value_candidates.rs"]
mod candidates;
#[path = "visitor_value_patterns.rs"]
mod patterns;

use candidates::{binding_from_identity, expand_binding};
use patterns::{binding_names, simple_binding_name, typed_pattern};

pub(in crate::source) type LocalValueScopes = Vec<BTreeMap<String, Vec<GuardedValueBinding>>>;

#[derive(Clone, Debug)]
pub(in crate::source) struct ValueCandidate {
    pub(in crate::source) identity: TypeIdentity,
    pub(in crate::source) guard: SyntaxGuard,
    pub(in crate::source) input: PatternInputMode,
}

#[derive(Clone, Debug)]
pub(in crate::source) struct GuardedValueBinding {
    value: ValueBinding,
    guard: SyntaxGuard,
    input: PatternInputMode,
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
                let input = self.pattern_input_from_type(&argument.ty);
                self.install_pattern(&argument.pat, Some(&argument.ty), input, &guard);
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
            let input = ty.map_or(PatternInputMode::Unresolved, |ty| {
                self.pattern_input_from_type(ty)
            });
            self.install_pattern(pattern, ty, input, &guard);
        }
        visit(self);
        self.local_values.pop();
    }

    pub(in crate::source) fn with_pattern_values(
        &mut self,
        pattern: &Pat,
        input: PatternInputMode,
        visit: impl FnOnce(&mut Self),
    ) {
        let checkpoint = self.value_scope_checkpoint();
        self.push_pattern_values(pattern, input);
        visit(self);
        self.restore_value_scopes(checkpoint);
    }

    pub(in crate::source) fn record_local_bindings(
        &mut self,
        local: &Local,
        input: PatternInputMode,
    ) {
        let (pattern, ty) = typed_pattern(&local.pat);
        let guard = self.syntax_guard();
        let input = ty.map_or(input, |ty| self.pattern_input_from_type(ty));
        self.install_pattern(pattern, ty, input, &guard);
    }

    pub(in crate::source) fn value_scope_checkpoint(&self) -> usize {
        self.local_values.len()
    }

    pub(in crate::source) fn push_pattern_values(
        &mut self,
        pattern: &Pat,
        input: PatternInputMode,
    ) {
        self.local_values.push(BTreeMap::new());
        let (pattern, ty) = typed_pattern(pattern);
        let guard = self.syntax_guard();
        let input = ty.map_or(input, |ty| self.pattern_input_from_type(ty));
        self.install_pattern(pattern, ty, input, &guard);
    }

    pub(in crate::source) fn restore_value_scopes(&mut self, checkpoint: usize) {
        self.local_values.truncate(checkpoint);
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
                input: PatternInputMode::Unresolved,
            });
        }
        candidates
    }

    pub(in crate::source) fn local_value_shadow_guard(&self, name: &str) -> SyntaxGuard {
        SyntaxGuard::from_predicate(CfgPredicate::any(
            self.local_values
                .iter()
                .filter_map(|scope| scope.get(name))
                .flatten()
                .map(|binding| binding.guard.predicate())
                .collect(),
        ))
    }

    fn install_pattern(
        &mut self,
        pattern: &Pat,
        ty: Option<&syn::Type>,
        input: PatternInputMode,
        guard: &SyntaxGuard,
    ) {
        let names = binding_names(pattern);
        let inputs = binding_input_modes(pattern, input);
        let exact_name = ty.and_then(|_| simple_binding_name(pattern));
        let exact = ty.map(|ty| binding_from_identity(self.resolve_type(ty)));
        let Some(scope) = self.local_values.last_mut() else {
            return;
        };
        for name in names {
            let input = inputs
                .get(&name)
                .copied()
                .unwrap_or(PatternInputMode::Unresolved);
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
                input,
            });
        }
    }
}
