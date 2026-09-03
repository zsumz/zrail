//! Archive paths are accepted only in one exact UTF-8 package root.

use std::{
    ffi::OsString,
    path::{Component, Path},
};

pub(super) fn checked_member_path(path: &Path, expected_root: &str) -> Result<String, String> {
    let raw = path
        .to_str()
        .ok_or_else(|| "crate archive member path is not UTF-8".to_owned())?;
    if invalid(raw) {
        return Err("crate archive member path is not normalized".into());
    }
    let mut components = path.components();
    let Some(Component::Normal(root)) = components.next() else {
        return Err("crate archive member has no package root".into());
    };
    if root != expected_root {
        return Err(format!(
            "crate archive member is outside root {expected_root:?}"
        ));
    }
    let mut relative = Vec::<OsString>::new();
    for component in components {
        let Component::Normal(component) = component else {
            return Err("crate archive member path is not normalized".into());
        };
        relative.push(component.to_os_string());
    }
    let relative = relative
        .iter()
        .map(|component| component.to_str())
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| "crate archive member path is not UTF-8".to_owned())?
        .join("/");
    if relative.is_empty() {
        return Err("crate archive member path is invalid".into());
    }
    Ok(relative)
}

pub(in crate::source::macro_resolution::exports::external) fn normalized_relative(
    path: &str,
) -> Result<String, String> {
    if invalid(path) {
        return Err(format!("crate source path {path:?} is not normalized"));
    }
    let path = Path::new(path);
    let mut parts = Vec::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            return Err(format!(
                "crate source path {} is not normalized",
                path.display()
            ));
        };
        parts.push(
            component
                .to_str()
                .ok_or_else(|| format!("crate source path {} is not UTF-8", path.display()))?,
        );
    }
    if parts.is_empty() {
        return Err("crate source path is empty".into());
    }
    Ok(parts.join("/"))
}

fn invalid(path: &str) -> bool {
    path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|component| matches!(component, "" | "." | ".."))
}
