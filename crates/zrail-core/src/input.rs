//! Architecture inputs are regular files; creation and replacement never follow aliases.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

pub const MAX_DIRECTORY_DEPTH: usize = 128;
pub const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_REPOSITORY_ENTRIES: usize = 200_000;

pub fn read_text(path: &Path) -> Result<String, String> {
    read_text_with_limit(path, MAX_INPUT_BYTES)
}

pub fn read_text_with_limit(path: &Path, limit: usize) -> Result<String, String> {
    let bytes = read_bytes_with_limit(path, limit)?;
    String::from_utf8(bytes).map_err(|error| format!("read {} as UTF-8: {error}", path.display()))
}

pub fn read_bytes_with_limit(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let metadata = require_regular(path)?;
    if metadata.len() > limit as u64 {
        return Err(oversized(path, metadata.len(), limit));
    }
    let file = File::open(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    if bytes.len() > limit {
        return Err(oversized(path, bytes.len() as u64, limit));
    }
    Ok(bytes)
}

pub fn replace_text(path: &Path, contents: &str) -> Result<(), String> {
    validate_output(path, contents)?;
    match fs::symlink_metadata(path) {
        Ok(_) => {
            require_regular(path)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("inspect {}: {error}", path.display())),
    }
    let parent = output_parent(path);
    let mut temporary = create_temporary(parent, path)?;
    if let Err(error) = write_and_replace(&mut temporary.file, contents, &temporary.path, path) {
        let _removed = fs::remove_file(&temporary.path);
        return Err(error);
    }
    temporary.persisted = true;
    Ok(())
}

pub fn create_text(path: &Path, contents: &str) -> Result<(), String> {
    validate_output(path, contents)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(format!(
                "architecture output must not be a symlink: {}",
                path.display()
            ));
        }
        Ok(_) => {
            return Err(format!(
                "architecture output already exists: {}",
                path.display()
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("inspect {}: {error}", path.display())),
    }
    let parent = output_parent(path);
    let mut temporary = create_temporary(parent, path)?;
    temporary
        .file
        .write_all(contents.as_bytes())
        .map_err(|error| format!("write {}: {error}", temporary.path.display()))?;
    temporary
        .file
        .sync_all()
        .map_err(|error| format!("sync {}: {error}", temporary.path.display()))?;
    fs::hard_link(&temporary.path, path)
        .map_err(|error| format!("create {} without replacement: {error}", path.display()))?;
    Ok(())
}

fn validate_output(path: &Path, contents: &str) -> Result<(), String> {
    if contents.len() > MAX_INPUT_BYTES {
        let observed = u64::try_from(contents.len()).unwrap_or(u64::MAX);
        return Err(oversized(path, observed, MAX_INPUT_BYTES));
    }
    Ok(())
}

fn output_parent(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn require_regular(path: &Path) -> Result<fs::Metadata, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("inspect {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "architecture input must not be a symlink: {}",
            path.display()
        ));
    }
    if !metadata.is_file() {
        return Err(format!(
            "architecture input must be a regular file: {}",
            path.display()
        ));
    }
    Ok(metadata)
}

fn oversized(path: &Path, observed: u64, limit: usize) -> String {
    format!(
        "architecture input exceeds the {limit}-byte safety limit (observed {observed}): {}",
        path.display()
    )
}

struct TemporaryFile {
    file: File,
    path: PathBuf,
    persisted: bool,
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        if !self.persisted {
            let _removed = fs::remove_file(&self.path);
        }
    }
}

fn create_temporary(parent: &Path, destination: &Path) -> Result<TemporaryFile, String> {
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "architecture output has no UTF-8 file name: {}",
                destination.display()
            )
        })?;
    for attempt in 0..100_u8 {
        let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(".{name}.tmp-{sequence}-{attempt}"));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                return Ok(TemporaryFile {
                    file,
                    path,
                    persisted: false,
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(format!("create {}: {error}", path.display())),
        }
    }
    Err(format!(
        "create temporary architecture output beside {}: name collision",
        destination.display()
    ))
}

fn write_and_replace(
    file: &mut File,
    contents: &str,
    temporary: &Path,
    destination: &Path,
) -> Result<(), String> {
    file.write_all(contents.as_bytes())
        .map_err(|error| format!("write {}: {error}", temporary.display()))?;
    file.sync_all()
        .map_err(|error| format!("sync {}: {error}", temporary.display()))?;
    fs::rename(temporary, destination).map_err(|error| {
        format!(
            "replace {} from {}: {error}",
            destination.display(),
            temporary.display()
        )
    })
}

#[cfg(test)]
#[path = "input_test.rs"]
mod input_test;
