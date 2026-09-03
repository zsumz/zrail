//! Logical Rust modules separate mounted evaluation context from physical source.

use std::collections::BTreeMap;

use zrail_core::SourceSpan;

use super::{
    BindingKind, CompilationDomain, ModuleBinding, SourceEntry, SourceIndex, SourceInstanceId,
    SourceInstances, SourceSyntax,
};

pub(super) type InlineModuleCatalog =
    BTreeMap<(String, SourceSyntax), BTreeMap<SourceSpan, String>>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct LogicalModule {
    pub(super) domain: CompilationDomain,
    pub(super) crate_root: String,
    pub(super) path: Vec<String>,
}

impl LogicalModule {
    pub(super) fn root(&self) -> Self {
        Self {
            domain: self.domain.clone(),
            crate_root: self.crate_root.clone(),
            path: Vec::new(),
        }
    }

    pub(super) fn child(&self, name: &str) -> Self {
        let mut child = self.clone();
        child.path.push(normalize(name));
        child
    }

    pub(super) fn parent(&self) -> Option<Self> {
        let mut parent = self.clone();
        parent.path.pop()?;
        Some(parent)
    }

    pub(super) fn display_path(&self) -> String {
        if self.path.is_empty() {
            "crate".into()
        } else {
            format!("crate::{}", self.path.join("::"))
        }
    }
}

pub(super) fn inline_catalog(index: &SourceIndex) -> InlineModuleCatalog {
    index
        .files
        .iter()
        .map(|file| {
            (
                (file.relative.clone(), file.syntax),
                file.import_bindings
                    .iter()
                    .filter_map(|binding| match binding.kind {
                        BindingKind::Module(ModuleBinding::Inline(span)) => {
                            binding.name.as_ref().map(|name| (span, normalize(name)))
                        }
                        _ => None,
                    })
                    .collect(),
            )
        })
        .collect()
}

pub(super) fn locate(
    instances: &SourceInstances,
    instance: SourceInstanceId,
    scope: &[SourceSpan],
    inline: &InlineModuleCatalog,
) -> Option<LogicalModule> {
    let source = instances.get(instance)?;
    let mut module = match (&source.parent, &source.entered_from) {
        (None, SourceEntry::CargoRoot) => LogicalModule {
            domain: source.domain.clone(),
            crate_root: source.file.clone(),
            path: Vec::new(),
        },
        (Some(parent), SourceEntry::Module(edge)) => {
            locate(instances, *parent, &edge.parent_scope, inline)?.child(&edge.module_name)
        }
        (Some(parent), SourceEntry::Include(edge)) => {
            locate(instances, *parent, &edge.parent_scope, inline)?
        }
        _ => return None,
    };
    module
        .path
        .extend(inline_names(inline, &source.file, source.syntax, scope));
    Some(module)
}

fn inline_names(
    inline: &InlineModuleCatalog,
    file: &str,
    syntax: SourceSyntax,
    scope: &[SourceSpan],
) -> Vec<String> {
    let Some(modules) = inline.get(&(file.into(), syntax)) else {
        return Vec::new();
    };
    scope
        .iter()
        .filter_map(|span| modules.get(span).cloned())
        .collect()
}

fn normalize(name: &str) -> String {
    name.strip_prefix("r#").unwrap_or(name).into()
}
