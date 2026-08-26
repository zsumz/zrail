//! Active and potentially procedural attributes become explicit expansion facts.

use syn::{Attribute, Meta, Path, Token, punctuated::Punctuated};

use super::SyntaxGuard;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ExpansionKind {
    Attribute,
    Derive,
}

pub(super) struct ExpansionPath {
    pub(super) path: Path,
    pub(super) kind: ExpansionKind,
    pub(super) guard: SyntaxGuard,
}

pub(super) fn attribute_paths(attribute: &Attribute) -> Result<Vec<ExpansionPath>, ()> {
    let mut paths = Vec::new();
    for effect in super::cfg::cfg_guards::guarded_attribute_effects(attribute)? {
        collect_meta(&effect.meta, &effect.guard, &mut paths)?;
    }
    Ok(paths)
}

pub(super) fn can_replace_item(attribute: &Attribute) -> bool {
    attribute_paths(attribute).map_or(true, |expansions| {
        expansions
            .iter()
            .any(|expansion| expansion.kind == ExpansionKind::Attribute)
    })
}

fn collect_meta(
    meta: &Meta,
    guard: &SyntaxGuard,
    paths: &mut Vec<ExpansionPath>,
) -> Result<(), ()> {
    if meta.path().is_ident("derive") {
        let Meta::List(list) = meta else {
            return Err(());
        };
        let derives = list
            .parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
            .map_err(|_| ())?;
        if derives.is_empty() {
            return Err(());
        }
        paths.extend(derives.into_iter().map(|path| ExpansionPath {
            path,
            kind: ExpansionKind::Derive,
            guard: guard.clone(),
        }));
        return Ok(());
    }
    if !is_inert(meta.path()) {
        paths.push(ExpansionPath {
            path: meta.path().clone(),
            kind: ExpansionKind::Attribute,
            guard: guard.clone(),
        });
    }
    Ok(())
}

pub(super) fn is_builtin_derive(path: &Path) -> bool {
    path.get_ident().is_some_and(|name| {
        matches!(
            name.to_string().as_str(),
            "Clone"
                | "Copy"
                | "Debug"
                | "Default"
                | "Eq"
                | "Hash"
                | "Ord"
                | "PartialEq"
                | "PartialOrd"
        )
    })
}

pub(super) fn is_compiler_derive(path: &Path, resolved: &str) -> bool {
    if !is_builtin_derive(path) {
        return false;
    }
    let name = path
        .get_ident()
        .map(ToString::to_string)
        .unwrap_or_default();
    if resolved == name {
        return true;
    }
    let root = resolved.split("::").next().unwrap_or_default();
    let leaf = resolved.rsplit("::").next().unwrap_or(resolved);
    matches!(root, "core" | "std") && leaf == name
}

fn is_inert(path: &Path) -> bool {
    let Some(name) = path.get_ident().map(ToString::to_string) else {
        return path.segments.first().is_some_and(|segment| {
            matches!(
                segment.ident.to_string().as_str(),
                "clippy" | "diagnostic" | "rustfmt"
            )
        });
    };
    matches!(
        name.as_str(),
        "alloc_error_handler"
            | "allow"
            | "automatically_derived"
            | "bench"
            | "cfg"
            | "cold"
            | "coverage"
            | "crate_name"
            | "crate_type"
            | "deny"
            | "deprecated"
            | "default"
            | "doc"
            | "expect"
            | "export_name"
            | "feature"
            | "forbid"
            | "global_allocator"
            | "ignore"
            | "inline"
            | "instruction_set"
            | "link"
            | "link_name"
            | "link_ordinal"
            | "link_section"
            | "macro_export"
            | "macro_use"
            | "must_use"
            | "naked"
            | "no_builtins"
            | "no_implicit_prelude"
            | "no_main"
            | "no_mangle"
            | "no_std"
            | "non_exhaustive"
            | "panic_handler"
            | "path"
            | "proc_macro"
            | "proc_macro_attribute"
            | "proc_macro_derive"
            | "register_tool"
            | "repr"
            | "should_panic"
            | "target_feature"
            | "test"
            | "thread_local"
            | "track_caller"
            | "type_length_limit"
            | "unsafe"
            | "used"
            | "warn"
            | "windows_subsystem"
    )
}

#[cfg(test)]
#[path = "macro_expansion_test.rs"]
mod macro_expansion_test;
