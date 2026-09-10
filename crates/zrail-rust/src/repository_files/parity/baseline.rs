//! Full frozen path selection and raw predicate outcomes are compared independently.

use std::collections::BTreeSet;

use zrail_core::RepositoryFilePredicate;

use super::super::model::GovernedRepositoryFile;
use super::{
    legacy,
    model::{LegacyObservation, Snapshot},
};

pub(super) fn compare(
    snapshot: &Snapshot,
    policies: &[GovernedRepositoryFile],
) -> Vec<LegacyObservation> {
    let mut results = Vec::new();
    for policy in policies {
        let source = &policy.policy;
        let paths = match &source.predicate {
            RepositoryFilePredicate::Count { .. } => {
                assert!(source.name.starts_with("kd-capability-root-"));
                let index = source
                    .name
                    .rsplit('-')
                    .next()
                    .expect("registry index")
                    .parse::<usize>()
                    .expect("index")
                    - 1;
                let expected = snapshot.config["capabilities"][index]["root"]
                    .as_str()
                    .expect("root");
                assert_eq!(source.include, [expected]);
                assert!(legacy::directory(&snapshot.root, expected));
                BTreeSet::from([expected.to_owned()])
            }
            RepositoryFilePredicate::Literal(literal)
                if source.name.starts_with("kd-capability-") =>
            {
                let indices = source
                    .name
                    .trim_start_matches("kd-capability-")
                    .split('-')
                    .map(|index| index.parse::<usize>().expect("registry index") - 1)
                    .collect::<Vec<_>>();
                let registry = &snapshot.config["capabilities"][indices[0]];
                let root = registry["root"].as_str().expect("capability root");
                assert_eq!(
                    literal.text,
                    registry["forbidden"][indices[1]].as_str().expect("token")
                );
                snapshot
                    .sources
                    .keys()
                    .filter(|path| std::path::Path::new(path).starts_with(root))
                    .cloned()
                    .collect()
            }
            _ => snapshot.sources.keys().cloned().collect(),
        };
        assert_eq!(
            policy
                .entries
                .iter()
                .map(|entry| entry.path.clone())
                .collect::<BTreeSet<_>>(),
            paths,
            "{} selection",
            policy.policy_id
        );
        for entry in &policy.entries {
            let accepted = match &source.predicate {
                RepositoryFilePredicate::Count { .. } => {
                    legacy::directory(&snapshot.root, &entry.path)
                }
                RepositoryFilePredicate::ForbiddenNames { .. } => {
                    legacy::names(&snapshot.root.join(&entry.path))
                }
                RepositoryFilePredicate::Literal(literal)
                    if source.name.starts_with("kd-capability-") =>
                {
                    legacy::capability(&entry.path, &snapshot.sources[&entry.path], &literal.text)
                }
                RepositoryFilePredicate::Literal(_) => {
                    legacy::contract(&snapshot.sources[&entry.path])
                }
                _ => panic!("unreviewed frozen file-policy family"),
            };
            assert!(
                accepted,
                "frozen legacy predicate fails: {} {}",
                policy.policy_id, entry.path
            );
            assert_eq!(
                accepted, entry.satisfied,
                "{} {}",
                policy.policy_id, entry.path
            );
            results.push(LegacyObservation {
                policy_id: policy.policy_id.clone(),
                path: entry.path.clone(),
                accepted,
            });
        }
        assert!(policy.satisfied, "{}", policy.policy_id);
    }
    results
}
