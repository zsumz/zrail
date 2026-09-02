//! Macro definitions retain their export class and canonical implementation digest.

use zrail_core::SourceSpan;

use super::SyntaxGuard;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MacroDefinitionFact {
    pub(crate) name: String,
    pub(crate) sha256: String,
    pub(crate) export: MacroDefinitionExport,
    pub(crate) span: Option<SourceSpan>,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MacroDefinitionExport {
    Lexical,
    CrateRoot,
    ProcMacro,
}

impl MacroDefinitionFact {
    pub(in crate::source) fn apply_guard(&mut self, guard: &SyntaxGuard) {
        self.guard = self.guard.combine(guard);
    }
}
