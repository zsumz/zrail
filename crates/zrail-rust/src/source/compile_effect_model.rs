//! Compile-effect facts retain the macro invocation that created the effect.

use zrail_core::Effect;

use super::MacroExpansionFact;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompileEffectFact {
    pub(crate) effect: Effect,
    pub(crate) invocation: MacroExpansionFact,
    pub(crate) target: Option<String>,
    pub(crate) opaque_input: bool,
}
