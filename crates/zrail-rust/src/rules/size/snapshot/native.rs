//! Exercise the production size evaluator over measured physical inputs, without locks.

use zrail_core::{FindingSink, LockedRatchet, RatchetContract, RustSourceContract, Severity};

use crate::{
    inventory::{RepositoryInventory, RustSourceFile},
    source::index_rust_source,
    source_budget::{self, EffectiveSizeBudget},
};

pub(super) fn budget(file: &RustSourceFile, rust: &RustSourceContract) -> EffectiveSizeBudget {
    source_budget::for_path(
        &file.relative,
        file.class,
        crate::source::Reachability::UNREACHABLE,
        &[],
        rust,
    )
    .expect("one matching native size scope")
    .expect("every frozen governed file has a budget")
}

pub(super) fn errors(
    file: &RustSourceFile,
    lines: usize,
    budget: &EffectiveSizeBudget,
    ratchet: Option<&RatchetContract>,
    rust: &RustSourceContract,
) -> Vec<String> {
    // Size consumes physical path/line facts. Empty syntax is a unit-test carrier;
    // this harness makes no Rust syntax, resolution, or compilation-world claim.
    let inventory = RepositoryInventory {
        root: std::path::PathBuf::new(),
        entries: Vec::new(),
        manifest_paths: Vec::new(),
        rust_files: vec![RustSourceFile {
            relative: file.relative.clone(),
            class: file.class,
            source: String::new(),
            lines,
        }],
    };
    let facts = index_rust_source(&inventory, rust).files.remove(0);
    // This is one unit-test input, not a LockFile, certificate, or accepted authority.
    let locked = ratchet.map(|ratchet| LockedRatchet {
        rule: ratchet.rule.clone(),
        selector: None,
        target: ratchet.target.clone(),
        value: ratchet
            .baseline
            .expect("every translated ratchet binds an authored baseline"),
    });
    let mut findings = FindingSink::default();
    super::super::super::check_file(
        &facts,
        Some(budget),
        ratchet,
        locked.as_ref(),
        &mut findings,
    );
    findings
        .into_findings()
        .into_iter()
        .filter(|finding| finding.severity == Severity::Error)
        .map(|finding| finding.id)
        .collect()
}
