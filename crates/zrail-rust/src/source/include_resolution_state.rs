//! Resolution state keeps lookup, consumer, and occurrence identity separate.

use std::collections::BTreeSet;

use zrail_core::SourceSpan;

use super::{SourceInstanceId, SyntaxGuard};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ModuleBoundary {
    External(SourceInstanceId),
    Inline(SourceInstanceId, SourceSpan),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct EffectiveModule {
    pub(super) root: SourceInstanceId,
    pub(super) boundaries: Vec<ModuleBoundary>,
    pub(super) names: Vec<String>,
}

impl EffectiveModule {
    pub(super) fn contains(&self, module: &Self) -> bool {
        self.root == module.root && self.boundaries.starts_with(&module.boundaries)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LookupKind {
    Lexical,
    Module,
    ExternRoot,
}

#[derive(Clone, Debug)]
pub(super) struct LookupMode {
    pub(super) kind: LookupKind,
    pub(super) consumer: EffectiveModule,
    pub(super) speculative: bool,
}

impl LookupMode {
    pub(super) const fn exact_scope(&self) -> bool {
        matches!(self.kind, LookupKind::Module | LookupKind::ExternRoot)
    }

    pub(super) const fn extern_root(&self) -> bool {
        matches!(self.kind, LookupKind::ExternRoot)
    }

    pub(super) fn lexical(consumer: EffectiveModule) -> Self {
        Self {
            kind: LookupKind::Lexical,
            consumer,
            speculative: false,
        }
    }

    pub(super) fn module(&self) -> Self {
        Self {
            kind: LookupKind::Module,
            consumer: self.consumer.clone(),
            speculative: self.speculative,
        }
    }

    pub(super) fn explicit_extern(consumer: EffectiveModule) -> Self {
        Self {
            kind: LookupKind::ExternRoot,
            consumer,
            speculative: false,
        }
    }

    pub(super) fn binding_target(module: EffectiveModule, speculative: bool) -> Self {
        Self {
            kind: LookupKind::Lexical,
            consumer: module,
            speculative,
        }
    }

    pub(super) fn glob_target(module: EffectiveModule) -> Self {
        Self {
            kind: LookupKind::Lexical,
            consumer: module,
            speculative: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ResolutionUsage {
    Path,
    Type,
    OperationType,
    ConstructorValue,
    Call,
}

pub(super) struct ResolveRequest<'a> {
    pub(super) instance: SourceInstanceId,
    pub(super) written: &'a str,
    pub(super) scope: &'a [SourceSpan],
    pub(super) depth: usize,
    pub(super) mode: LookupMode,
    pub(super) usage: ResolutionUsage,
    pub(super) guard: SyntaxGuard,
}

pub(super) struct WrittenResolveRequest<'a> {
    pub(super) instance: SourceInstanceId,
    pub(super) written: &'a str,
    pub(super) scope: &'a [SourceSpan],
    pub(super) depth: usize,
    pub(super) usage: ResolutionUsage,
    pub(super) guard: &'a SyntaxGuard,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ResolutionKey {
    Alias {
        instance: SourceInstanceId,
        name: String,
        scope: Vec<SourceSpan>,
    },
    Glob {
        instance: SourceInstanceId,
        target: String,
        scope: Vec<SourceSpan>,
    },
}

pub(super) type ResolutionTrail = BTreeSet<ResolutionKey>;
