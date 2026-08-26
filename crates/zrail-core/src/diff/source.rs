//! Source-convention and budget permission changes.

mod feature_worlds;
mod file_roles;
mod item_macros;
mod macros;
mod out_dir;
mod size;
mod types;

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, GeneratedSourceContract};

use super::{
    ArchitectureChange, ChangeKind,
    support::{
        compare_named_set, compare_number, compare_ordered_mode, rank_facades, rank_glob_imports,
        rank_lint_suppressions, rank_module_docs, rank_policy, rank_tests,
    },
};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    size::compare(before, after, changes);
    file_roles::compare(before, after, changes);
    feature_worlds::compare(before, after, changes);
    compare_generated(before, after, changes);
    out_dir::compare(before, after, changes);
    item_macros::compare(before, after, changes);
    macros::compare(before, after, changes);
    types::compare(before, after, changes);
    compare_modes(before, after, changes);
    compare_hygiene(before, after, changes);
}

fn compare_generated(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    let old = generated_by_root(&before.source.rust.generated);
    let new = generated_by_root(&after.source.rust.generated);
    let roots = old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for root in roots {
        match (old.get(root), new.get(root)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.generated-source",
                root,
                "source became generator-owned",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.generated-source",
                root,
                "source returned to handwritten enforcement",
            )),
            (Some(left), Some(right)) => {
                compare_number(
                    "rust.generated-source",
                    &format!("{root}.target"),
                    left.target,
                    right.target,
                    changes,
                );
                compare_number(
                    "rust.generated-source",
                    &format!("{root}.hard"),
                    left.hard,
                    right.hard,
                    changes,
                );
                if left.manifest != right.manifest {
                    changes.push(
                        ArchitectureChange::new(
                            ChangeKind::Unknown,
                            "rust.generated-source",
                            root,
                            "generated manifest identity changed",
                        )
                        .values(&left.manifest, &right.manifest),
                    );
                }
                compare_named_set(
                    "rust.generated-source.auxiliary",
                    root,
                    &left.auxiliary,
                    &right.auxiliary,
                    ChangeKind::Grant,
                    ChangeKind::Revoke,
                    "allows manifest-owned source outside the Cargo graph",
                    changes,
                );
                compare_named_set(
                    "rust.generated-source.input",
                    root,
                    &left.inputs,
                    &right.inputs,
                    ChangeKind::Revoke,
                    ChangeKind::Grant,
                    "verifies generator input",
                    changes,
                );
            }
            (None, None) => {}
        }
    }
}

fn generated_by_root(
    generated: &[GeneratedSourceContract],
) -> BTreeMap<&str, &GeneratedSourceContract> {
    generated
        .iter()
        .map(|generated| (generated.root.as_str(), generated))
        .collect()
}

fn compare_modes(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    compare_ordered_mode(
        "rust.module-docs",
        "source.rust.module_docs",
        rank_module_docs(before.source.rust.module_docs),
        rank_module_docs(after.source.rust.module_docs),
        changes,
    );
    compare_ordered_mode(
        "rust.facades",
        "source.rust.facades",
        rank_facades(before.source.rust.facades),
        rank_facades(after.source.rust.facades),
        changes,
    );
    compare_ordered_mode(
        "rust.entrypoints",
        "source.rust.entrypoints",
        rank_facades(before.source.rust.entrypoints),
        rank_facades(after.source.rust.entrypoints),
        changes,
    );
    compare_ordered_mode(
        "rust.tests",
        "source.rust.tests",
        rank_tests(before.source.rust.tests),
        rank_tests(after.source.rust.tests),
        changes,
    );
    compare_ordered_mode(
        "rust.unsafe",
        "source.rust.hygiene.unsafe",
        rank_policy(before.source.rust.hygiene.unsafe_code),
        rank_policy(after.source.rust.hygiene.unsafe_code),
        changes,
    );
    compare_ordered_mode(
        "rust.lint-suppressions",
        "source.rust.hygiene.lint_suppressions",
        rank_lint_suppressions(before.source.rust.hygiene.lint_suppressions),
        rank_lint_suppressions(after.source.rust.hygiene.lint_suppressions),
        changes,
    );
    compare_ordered_mode(
        "rust.glob-imports",
        "source.rust.hygiene.glob_imports",
        rank_glob_imports(before.source.rust.hygiene.glob_imports),
        rank_glob_imports(after.source.rust.hygiene.glob_imports),
        changes,
    );
}

fn compare_hygiene(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    compare_named_set(
        "rust.deny-methods",
        "source",
        &before.source.rust.hygiene.deny_methods,
        &after.source.rust.hygiene.deny_methods,
        ChangeKind::Revoke,
        ChangeKind::Grant,
        "denies method use",
        changes,
    );
    compare_named_set(
        "rust.deny-macros",
        "source",
        &before.source.rust.hygiene.deny_macros,
        &after.source.rust.hygiene.deny_macros,
        ChangeKind::Revoke,
        ChangeKind::Grant,
        "denies macro use",
        changes,
    );
}
