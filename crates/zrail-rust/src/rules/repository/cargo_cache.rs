//! Canonical Cargo cache exclusions are valid before or after cache creation.

use std::{fs, io::Read, path::Path};

use super::RuleContext;

pub(super) fn exclusion(context: &RuleContext<'_>, pattern: &str) -> Option<(String, bool)> {
    let target = target_directory(context, pattern)?;
    let absolute = context.inventory.root.join(&target);
    let metadata = match fs::symlink_metadata(&absolute) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Some((target, true)),
        Err(_) => return Some((target, false)),
    };
    if !metadata.file_type().is_dir() {
        return Some((target, false));
    }
    let valid = valid_tag(&absolute.join("CACHEDIR.TAG"));
    Some((target, valid))
}

fn target_directory(context: &RuleContext<'_>, pattern: &str) -> Option<String> {
    let root_manifest = context.inventory.root.join("Cargo.toml");
    if root_manifest.is_file() && pattern == "target/**" {
        return Some("target".into());
    }
    context.cargo.packages.iter().find_map(|package| {
        let target = if package.directory == "." {
            "target".into()
        } else {
            format!("{}/target", package.directory)
        };
        (pattern == format!("{target}/**")).then_some(target)
    })
}

fn valid_tag(path: &Path) -> bool {
    const SIGNATURE: &[u8] = b"Signature: 8a477f597d28d172789f06886806bc55";
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return true,
        Err(_) => return false,
    };
    if !metadata.file_type().is_file() {
        return false;
    }
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    let mut observed = [0_u8; SIGNATURE.len()];
    file.read_exact(&mut observed).is_ok() && observed == SIGNATURE
}
