//! Macro import lookup reads only the macro namespace of an export target.

use std::collections::BTreeSet;

use zrail_core::AnalysisQuality;

use super::super::{GuardAvailability, MacroCandidate, MacroDerivation, MacroOrigin, SyntaxGuard};
use super::paths::split_target;
use super::{
    ExportVisibility, ExportedMacro, MacroExports, ModuleResolution, MountedModule, macro_symbol,
};

#[derive(Default)]
pub(super) struct ContextResolution {
    pub(super) candidates: Vec<MacroCandidate>,
    pub(super) unknown: BTreeSet<String>,
    pub(super) resolved: bool,
}

impl MacroExports {
    pub(super) fn resolve_import(
        &self,
        candidate: &MacroCandidate,
        context: &MountedModule,
    ) -> ContextResolution {
        let (module_path, name) = split_target(&candidate.observation.name);
        match self.resolve_module(&context.module, module_path) {
            ModuleResolution::Local { modules } => {
                self.resolve_local(candidate, context, name, modules)
            }
            ModuleResolution::External { origins, .. }
                if candidate.derivation != MacroDerivation::GlobImport =>
            {
                external(
                    candidate,
                    context,
                    origins,
                    candidate.observation.quality.max(AnalysisQuality::Exact),
                    self,
                )
            }
            ModuleResolution::External {
                origins,
                module: Some(module),
            } => self.resolve_external_glob(candidate, context, origins, &module, name),
            ModuleResolution::External { module: None, .. } => unknown(format!(
                "macro export set for external glob {module_path:?} is unknown"
            )),
            ModuleResolution::Missing if candidate.derivation == MacroDerivation::GlobImport => {
                unknown(format!(
                    "macro export target {module_path:?} is not an analyzed module"
                ))
            }
            ModuleResolution::Missing => ContextResolution::default(),
            ModuleResolution::Unknown(reason) => unknown(reason),
        }
    }

    fn resolve_external_glob(
        &self,
        candidate: &MacroCandidate,
        context: &MountedModule,
        origins: Vec<MacroOrigin>,
        module: &super::ExternalModule,
        name: &str,
    ) -> ContextResolution {
        let exports = match self.external.module(module) {
            Ok(exports) => exports,
            Err(reason) => return unknown(reason),
        };
        if let Some(reason) = exports.uncertain.get(name).or(exports.open.as_ref()) {
            return unknown(reason.clone());
        }
        if exports.macros.contains(name) {
            return external(candidate, context, origins, AnalysisQuality::Exact, self);
        }
        ContextResolution::default()
    }

    fn resolve_local(
        &self,
        candidate: &MacroCandidate,
        context: &MountedModule,
        name: &str,
        modules: BTreeSet<super::LogicalModule>,
    ) -> ContextResolution {
        let invocation_guard = context.guard.combine(&candidate.observation.guard);
        let mut resolved = ContextResolution::default();
        for module in modules {
            let Some(set) = self.sets.get(&module) else {
                resolved.unknown.insert(format!(
                    "macro export set for {} is unavailable",
                    module.display_path()
                ));
                continue;
            };
            resolved.unknown.extend(
                set.unknown
                    .iter()
                    .filter(|unknown| {
                        unknown.matches(name)
                            && unknown.visible_from(&context.module)
                            && unknown.active_for(&context.module, &invocation_guard)
                    })
                    .map(|unknown| unknown.reason().to_owned()),
            );
            let Some(exports) = set.macros.get(&macro_symbol(name)) else {
                continue;
            };
            for exported in exports
                .iter()
                .filter(|exported| exported.visible_from(&context.module))
            {
                let availability = exported
                    .guard
                    .combine(&invocation_guard)
                    .availability_in_domain(&context.module.domain);
                if availability == GuardAvailability::Absent {
                    continue;
                }
                resolved.candidates.push(super::resolve::resolved_candidate(
                    candidate,
                    exported,
                    availability,
                    &module,
                    self,
                ));
                resolved.resolved = true;
            }
        }
        resolved
    }
}

fn external(
    candidate: &MacroCandidate,
    context: &MountedModule,
    origins: Vec<MacroOrigin>,
    quality: AnalysisQuality,
    exports: &MacroExports,
) -> ContextResolution {
    let availability = candidate
        .observation
        .guard
        .combine(&context.guard)
        .availability_in_domain(&context.module.domain);
    if availability == GuardAvailability::Absent {
        return ContextResolution::default();
    }
    let exported = ExportedMacro {
        origins,
        proc_macro: false,
        authority_name: Some(candidate.observation.name.clone()),
        definition: None,
        definition_name: None,
        definition_sha256: None,
        guard: SyntaxGuard::Ordinary,
        quality,
        visibility: ExportVisibility::default(),
    };
    ContextResolution {
        candidates: vec![super::resolve::resolved_candidate(
            candidate,
            &exported,
            availability,
            &context.module,
            exports,
        )],
        resolved: true,
        ..ContextResolution::default()
    }
}

fn unknown(reason: String) -> ContextResolution {
    ContextResolution {
        unknown: BTreeSet::from([reason]),
        ..ContextResolution::default()
    }
}
