//! Implicit preludes are the final lexical binding layer, never module members.

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    IncludeBindings, ResolvedOrigin, ResolvedPath, ResolvedTerminal,
    implicit_prelude::PreludeDirectiveKind,
    implicit_prelude_catalog::{self, PreludeEntry, PreludeItemKind},
};
use crate::source::{
    ConstructorForm, GuardAvailability, SourceEntry, SourceInstanceId, SyntaxGuard,
    include_binding_helpers::{join, split_root},
    include_resolution_state::{LookupMode, ResolutionUsage},
};

impl IncludeBindings {
    pub(in crate::source) fn extern_prelude_precedes_implicit(
        &self,
        request: &crate::source::include_resolution_state::ResolveRequest<'_>,
    ) -> bool {
        let Some(source) = self.instances.get(request.instance) else {
            return false;
        };
        if source.domain.edition == "2015" {
            return false;
        }
        let (root, suffix) = split_root(request.written);
        if !self.is_extern_root(request.instance, root) {
            return false;
        }
        if !suffix.is_empty()
            || matches!(
                request.usage,
                ResolutionUsage::Type | ResolutionUsage::OperationType
            )
        {
            return true;
        }
        implicit_prelude_catalog::core(root, &source.domain.edition)
            .or_else(|| implicit_prelude_catalog::std_only(root))
            .is_none_or(|entry| entry.kind == PreludeItemKind::Type)
    }

    pub(in crate::source) fn implicit_prelude_candidate(
        &self,
        instance: SourceInstanceId,
        written: &str,
        scope: &[SourceSpan],
        crossed_include: bool,
        mode: &LookupMode,
        usage: ResolutionUsage,
        guard: &SyntaxGuard,
    ) -> Option<ResolvedPath> {
        if mode.exact_scope() || mode.speculative {
            return None;
        }
        let source = self.instances.get(instance)?;
        let disabled = self.no_implicit_availability(instance, scope, guard);
        if disabled == GuardAvailability::Exact {
            return None;
        }
        let (root, suffix) = split_root(written);
        let no_std = self.no_std_availability(instance, guard);
        let mut conditional = disabled == GuardAvailability::Possible;
        let entry = implicit_prelude_catalog::core(root, &source.domain.edition).or_else(|| {
            if no_std == GuardAvailability::Exact {
                return None;
            }
            conditional |= no_std == GuardAvailability::Possible;
            implicit_prelude_catalog::std_only(root)
        })?;
        if !supports(entry, suffix, usage) {
            return None;
        }
        let name = join(entry.canonical, suffix)?;
        Some(ResolvedPath {
            name,
            quality: if conditional {
                AnalysisQuality::Unresolved
            } else {
                AnalysisQuality::Exact
            },
            crossed_include,
            requires_projection: true,
            blocks_completeness: conditional,
            origin: ResolvedOrigin::External,
            terminal: terminal(entry, suffix),
        })
    }

    fn no_implicit_availability(
        &self,
        instance: SourceInstanceId,
        scope: &[SourceSpan],
        guard: &SyntaxGuard,
    ) -> GuardAvailability {
        let Some(source) = self.instances.get(instance) else {
            return GuardAvailability::Possible;
        };
        let domain = source.domain.clone();
        let context = source.guard.combine(guard);
        let mut current = instance;
        let mut current_scope = scope.to_vec();
        let mut availability = GuardAvailability::Absent;
        loop {
            let Some(source) = self.instances.get(current) else {
                return GuardAvailability::Possible;
            };
            availability = merge(
                availability,
                self.directive_availability(
                    current,
                    &current_scope,
                    PreludeDirectiveKind::NoImplicit,
                    &context,
                    &domain,
                ),
            );
            if availability == GuardAvailability::Exact {
                return availability;
            }
            let Some(parent) = source.parent else {
                return availability;
            };
            current_scope = match &source.entered_from {
                SourceEntry::Include(edge) => edge.parent_scope.clone(),
                SourceEntry::Module(edge) => {
                    let mut scope = edge.parent_scope.clone();
                    if let Some(span) = edge.span {
                        scope.push(span);
                    }
                    scope
                }
                SourceEntry::CargoRoot => return availability,
            };
            current = parent;
        }
    }

    fn no_std_availability(
        &self,
        instance: SourceInstanceId,
        guard: &SyntaxGuard,
    ) -> GuardAvailability {
        let Some(source) = self.instances.get(instance) else {
            return GuardAvailability::Possible;
        };
        let domain = source.domain.clone();
        let context = source.guard.combine(guard);
        let mut root = instance;
        while let Some(parent) = self.instances.get(root).and_then(|source| source.parent) {
            root = parent;
        }
        self.instances
            .get(root)
            .map_or(GuardAvailability::Possible, |_| {
                self.directive_availability(
                    root,
                    &[],
                    PreludeDirectiveKind::NoStd,
                    &context,
                    &domain,
                )
            })
    }

    fn directive_availability(
        &self,
        instance: SourceInstanceId,
        scope: &[SourceSpan],
        kind: PreludeDirectiveKind,
        context: &SyntaxGuard,
        domain: &crate::source::CompilationDomain,
    ) -> GuardAvailability {
        self.prelude_directives
            .get(&instance)
            .into_iter()
            .flatten()
            .filter(|directive| {
                directive.kind == kind && scope.starts_with(&directive.lexical_scope)
            })
            .fold(GuardAvailability::Absent, |availability, directive| {
                merge(
                    availability,
                    directive.guard.availability_for_domain(context, domain),
                )
            })
    }
}

fn supports(entry: PreludeEntry, suffix: &str, usage: ResolutionUsage) -> bool {
    if !suffix.is_empty() {
        return entry.kind == PreludeItemKind::Type;
    }
    match entry.kind {
        PreludeItemKind::Type => !matches!(
            usage,
            ResolutionUsage::Call | ResolutionUsage::ConstructorValue
        ),
        PreludeItemKind::Value => {
            matches!(usage, ResolutionUsage::Path | ResolutionUsage::Call)
        }
        PreludeItemKind::TupleConstructor | PreludeItemKind::UnitConstructor => matches!(
            usage,
            ResolutionUsage::Path | ResolutionUsage::Call | ResolutionUsage::ConstructorValue
        ),
    }
}

fn terminal(entry: PreludeEntry, suffix: &str) -> ResolvedTerminal {
    if suffix.is_empty() {
        return match entry.kind {
            PreludeItemKind::Type => ResolvedTerminal::Type,
            PreludeItemKind::Value => ResolvedTerminal::Value,
            PreludeItemKind::TupleConstructor => {
                ResolvedTerminal::Constructor(ConstructorForm::Tuple)
            }
            PreludeItemKind::UnitConstructor => {
                ResolvedTerminal::Constructor(ConstructorForm::Unit)
            }
        };
    }
    match (entry.canonical, suffix) {
        ("core::option::Option", "::Some") | ("core::result::Result", "::Ok" | "::Err") => {
            ResolvedTerminal::Constructor(ConstructorForm::Tuple)
        }
        ("core::option::Option", "::None") => ResolvedTerminal::Constructor(ConstructorForm::Unit),
        _ => ResolvedTerminal::Unknown,
    }
}

const fn merge(left: GuardAvailability, right: GuardAvailability) -> GuardAvailability {
    match (left, right) {
        (GuardAvailability::Exact, _) | (_, GuardAvailability::Exact) => GuardAvailability::Exact,
        (GuardAvailability::Possible, _) | (_, GuardAvailability::Possible) => {
            GuardAvailability::Possible
        }
        _ => GuardAvailability::Absent,
    }
}
