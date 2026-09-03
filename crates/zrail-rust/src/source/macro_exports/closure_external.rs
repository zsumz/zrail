//! Checksum-proven external glob exports join repository export closure.

use zrail_core::AnalysisQuality;

use super::edges::{insert_unknown, limit_reason, unknown_glob};
use super::{
    ExportedMacro, ExternalModule, GlobExport, MacroExportSet, MacroExports, MacroOrigin,
    UnknownExport, merge_export,
};

impl MacroExports {
    pub(super) fn apply_external_glob(
        &self,
        module: &super::LogicalModule,
        edge: &GlobExport,
        output: &mut MacroExportSet,
        origins: &[MacroOrigin],
        target: &ExternalModule,
    ) -> bool {
        let exports = match self.external.module(target) {
            Ok(exports) => exports,
            Err(reason) => return insert_unknown(output, [unknown_glob(edge, reason)]),
        };
        let mut changed = false;
        for name in exports.macros {
            let exported = ExportedMacro {
                origins: origins.to_vec(),
                proc_macro: false,
                authority_name: Some(format!("{}::{name}", edge.target)),
                definition: None,
                definition_name: None,
                definition_sha256: None,
                guard: edge.guard.clone(),
                quality: AnalysisQuality::Exact,
                visibility: edge.visibility.clone(),
            };
            match merge_export(output, name, exported) {
                Ok(inserted) => changed |= inserted,
                Err(()) => {
                    changed |=
                        insert_unknown(output, [UnknownExport::unbounded(limit_reason(module))]);
                }
            }
        }
        changed |= insert_unknown(
            output,
            exports.uncertain.into_iter().map(|(name, reason)| {
                UnknownExport::new(
                    Some(name),
                    reason,
                    edge.guard.clone(),
                    edge.visibility.clone(),
                )
            }),
        );
        if let Some(reason) = exports.open {
            changed |= insert_unknown(output, [unknown_glob(edge, reason)]);
        }
        changed
    }
}
