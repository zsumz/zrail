//! Use declarations become guarded named or glob macro-export edges.

use std::collections::BTreeMap;

use zrail_core::AnalysisQuality;

use super::super::{
    BindingKind, RustFileFacts, SourceInstanceId, SourceInstances,
    logical_modules::{InlineModuleCatalog, LogicalModule, locate},
    source_instance::SourceInstance,
};
use super::{ExportVisibility, GlobExport, ModuleDraft, NamedExport, UnknownExport, visibility};

pub(super) fn collect(
    file: &RustFileFacts,
    instances: &SourceInstances,
    instance: SourceInstanceId,
    mount: &SourceInstance,
    inline: &InlineModuleCatalog,
    drafts: &mut BTreeMap<LogicalModule, ModuleDraft>,
) {
    for binding in &file.import_bindings {
        if !matches!(binding.kind, BindingKind::Import | BindingKind::Glob) {
            continue;
        }
        let Some(module) = locate(instances, instance, &binding.lexical_scope, inline) else {
            continue;
        };
        let guard = mount.guard.combine(&binding.guard);
        let visibility = match visibility(&binding.visibility, &module) {
            Ok(visibility) => visibility,
            Err(reason) => {
                let name = (binding.kind == BindingKind::Import)
                    .then(|| binding.name.clone())
                    .flatten();
                drafts
                    .entry(module)
                    .or_default()
                    .unknown
                    .insert(UnknownExport::new(
                        name,
                        reason,
                        guard,
                        ExportVisibility::default(),
                    ));
                continue;
            }
        };
        let quality = if binding.replacement_macros.is_empty() {
            binding.quality_without_macros
        } else {
            AnalysisQuality::Unresolved
        };
        let draft = drafts.entry(module).or_default();
        if binding.kind == BindingKind::Glob {
            draft.globs.push(GlobExport {
                target: binding.target.clone(),
                guard,
                visibility,
                quality,
            });
        } else if let Some(name) = &binding.name {
            draft.named.push(NamedExport {
                name: name.clone(),
                target: binding.target.clone(),
                guard,
                visibility,
                quality,
            });
        }
    }
}
