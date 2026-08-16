//! Compiler intrinsics expose environment and filesystem effects explicitly.

use syn::Macro;
use zrail_core::{AnalysisQuality, Effect};

use super::{ObservedFact, model::CompileEffectFact, visitor::FactVisitor};

pub(super) fn record(visitor: &mut FactVisitor<'_>, invocation: &Macro, observed: &ObservedFact) {
    if observed.quality != AnalysisQuality::Exact || observed.name.contains("::") {
        return;
    }
    let effect = match observed.name.as_str() {
        "env" | "option_env" => Effect::CompileEnvironment,
        "include" | "include_str" | "include_bytes" => Effect::CompileFilesystem,
        _ => return,
    };
    visitor.compile_effects.push(CompileEffectFact {
        effect,
        invocation: observed.clone(),
        target: syn::parse2::<syn::LitStr>(invocation.tokens.clone())
            .ok()
            .map(|literal| literal.value()),
    });
}
