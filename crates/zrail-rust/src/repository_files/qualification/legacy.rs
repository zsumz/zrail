//! Policy IDs select complete frozen test bodies, never evaluator code inside analysis.

#[path = "gate.rs"]
mod gate;
#[path = "release.rs"]
mod release;
#[path = "security.rs"]
mod security;

use std::{fs, path::Path};

use zrail_core::{RepositoryFilePredicate, RepositoryFileRule};

pub(super) fn check(root: &Path, policy: &RepositoryFileRule) {
    if matches!(policy.predicate, RepositoryFilePredicate::BytesEqual { .. }) {
        read(&root.join(&policy.include[0]));
        return;
    }
    let line = policy
        .name
        .strip_prefix("kd-qual-")
        .expect("reviewed policy")
        .split('-')
        .next()
        .unwrap()
        .parse::<usize>()
        .expect("source assertion line");
    match line {
        15..=17 => gate::canonical_gate_uses_the_locked_graph_and_detects_mutation(root),
        25..=27 => gate::ci_prefetches_before_running_the_gate_offline(root),
        44..=69 => gate::functional_smoke_runs_on_pull_requests_and_main_without_performance(root),
        81..=97 => release::release_qualification_runs_the_canonical_gate_before_packaging(root),
        105..=109 => gate::ci_uses_the_locked_registry_protocol_without_a_sibling_checkout(root),
        119..=151 => release::release_qualification_builds_normalized_public_archives(root),
        160..=168 => {
            release::release_qualification_resolves_latest_compatible_from_a_clean_registry(root);
        }
        176..=182 => {
            release::release_qualification_is_scheduled_and_uses_the_registry_protocol(root);
        }
        193..=209 => {
            security::secure_cluster_qualification_binds_each_advertised_host_to_its_identity(root);
        }
        228 => security::qualification_containers_use_immutable_digests(root),
        _ => panic!("unknown frozen qualification assertion"),
    }
}

pub(crate) fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}
