//! Private temporary roots preserve file mode and create-only snapshot writes.

use std::{
    fs::{self, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::app::error::CliError;

static TEMPORARY_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub(in crate::app::commands) struct TemporaryRoot(PathBuf);

impl TemporaryRoot {
    pub(super) fn create() -> Result<Self, CliError> {
        let base = std::env::temp_dir();
        for _ in 0..100 {
            let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = base.join(format!("zrail-git-{}-{sequence}", std::process::id()));
            match create_private_directory(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(CliError::new(format!("create {}: {error}", path.display())));
                }
            }
        }
        Err(CliError::new("create Git snapshot: name collision"))
    }

    pub(in crate::app::commands) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryRoot {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

pub(super) fn write_new(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    let parent = path
        .parent()
        .ok_or_else(|| CliError::new(format!("snapshot path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent)
        .map_err(|error| CliError::new(format!("create {}: {error}", parent.display())))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| CliError::new(format!("create {}: {error}", path.display())))?;
    file.write_all(bytes)
        .map_err(|error| CliError::new(format!("write {}: {error}", path.display())))?;
    file.sync_all()
        .map_err(|error| CliError::new(format!("sync {}: {error}", path.display())))
}

pub(super) fn set_executable(path: &Path, executable: bool) -> Result<(), CliError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = if executable { 0o755 } else { 0o644 };
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(|error| {
            CliError::new(format!("set Git snapshot mode {}: {error}", path.display()))
        })?;
    }
    #[cfg(not(unix))]
    let _ = (path, executable);
    Ok(())
}

fn create_private_directory(path: &Path) -> std::io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(0o700);
    }
    builder.create(path)
}

#[cfg(test)]
#[path = "filesystem_test.rs"]
mod filesystem_test;
