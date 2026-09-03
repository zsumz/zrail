//! Positive export edges converge to a bounded least fixed point.

use std::collections::{BTreeMap, BTreeSet};

use super::edges::{
    apply_glob_edge, insert_named, insert_unknown, limit_reason, propagate_named_unknown,
    unknown_glob, unknown_named,
};
use super::paths::split_target;
use super::{
    ExportedMacro, GlobExport, MacroExportSet, MacroExports, ModuleDraft, ModuleResolution,
    NamedExport, UnknownExport, macro_symbol, merge_export,
};

impl MacroExports {
    pub(super) fn close(&mut self, mut drafts: BTreeMap<super::LogicalModule, ModuleDraft>) {
        for draft in drafts.values_mut() {
            draft.named.sort();
            draft.named.dedup();
            draft.globs.sort();
            draft.globs.dedup();
        }
        for (module, draft) in &drafts {
            let set = self.sets.entry(module.clone()).or_default();
            set.unknown.extend(draft.unknown.iter().cloned());
            for (name, exports) in &draft.direct {
                for exported in exports {
                    if merge_export(set, name.clone(), exported.clone()).is_err() {
                        set.unknown
                            .insert(UnknownExport::unbounded(limit_reason(module)));
                    }
                }
            }
        }
        let limit = drafts.len().saturating_add(1).max(1);
        for _ in 0..limit {
            let previous = self.sets.clone();
            let mut changed = false;
            for (module, draft) in &drafts {
                let mut next = previous.get(module).cloned().unwrap_or_default();
                for edge in &draft.named {
                    changed |= self.apply_named(module, draft, edge, &previous, &mut next);
                }
                for edge in &draft.globs {
                    changed |= self.apply_glob(module, edge, &previous, &mut next);
                }
                self.sets.insert(module.clone(), next);
            }
            if !changed {
                return;
            }
        }
        for (module, set) in &mut self.sets {
            set.unknown.insert(UnknownExport::unbounded(format!(
                "macro export closure did not converge for {}",
                module.display_path()
            )));
        }
    }

    fn apply_named(
        &self,
        module: &super::LogicalModule,
        draft: &ModuleDraft,
        edge: &NamedExport,
        sets: &BTreeMap<super::LogicalModule, MacroExportSet>,
        output: &mut MacroExportSet,
    ) -> bool {
        let (target_module, target_name) = split_target(&edge.target);
        if target_module.is_empty() {
            if let Some(exports) = draft.local.get(target_name) {
                return insert_named(output, edge, exports, true);
            }
            if let Some(exports) = sets
                .get(module)
                .and_then(|set| set.macros.get(&macro_symbol(target_name)))
            {
                return insert_named(output, edge, exports, false);
            }
            return sets.get(module).is_some_and(|set| {
                propagate_named_unknown(output, edge, set, module, target_name)
            });
        }
        match self.resolve_module(module, target_module) {
            ModuleResolution::Local { modules } => {
                let mut changed = false;
                for target in modules {
                    let Some(set) = sets.get(&target) else {
                        continue;
                    };
                    changed |= propagate_named_unknown(output, edge, set, module, target_name);
                    let Some(exports) = set.macros.get(&macro_symbol(target_name)) else {
                        continue;
                    };
                    let visible = exports
                        .iter()
                        .filter(|exported| exported.visible_from(module))
                        .cloned()
                        .map(|mut exported| {
                            if exported.proc_macro
                                && edge.visibility.is_public()
                                && target.domain.package != module.domain.package
                            {
                                exported.origins = self.repository_origin(module);
                            }
                            exported
                        })
                        .collect::<BTreeSet<_>>();
                    changed |= insert_named(output, edge, &visible, false);
                }
                changed
            }
            ModuleResolution::External { origins, .. } => insert_named(
                output,
                edge,
                &BTreeSet::from([ExportedMacro {
                    origins,
                    proc_macro: false,
                    authority_name: Some(edge.target.clone()),
                    definition: None,
                    definition_name: None,
                    definition_sha256: None,
                    guard: edge.guard.clone(),
                    quality: edge.quality,
                    visibility: super::ExportVisibility::default(),
                }]),
                true,
            ),
            ModuleResolution::Missing => false,
            ModuleResolution::Unknown(reason) => {
                insert_unknown(output, [unknown_named(edge, reason)])
            }
        }
    }

    fn apply_glob(
        &self,
        module: &super::LogicalModule,
        edge: &GlobExport,
        sets: &BTreeMap<super::LogicalModule, MacroExportSet>,
        output: &mut MacroExportSet,
    ) -> bool {
        match self.resolve_module(module, &edge.target) {
            ModuleResolution::Local { modules } => {
                let mut changed = false;
                for target in modules {
                    let Some(set) = sets.get(&target) else {
                        continue;
                    };
                    changed |= insert_unknown(
                        output,
                        set.unknown
                            .iter()
                            .filter(|unknown| unknown.visible_from(module))
                            .map(|unknown| unknown.through_glob(edge)),
                    );
                    for ((name, _), exports) in &set.macros {
                        for exported in exports
                            .iter()
                            .filter(|exported| exported.visible_from(module))
                        {
                            let exported = apply_glob_edge(exported, edge);
                            match merge_export(output, name.clone(), exported) {
                                Ok(inserted) => changed |= inserted,
                                Err(()) => {
                                    changed |= insert_unknown(
                                        output,
                                        [UnknownExport::unbounded(limit_reason(module))],
                                    );
                                }
                            }
                        }
                    }
                }
                changed
            }
            ModuleResolution::External {
                origins,
                module: Some(target),
            } => self.apply_external_glob(module, edge, output, &origins, &target),
            ModuleResolution::External { module: None, .. } => insert_unknown(
                output,
                [unknown_glob(
                    edge,
                    format!(
                        "macro export set for external glob {:?} is unknown",
                        edge.target
                    ),
                )],
            ),
            ModuleResolution::Missing => insert_unknown(
                output,
                [unknown_glob(
                    edge,
                    format!(
                        "macro export target {:?} is not an analyzed module",
                        edge.target
                    ),
                )],
            ),
            ModuleResolution::Unknown(reason) => {
                insert_unknown(output, [unknown_glob(edge, reason)])
            }
        }
    }
}
