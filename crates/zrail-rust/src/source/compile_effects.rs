//! Compiler intrinsics expose environment and filesystem effects explicitly.

use proc_macro2::TokenStream;
use syn::Macro;
use zrail_core::{AnalysisQuality, Effect};

use super::{MacroExpansionFact, visitor::FactVisitor};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompileEffectFact {
    pub(crate) effect: Effect,
    pub(crate) invocation: MacroExpansionFact,
    pub(crate) target: Option<String>,
    pub(crate) opaque_input: bool,
}

pub(super) fn record(
    visitor: &mut FactVisitor<'_>,
    invocation: &Macro,
    observed: &MacroExpansionFact,
) {
    record_tokens(visitor, invocation.tokens.clone(), observed, false);
}

pub(super) fn record_tokens(
    visitor: &mut FactVisitor<'_>,
    tokens: TokenStream,
    observed: &MacroExpansionFact,
    opaque_input: bool,
) {
    if observed.quality != AnalysisQuality::Exact {
        return;
    }
    let leaf = observed.name.rsplit("::").next().unwrap_or(&observed.name);
    let effect = match leaf {
        "env" | "option_env" => Effect::CompileEnvironment,
        "include" | "include_str" | "include_bytes" => Effect::CompileFilesystem,
        _ => return,
    };
    visitor.compile_effects.push(CompileEffectFact {
        effect,
        invocation: observed.clone(),
        target: syn::parse2::<syn::LitStr>(tokens)
            .ok()
            .map(|literal| literal.value()),
        opaque_input,
    });
}
