//! Authored trait/type pairs reuse type-policy facts and supplement omitted AST contexts.
//!
//! This index claims written path suffixes only. Nominal resolution, implementation
//! polarity, and compiler validity remain independent of this explicit syntax inventory.

use std::collections::BTreeMap;

use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};
use zrail_core::{AnalysisQuality, SourceSpan};

use crate::source::{
    ObservedFact,
    fact::{source_span, written_fact},
    type_policy_model::{TraitImplFact, TypePolicyFacts},
};

pub(super) fn collect(syntax: &syn::File, facts: &TypePolicyFacts) -> Vec<ObservedFact> {
    let mut collector = Collector {
        existing: facts
            .trait_impls
            .iter()
            .map(|fact| (fact.trait_span, fact))
            .collect(),
        implementations: Vec::new(),
    };
    collector.visit_file(syntax);
    collector.implementations
}

struct Collector<'a> {
    existing: BTreeMap<SourceSpan, &'a TraitImplFact>,
    implementations: Vec<ObservedFact>,
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if self.implementations.len() > super::MAX_FACTS_PER_FILE {
            return;
        }
        if let Some((_, path, _)) = &item.trait_
            && let Some(trait_segment) = path.segments.last()
            && let syn::Type::Path(ty) = item.self_ty.as_ref()
            && let Some(type_segment) = ty.path.segments.last()
        {
            let existing = self.existing.get(&source_span(path.span()));
            let trait_name = existing.map_or_else(
                || trait_segment.ident.to_string(),
                |fact| fact.trait_hint.clone(),
            );
            let identity = format!("{trait_name} for {}", type_segment.ident);
            let mut fact = written_fact(
                trait_name,
                identity,
                item.span(),
                AnalysisQuality::Conservative,
                existing.map_or(&[], |fact| fact.lexical_scope.as_slice()),
            );
            if let Some(existing) = existing {
                fact.guard = existing.guard.clone();
            }
            self.implementations.push(fact);
        }
        visit::visit_item_impl(self, item);
    }
}
