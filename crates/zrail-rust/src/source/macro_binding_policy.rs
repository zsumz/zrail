//! Reviewed macro occurrences may remove only the opacity they introduced.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::SourceSpan;

use super::{ImportBindingFact, MacroExpansionFact, ObservedFact, SourceIndex, SyntaxGuard};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct MacroOccurrence {
    pub(crate) span: Option<SourceSpan>,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
}

impl MacroOccurrence {
    pub(super) fn new(span: SourceSpan, guard: SyntaxGuard, lexical_scope: &[SourceSpan]) -> Self {
        Self {
            span: Some(span),
            guard,
            lexical_scope: lexical_scope.to_vec(),
        }
    }

    fn from_expansion(expansion: &MacroExpansionFact) -> Self {
        Self {
            span: expansion.span,
            guard: expansion.guard,
            lexical_scope: expansion.lexical_scope.clone(),
        }
    }

    fn from_fact(fact: &ObservedFact) -> Self {
        Self {
            span: fact.span,
            guard: fact.guard,
            lexical_scope: fact.lexical_scope.clone(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct BindingMacroPolicy {
    reviewed: BTreeMap<String, BTreeSet<MacroOccurrence>>,
}

impl BindingMacroPolicy {
    pub(crate) fn trust(&mut self, file: &str, expansion: &MacroExpansionFact) {
        self.reviewed
            .entry(file.into())
            .or_default()
            .insert(MacroOccurrence::from_expansion(expansion));
    }

    pub(super) fn apply(&self, index: &mut SourceIndex) {
        for file in &mut index.files {
            for binding in &mut file.import_bindings {
                self.restore_binding(&file.relative, binding);
            }
        }
    }

    pub(crate) fn retains_opacity(&self, file: &str, fact: &ObservedFact) -> bool {
        !self.covers(file, &MacroOccurrence::from_fact(fact))
    }

    fn restore_binding(&self, file: &str, binding: &mut ImportBindingFact) {
        if !binding.replacement_macros.is_empty()
            && binding
                .replacement_macros
                .iter()
                .all(|occurrence| self.covers(file, occurrence))
        {
            binding.quality = binding.quality_without_macros;
        }
    }

    fn covers(&self, file: &str, occurrence: &MacroOccurrence) -> bool {
        self.reviewed
            .get(file)
            .is_some_and(|reviewed| reviewed.contains(occurrence))
    }
}
