//! External source module paths stay archive-relative and normalized.

use super::super::archive::normalized_relative;

pub(super) fn absolute_target(module: &[String], target: &[String]) -> Option<Vec<String>> {
    let (first, tail) = target.split_first()?;
    let mut output = match first.as_str() {
        "crate" => Vec::new(),
        "self" => module.to_vec(),
        "super" => module.get(..module.len().checked_sub(1)?)?.to_vec(),
        _ => return None,
    };
    output.extend(tail.iter().cloned());
    Some(output)
}

pub(super) fn parent(path: &str) -> String {
    path.rsplit_once('/')
        .map_or("", |(parent, _)| parent)
        .to_owned()
}

pub(super) fn join(parent: &str, child: &str) -> String {
    let joined = if parent.is_empty() {
        child.to_owned()
    } else {
        format!("{parent}/{child}")
    };
    normalized_relative(&joined).unwrap_or(joined)
}

pub(super) fn display(module: &[String]) -> String {
    if module.is_empty() {
        "crate root".into()
    } else {
        module.join("::")
    }
}

pub(super) fn path_text(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
