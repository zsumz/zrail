//! Attribute facts retain hygiene and macro-expansion trust independently.

use syn::{Attribute, spanned::Spanned};
use zrail_core::AnalysisQuality;

use super::{
    attributes::{is_lint_suppression, lint_suppression_is_reasoned, unsafe_attribute_names},
    fact::fact,
    model::MacroExpansionFact,
    visitor::FactVisitor,
};

impl FactVisitor<'_> {
    pub(super) fn record_attribute(&mut self, attribute: &Attribute) {
        self.record_lint_suppression(attribute);
        self.record_unsafe_attributes(attribute);
        if attribute.path().is_ident("macro_use") {
            self.macro_expansions
                .push(MacroExpansionFact::unresolved(fact(
                    "macro_use",
                    attribute.span(),
                    AnalysisQuality::Unresolved,
                )));
        } else {
            self.record_attribute_expansions(attribute);
        }
    }

    fn record_lint_suppression(&mut self, attribute: &Attribute) {
        if is_lint_suppression(attribute) {
            self.lint_suppressions.push(fact(
                if lint_suppression_is_reasoned(attribute) {
                    "reasoned lint suppression"
                } else {
                    "unreasoned lint suppression"
                },
                attribute.span(),
                AnalysisQuality::Exact,
            ));
        }
    }

    fn record_unsafe_attributes(&mut self, attribute: &Attribute) {
        let quality = if attribute.path().is_ident("cfg_attr") {
            AnalysisQuality::Conservative
        } else {
            AnalysisQuality::Exact
        };
        self.unsafe_constructs
            .extend(unsafe_attribute_names(attribute).into_iter().map(|name| {
                fact(
                    format!("unsafe attribute {name}"),
                    attribute.span(),
                    quality,
                )
            }));
    }

    fn record_attribute_expansions(&mut self, attribute: &Attribute) {
        match super::macro_expansion::attribute_paths(attribute) {
            Ok(paths) => {
                for path in paths {
                    let (name, quality, scoped, local_module) = self.resolve_macro_path(&path);
                    let mut expansion = fact(name.clone(), path.span(), quality);
                    if local_module {
                        expansion.canonical.push(name.clone());
                    }
                    let mut facts = vec![MacroExpansionFact::pending(expansion, local_module)];
                    if !scoped {
                        facts.extend(
                            super::calls::candidates(&path, self.imports, &name)
                                .into_iter()
                                .map(|fact| MacroExpansionFact::pending(fact, false)),
                        );
                    }
                    self.macro_expansions.extend(facts);
                }
            }
            Err(()) => self
                .macro_expansions
                .push(MacroExpansionFact::unresolved(fact(
                    format!(
                        "unparsed attribute {}",
                        attribute
                            .path()
                            .segments
                            .last()
                            .map_or("<empty>".into(), |segment| segment.ident.to_string())
                    ),
                    attribute.span(),
                    AnalysisQuality::Unresolved,
                ))),
        }
    }
}
