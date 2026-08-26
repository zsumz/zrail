//! Binding collectors derive explicit and match-ergonomic reference modes.

use std::collections::BTreeMap;

use syn::{Pat, visit::Visit};

use super::{PatternInputMode, syntactic_input_from_type};

pub(in crate::source) fn binding_input_modes(
    pattern: &Pat,
    input: PatternInputMode,
) -> BTreeMap<String, PatternInputMode> {
    let mut collector = BindingInputCollector {
        input,
        bindings: BTreeMap::new(),
    };
    collector.visit_pat(pattern);
    collector.bindings
}

struct BindingInputCollector {
    input: PatternInputMode,
    bindings: BTreeMap<String, PatternInputMode>,
}

impl BindingInputCollector {
    fn record(&mut self, name: String, input: PatternInputMode) {
        self.bindings
            .entry(name)
            .and_modify(|existing| {
                if *existing != input {
                    *existing = PatternInputMode::Unresolved;
                }
            })
            .or_insert(input);
    }

    fn with_input(&mut self, input: PatternInputMode, visit: impl FnOnce(&mut Self)) {
        let previous = std::mem::replace(&mut self.input, input);
        visit(self);
        self.input = previous;
    }
}

impl<'ast> Visit<'ast> for BindingInputCollector {
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        let input = match (pattern.by_ref, pattern.mutability) {
            (Some(_), Some(_)) => PatternInputMode::MutableReference,
            (Some(_), None) => PatternInputMode::SharedReference,
            (None, _) => self.input,
        };
        self.record(pattern.ident.to_string(), input);
        if let Some((_, subpattern)) = &pattern.subpat {
            self.visit_pat(subpattern);
        }
    }

    fn visit_pat_reference(&mut self, pattern: &'ast syn::PatReference) {
        self.with_input(PatternInputMode::Value, |collector| {
            collector.visit_pat(&pattern.pat);
        });
    }

    fn visit_pat_type(&mut self, pattern: &'ast syn::PatType) {
        let input = syntactic_input_from_type(&pattern.ty);
        self.with_input(input, |collector| collector.visit_pat(&pattern.pat));
    }
}
