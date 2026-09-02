//! Macro glob resolution consumes explicit logical-module export sets.

#[path = "macro_exports/closure.rs"]
mod closure;
#[path = "macro_exports/collect.rs"]
mod collect;
#[path = "macro_exports/contexts.rs"]
mod contexts;
#[path = "macro_exports/edges.rs"]
mod edges;
#[path = "macro_exports/imports.rs"]
mod imports;
#[path = "macro_exports/lookup.rs"]
mod lookup;
#[path = "macro_exports/paths.rs"]
mod paths;
#[path = "macro_exports/resolve.rs"]
mod resolve;
#[path = "macro_exports/unknown.rs"]
mod unknown;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    BindingVisibility, MacroDerivation, MacroOrigin, SourceInstanceId, SourceSyntax, SyntaxGuard,
    logical_modules::LogicalModule,
};
use unknown::UnknownExport;

const MAX_EXPORTS_PER_MODULE: usize = 512;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ExportedMacro {
    origins: Vec<MacroOrigin>,
    authority_name: Option<String>,
    definition: Option<String>,
    definition_name: Option<String>,
    definition_sha256: Option<String>,
    guard: SyntaxGuard,
    quality: AnalysisQuality,
    visibility: ExportVisibility,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct MacroExportSet {
    macros: BTreeMap<MacroSymbol, BTreeSet<ExportedMacro>>,
    unknown: BTreeSet<UnknownExport>,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct MacroNamespace;

type MacroSymbol = (String, MacroNamespace);

#[derive(Clone, Debug, Default)]
struct ModuleDraft {
    local: BTreeMap<String, BTreeSet<ExportedMacro>>,
    direct: BTreeMap<String, BTreeSet<ExportedMacro>>,
    named: Vec<NamedExport>,
    globs: Vec<GlobExport>,
    unknown: BTreeSet<UnknownExport>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct NamedExport {
    name: String,
    target: String,
    guard: SyntaxGuard,
    visibility: ExportVisibility,
    quality: AnalysisQuality,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct GlobExport {
    target: String,
    guard: SyntaxGuard,
    visibility: ExportVisibility,
    quality: AnalysisQuality,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct MountedModule {
    instance: SourceInstanceId,
    module: LogicalModule,
    guard: SyntaxGuard,
    inherited_imports: BTreeSet<MountedImport>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct MountedImport {
    target: String,
    derivation: MacroDerivation,
    guard: SyntaxGuard,
    quality: AnalysisQuality,
}

pub(in crate::source) struct MacroExports {
    sets: BTreeMap<LogicalModule, MacroExportSet>,
    contexts: BTreeMap<(String, SourceSyntax, Vec<SourceSpan>), BTreeSet<MountedModule>>,
    modules: BTreeSet<LogicalModule>,
    package_directories: BTreeMap<String, String>,
    package_dependencies: BTreeMap<String, Vec<crate::cargo::Dependency>>,
    package_roots: BTreeMap<String, BTreeSet<LogicalModule>>,
}

enum ModuleResolution {
    Local { modules: BTreeSet<LogicalModule> },
    External(Vec<MacroOrigin>),
    Missing,
    Unknown(String),
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct ExportVisibility(BTreeSet<LogicalModule>);

impl ExportedMacro {
    fn visible_from(&self, consumer: &LogicalModule) -> bool {
        self.visibility.visible_from(consumer)
    }
}

impl ExportVisibility {
    fn visible_from(&self, consumer: &LogicalModule) -> bool {
        self.0.iter().all(|scope| {
            scope.domain == consumer.domain
                && scope.crate_root == consumer.crate_root
                && consumer.path.starts_with(&scope.path)
        })
    }

    fn private(module: &LogicalModule) -> Self {
        Self(BTreeSet::from([module.clone()]))
    }

    fn restrict(&mut self, other: &Self) {
        self.0.extend(other.0.iter().cloned());
    }
}

fn visibility(
    visibility: &BindingVisibility,
    module: &LogicalModule,
) -> Result<ExportVisibility, String> {
    match visibility {
        BindingVisibility::Public => Ok(ExportVisibility::default()),
        BindingVisibility::Private => Ok(ExportVisibility::private(module)),
        BindingVisibility::Restricted(path) => restricted_scope(module, path)
            .map(|scope| ExportVisibility::private(&scope))
            .ok_or_else(|| {
                format!(
                    "macro visibility path {:?} cannot be resolved from {}",
                    path,
                    module.display_path()
                )
            }),
    }
}

fn restricted_scope(module: &LogicalModule, path: &[String]) -> Option<LogicalModule> {
    let (first, tail) = path.split_first()?;
    let mut scope = match normalize(first) {
        "crate" => module.root(),
        "self" => module.clone(),
        "super" => module.parent()?,
        name if module.domain.edition == "2015" => module.root().child(name),
        _ => return None,
    };
    for segment in tail {
        match normalize(segment) {
            "self" => {}
            "super" => scope = scope.parent()?,
            name => scope = scope.child(name),
        }
    }
    Some(scope)
}

fn merge_export(
    set: &mut MacroExportSet,
    name: String,
    exported: ExportedMacro,
) -> Result<bool, ()> {
    let count = set.macros.values().map(BTreeSet::len).sum::<usize>();
    let exports = set.macros.entry(macro_symbol(name)).or_default();
    if exports.contains(&exported) {
        return Ok(false);
    }
    if count >= MAX_EXPORTS_PER_MODULE {
        return Err(());
    }
    exports.insert(exported);
    Ok(true)
}

fn macro_symbol(name: impl Into<String>) -> MacroSymbol {
    (name.into(), MacroNamespace)
}

fn normalize(segment: &str) -> &str {
    segment.strip_prefix("r#").unwrap_or(segment)
}
