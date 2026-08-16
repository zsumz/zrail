//! Active and potentially procedural attributes become explicit expansion facts.

use syn::{Attribute, Meta, Path, Token, punctuated::Punctuated};

pub(super) fn attribute_paths(attribute: &Attribute) -> Result<Vec<Path>, ()> {
    let mut paths = Vec::new();
    collect_meta(&attribute.meta, &mut paths)?;
    Ok(paths)
}

fn collect_meta(meta: &Meta, paths: &mut Vec<Path>) -> Result<(), ()> {
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
        paths.extend(derives);
        return Ok(());
    }
    if meta.path().is_ident("cfg_attr") {
        let Meta::List(list) = meta else {
            return Err(());
        };
        let arguments = list
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .map_err(|_| ())?;
        if arguments.len() < 2 {
            return Err(());
        }
        for nested in arguments.iter().skip(1) {
            collect_meta(nested, paths)?;
        }
        return Ok(());
    }
    if !is_inert(meta.path()) {
        paths.push(meta.path().clone());
    }
    Ok(())
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
