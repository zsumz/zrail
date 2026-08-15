//! Semantic comparison for trusted `OUT_DIR` generated-source bindings.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, OutDirSourceContract};

use super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    let old = by_identity(before);
    let new = by_identity(after);
    let identities = old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for identity in identities {
        let subject = format!("{}:{}", identity.0, identity.1);
        match (old.get(&identity), new.get(&identity)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.source-graph.out-dir",
                subject,
                "OUT_DIR expansion became trusted through a generated snapshot",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.source-graph.out-dir",
                subject,
                "OUT_DIR generated-source trust was removed",
            )),
            (Some(left), Some(right)) if left.source != right.source => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Unknown,
                    "rust.source-graph.out-dir",
                    subject,
                    "OUT_DIR generated snapshot identity changed",
                )
                .values(&left.source, &right.source),
            ),
            _ => {}
        }
    }
}

fn by_identity(contract: &Contract) -> BTreeMap<(&str, &str), &OutDirSourceContract> {
    contract
        .source
        .rust
        .out_dir
        .iter()
        .map(|binding| ((binding.path.as_str(), binding.output.as_str()), binding))
        .collect()
}
