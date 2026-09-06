//! Closed subject matching keeps syntax identities separate and charges bounded work.

use std::collections::BTreeSet;
use zrail_core::RustInventorySubject;

const MAX_WORK: usize = 64 * 1024 * 1024;

pub(super) fn selected_names<'a>(
    subject: &RustInventorySubject,
    fact: &'a crate::source::ObservedFact,
    names: &BTreeSet<&str>,
    work: &mut usize,
) -> Result<BTreeSet<&'a str>, String> {
    charge(work)?;
    let mut selected = BTreeSet::new();
    let name = match subject {
        RustInventorySubject::WrittenTraitImpls {
            implementing_types, ..
        } => {
            if names.contains(fact.name.as_str()) {
                let identity = fact
                    .written
                    .as_deref()
                    .ok_or("trait impl has no written identity")?;
                let (_, type_name) = identity
                    .split_once(" for ")
                    .ok_or("trait impl has no implementing type")?;
                let mut included = true;
                if let Some(types) = implementing_types.get(&fact.name) {
                    included = false;
                    for name in types {
                        charge(work)?;
                        if name == type_name {
                            included = true;
                            break;
                        }
                    }
                }
                if included {
                    selected.insert(identity);
                }
            }
            None
        }
        RustInventorySubject::WrittenImportRenames { .. } => {
            if names.contains(fact.name.as_str()) {
                selected.insert(
                    fact.written
                        .as_deref()
                        .ok_or("import rename has no written identity")?,
                );
            }
            None
        }
        RustInventorySubject::WrittenPathsContaining { .. } => {
            let written = fact
                .written
                .as_deref()
                .ok_or("authored path has no written spelling")?;
            for segment in written.split("::") {
                charge(work)?;
                if names.contains(segment) {
                    selected.insert(segment);
                }
            }
            None
        }
        RustInventorySubject::WrittenMethods { .. } => Some(fact.name.as_str()),
        RustInventorySubject::WrittenExpressionPaths { .. } => {
            fact.written.as_deref().and_then(|written| {
                let written = written.trim_start_matches("::");
                let (owner, _) = written.rsplit_once("::")?;
                Some(
                    owner
                        .rfind("::")
                        .map_or(written, |offset| &written[offset + 2..]),
                )
            })
        }
    };
    if let Some(name) = name.filter(|name| names.contains(name)) {
        selected.insert(name);
    }
    Ok(selected)
}

fn charge(work: &mut usize) -> Result<(), String> {
    *work += 1;
    if *work > MAX_WORK {
        Err(format!(
            "Rust inventories exceed the {MAX_WORK}-fact comparison limit"
        ))
    } else {
        Ok(())
    }
}
