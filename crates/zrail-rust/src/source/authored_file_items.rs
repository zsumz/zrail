//! Direct file items reuse the parsed syntax tree without changing module mounting or type identity.

use zrail_core::AnalysisQuality;

use crate::source::{ObservedFact, attributes::cfg_guard, fact::written_fact};

pub(super) fn collect(
    syntax: &syn::File,
    collect_modules: bool,
    collect_variants: bool,
) -> (Vec<ObservedFact>, Vec<ObservedFact>) {
    let mut modules = Vec::new();
    let mut variants = Vec::new();
    for item in &syntax.items {
        match item {
            syn::Item::Mod(module) if collect_modules => {
                let name = module.ident.to_string();
                let mut fact = written_fact(
                    name.clone(),
                    name,
                    module.ident.span(),
                    AnalysisQuality::Conservative,
                    &[],
                );
                fact.guard = cfg_guard(&module.attrs);
                modules.push(fact);
            }
            syn::Item::Enum(enumeration) if collect_variants => {
                let parent_guard = cfg_guard(&enumeration.attrs);
                for variant in &enumeration.variants {
                    let name = format!("{}::{}", enumeration.ident, variant.ident);
                    let mut fact = written_fact(
                        name.clone(),
                        name,
                        variant.ident.span(),
                        AnalysisQuality::Conservative,
                        &[],
                    );
                    fact.guard = parent_guard.combine(cfg_guard(&variant.attrs));
                    variants.push(fact);
                    if modules.len() + variants.len() > super::MAX_FACTS_PER_FILE {
                        return (modules, variants);
                    }
                }
            }
            _ => {}
        }
        if modules.len() + variants.len() > super::MAX_FACTS_PER_FILE {
            break;
        }
    }
    (modules, variants)
}
