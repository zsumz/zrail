//! Attribute facts retain hygiene and macro-expansion trust independently.

use syn::{Attribute, spanned::Spanned};
use zrail_core::AnalysisQuality;

use super::{
    FactVisitor, MacroExpansionFact,
    attributes::{lint_suppression_effects, unsafe_attribute_effects},
    fact::fact,
};

impl FactVisitor<'_> {
    pub(in crate::source) fn record_attribute(&mut self, attribute: &Attribute) {
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
        let enclosing = self.syntax_guard();
        self.lint_suppressions
            .extend(
                lint_suppression_effects(attribute)
                    .into_iter()
                    .map(|effect| {
                        let mut observed = fact(
                            if effect.reasoned {
                                "reasoned lint suppression"
                            } else {
                                "unreasoned lint suppression"
                            },
                            attribute.span(),
                            AnalysisQuality::Exact,
                        );
                        observed.guard = enclosing.combine(effect.guard);
                        observed
                    }),
            );
    }

    fn record_unsafe_attributes(&mut self, attribute: &Attribute) {
        let enclosing = self.syntax_guard();
        self.unsafe_constructs
            .extend(
                unsafe_attribute_effects(attribute)
                    .into_iter()
                    .map(|effect| {
                        let mut observed = fact(
                            format!("unsafe attribute {}", effect.name),
                            attribute.span(),
                            AnalysisQuality::Exact,
                        );
                        observed.guard = enclosing.combine(effect.guard);
                        observed
                    }),
            );
    }

    fn record_attribute_expansions(&mut self, attribute: &Attribute) {
        let Ok(expansions) = super::macro_expansion::attribute_paths(attribute) else {
            let name = format!(
                "unparsed attribute {}",
                attribute
                    .path()
                    .segments
                    .last()
                    .map_or("<empty>".into(), |segment| segment.ident.to_string())
            );
            self.macro_expansions
                .push(MacroExpansionFact::unresolved(fact(
                    &name,
                    attribute.span(),
                    AnalysisQuality::Unresolved,
                )));
            self.record_opaque_attribute(
                attribute.path(),
                attribute.span(),
                &super::SyntaxGuard::Ordinary,
            );
            return;
        };
        for expansion in expansions {
            let (resolved, quality, _, _) = self.resolve_macro_path(&expansion.path);
            let observed = fact(
                expansion
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::"),
                expansion.path.span(),
                AnalysisQuality::Exact,
            );
            let compiler_derive = expansion.kind == super::macro_expansion::ExpansionKind::Derive
                && quality == AnalysisQuality::Exact
                && super::macro_expansion::is_compiler_derive(&expansion.path, &resolved);
            if !compiler_derive {
                self.record_opaque_attribute(
                    &expansion.path,
                    expansion.path.span(),
                    &expansion.guard,
                );
            }
            let mut invocation = self.macro_invocation(&expansion.path);
            if expansion.kind == super::macro_expansion::ExpansionKind::Derive
                && super::macro_expansion::is_builtin_derive(&expansion.path)
            {
                invocation.mark_builtin_derive_syntax();
            }
            if compiler_derive {
                invocation.observation = observed;
                invocation.bind_compiler_candidate(&resolved);
            }
            invocation.apply_guard(&expansion.guard);
            self.macro_expansions.push(invocation);
        }
    }

    fn record_opaque_attribute(
        &mut self,
        path: &syn::Path,
        span: proc_macro2::Span,
        effect_guard: &super::SyntaxGuard,
    ) {
        let mut opaque = fact(
            path.segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
            span,
            AnalysisQuality::Unresolved,
        );
        opaque.guard = self.syntax_guard().combine(effect_guard);
        opaque.lexical_scope.clone_from(&self.lexical_scope);
        self.opaque_binding_macros.push(opaque);
    }
}
