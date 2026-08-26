//! Validated, rollback-safe multi-file contract rewrites.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use zrail_core::{
    ContractBundle, contract_imports, format_contract_source, glob_matches, load_contract,
    migrate_contract_source, replace_text, repository_file, repository_relative,
};

use crate::app::error::CliError;

pub(super) struct EditPlan {
    root: PathBuf,
    originals: BTreeMap<String, String>,
    rendered: BTreeMap<String, String>,
}

pub(super) fn migration(root: &Path, config: &Path) -> Result<EditPlan, CliError> {
    plan(root, config, true)
}

pub(super) fn format(root: &Path, config: &Path) -> Result<EditPlan, CliError> {
    plan(root, config, false)
}

fn plan(root: &Path, config: &Path, migrate: bool) -> Result<EditPlan, CliError> {
    let bundle = load_contract(root, config).map_err(|error| CliError::new(error.to_string()))?;
    let root = fs::canonicalize(root)
        .map_err(|error| CliError::new(format!("open repository {}: {error}", root.display())))?;
    let config_path = repository_file(&root, config).map_err(CliError::new)?;
    let config = repository_relative(&root, &config_path).map_err(CliError::new)?;
    let originals = bundle
        .sources
        .iter()
        .map(|source| (source.path.clone(), source.content.clone()))
        .collect::<BTreeMap<_, _>>();
    let rendered = render_sources(&bundle, &config, migrate)?;
    validate_overlay(&config, &rendered)?;
    Ok(EditPlan {
        root,
        originals,
        rendered,
    })
}

fn render_sources(
    bundle: &ContractBundle,
    config: &str,
    migrate: bool,
) -> Result<BTreeMap<String, String>, CliError> {
    let paths = bundle
        .sources
        .iter()
        .map(|source| source.path.as_str())
        .collect::<Vec<_>>();
    bundle
        .sources
        .iter()
        .map(|source| {
            let text = if migrate {
                let imports = expand_imports(&source.content, &source.path, &paths)?;
                migrate_contract_source(&source.content, source.path == config, &imports)
            } else {
                format_contract_source(&source.content)
            }
            .map_err(|error| {
                CliError::new(format!("format contract source {}: {error}", source.path))
            })?;
            Ok((source.path.clone(), text))
        })
        .collect()
}

fn expand_imports(source: &str, path: &str, paths: &[&str]) -> Result<Vec<String>, CliError> {
    let imports = contract_imports(source, path)
        .map_err(|error| CliError::new(format!("read imports from {path}: {error}")))?;
    let mut exact = Vec::new();
    for pattern in imports {
        if pattern.bytes().any(|byte| matches!(byte, b'*' | b'?')) {
            exact.extend(
                paths
                    .iter()
                    .filter(|path| glob_matches(&pattern, path))
                    .map(|path| (*path).to_owned()),
            );
        } else {
            exact.push(pattern);
        }
    }
    exact.sort();
    exact.dedup();
    Ok(exact)
}

fn validate_overlay(config: &str, rendered: &BTreeMap<String, String>) -> Result<(), CliError> {
    let overlay = TemporaryOverlay::create(rendered)?;
    load_contract(overlay.path(), Path::new(config))
        .map(|_| ())
        .map_err(|error| CliError::new(format!("validate formatted contract: {error}")))
}

impl EditPlan {
    pub(super) fn changed(&self) -> usize {
        self.changed_paths().count()
    }

    pub(super) fn changed_paths(&self) -> impl Iterator<Item = &str> {
        self.rendered.iter().filter_map(|(path, rendered)| {
            (self.originals.get(path) != Some(rendered)).then_some(path.as_str())
        })
    }

    pub(super) fn write(&self) -> Result<(), CliError> {
        self.write_with(require_text, replace_text)
    }

    fn write_with(
        &self,
        mut verify: impl FnMut(&Path, &str) -> Result<(), String>,
        mut replace: impl FnMut(&Path, &str) -> Result<(), String>,
    ) -> Result<(), CliError> {
        let mut written = Vec::new();
        for path in self.changed_paths() {
            let destination = self.root.join(path);
            let write = verify(&destination, &self.originals[path])
                .map_err(|error| format!("verify {path}: {error}"))
                .and_then(|()| {
                    replace(&destination, &self.rendered[path])
                        .map_err(|error| format!("write {path}: {error}"))
                });
            if let Err(error) = write {
                let mut rollback_errors = Vec::new();
                for restored in written.into_iter().rev() {
                    let destination = self.root.join(restored);
                    match verify(&destination, &self.rendered[restored]) {
                        Err(rollback) => rollback_errors.push(format!(
                            "refuse to restore changed source {restored}: {rollback}"
                        )),
                        Ok(()) => {
                            if let Err(rollback) = replace(&destination, &self.originals[restored])
                            {
                                rollback_errors.push(format!("restore {restored}: {rollback}"));
                            }
                        }
                    }
                }
                let rollback = if rollback_errors.is_empty() {
                    String::new()
                } else {
                    format!("; rollback also failed: {}", rollback_errors.join("; "))
                };
                return Err(CliError::new(format!("{error}{rollback}")));
            }
            written.push(path);
        }
        Ok(())
    }
}

fn require_text(path: &Path, expected: &str) -> Result<(), String> {
    let current = fs::read_to_string(path)
        .map_err(|error| format!("read current source {}: {error}", path.display()))?;
    if current != expected {
        return Err(format!(
            "source {} changed after the edit was planned",
            path.display()
        ));
    }
    Ok(())
}

struct TemporaryOverlay(PathBuf);

static OVERLAY_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

impl TemporaryOverlay {
    fn create(rendered: &BTreeMap<String, String>) -> Result<Self, CliError> {
        let root = (0..100)
            .find_map(|_| {
                let sequence = OVERLAY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
                let root = std::env::temp_dir().join(format!(
                    "zrail-config-edit-{}-{sequence}",
                    std::process::id()
                ));
                match fs::create_dir(&root) {
                    Ok(()) => Some(Ok(root)),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => None,
                    Err(error) => Some(Err(error)),
                }
            })
            .ok_or_else(|| CliError::new("create config overlay: name collision"))?
            .map_err(|error| CliError::new(format!("create config overlay: {error}")))?;
        let overlay = Self(root);
        for (path, content) in rendered {
            let destination = overlay.0.join(path);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    CliError::new(format!("create config overlay directory: {error}"))
                })?;
            }
            fs::write(&destination, content)
                .map_err(|error| CliError::new(format!("write config overlay: {error}")))?;
        }
        Ok(overlay)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryOverlay {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
#[path = "config_edit_test.rs"]
mod config_edit_test;
