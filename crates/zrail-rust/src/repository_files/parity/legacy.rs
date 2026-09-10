//! Byte-exact frozen Kafka-driver raw predicates; trusted qualification only.

use std::path::Path;

const VAGUE_MODULES: [&str; 5] = ["common", "context", "helpers", "manager", "utils"];

fn has_contract(source: &str) -> bool {
    source.trim_start().starts_with("//!")
}

fn has_vague_name(path: &Path) -> bool {
    path.iter().any(|component| {
        Path::new(component)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(|stem| VAGUE_MODULES.contains(&stem))
    })
}

fn source_violations(relative: &str, source: &str, forbidden: &[String]) -> Vec<String> {
    forbidden
        .iter()
        .filter(|token| source.contains(token.as_str()))
        .map(|token| format!("{relative} names forbidden capability `{token}`"))
        .collect()
}

pub(super) fn contract(source: &str) -> bool {
    has_contract(source)
}

pub(super) fn names(path: &Path) -> bool {
    !has_vague_name(path)
}

pub(super) fn directory(root: &Path, relative: &str) -> bool {
    root.join(relative).is_dir()
}

pub(super) fn capability(path: &str, source: &str, token: &str) -> bool {
    source_violations(path, source, &[token.to_owned()]).is_empty()
}
