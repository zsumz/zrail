//! Export edges retain authority, visibility, cfg, and bounded unknown state.

use std::collections::BTreeSet;

use super::{ExportedMacro, GlobExport, MacroExportSet, NamedExport, UnknownExport, merge_export};

pub(super) fn insert_named(
    output: &mut MacroExportSet,
    edge: &NamedExport,
    exports: &BTreeSet<ExportedMacro>,
    defines_visibility: bool,
) -> bool {
    let mut changed = false;
    for exported in exports {
        let mut exported = exported.clone();
        exported.guard = exported.guard.combine(&edge.guard);
        exported.quality = exported.quality.max(edge.quality);
        if defines_visibility {
            exported.visibility.clone_from(&edge.visibility);
        } else {
            exported.visibility.restrict(&edge.visibility);
        }
        match merge_export(output, edge.name.clone(), exported) {
            Ok(inserted) => changed |= inserted,
            Err(()) => {
                changed |= insert_unknown(
                    output,
                    [unknown_named(edge, "macro export set exceeds limit".into())],
                );
            }
        }
    }
    changed
}

pub(super) fn apply_glob_edge(exported: &ExportedMacro, edge: &GlobExport) -> ExportedMacro {
    let mut exported = exported.clone();
    exported.guard = exported.guard.combine(&edge.guard);
    exported.quality = exported.quality.max(edge.quality);
    exported.visibility.restrict(&edge.visibility);
    exported
}

pub(super) fn propagate_named_unknown(
    output: &mut MacroExportSet,
    edge: &NamedExport,
    source: &MacroExportSet,
    consumer: &super::LogicalModule,
    target_name: &str,
) -> bool {
    insert_unknown(
        output,
        source
            .unknown
            .iter()
            .filter(|unknown| unknown.matches(target_name) && unknown.visible_from(consumer))
            .map(|unknown| unknown.through_named(edge)),
    )
}

pub(super) fn unknown_named(edge: &NamedExport, reason: String) -> UnknownExport {
    UnknownExport::new(
        Some(edge.name.clone()),
        reason,
        edge.guard.clone(),
        edge.visibility.clone(),
    )
}

pub(super) fn unknown_glob(edge: &GlobExport, reason: String) -> UnknownExport {
    UnknownExport::new(None, reason, edge.guard.clone(), edge.visibility.clone())
}

pub(super) fn insert_unknown(
    output: &mut MacroExportSet,
    unknown: impl IntoIterator<Item = UnknownExport>,
) -> bool {
    let before = output.unknown.len();
    output.unknown.extend(unknown);
    output.unknown.len() != before
}

pub(super) fn limit_reason(module: &super::LogicalModule) -> String {
    format!(
        "macro export set for {} exceeds the analysis limit",
        module.display_path()
    )
}
