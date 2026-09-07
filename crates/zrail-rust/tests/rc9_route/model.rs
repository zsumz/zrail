//! Exact physical fixture selection and typed policy parsing for the mixed route test.

use std::{collections::BTreeMap, fs, path::PathBuf};

use serde::Deserialize;
use zrail_core::RepositoryFileRule;

pub(super) const FACADE: &str = "src/reactor/direct_plaintext/cluster_runtime.rs";
pub(super) const MARKER: &str = "connections: DirectSetOwner<T>";
pub(super) const FORBIDDEN: [&str; 4] = ["ConnectionSet", "Poller", "SingleBroker", "BrokerSet"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    repository: Rules,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    files: Vec<RepositoryFileRule>,
}

pub(super) fn project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("project")
        .to_owned()
}

pub(super) fn policies() -> Vec<RepositoryFileRule> {
    let fragment: Fragment = toml::from_str(
        &fs::read_to_string(
            project().join("docs/rc9/policies/kafka-driver.route-sources.fragment.toml"),
        )
        .expect("route policy bytes"),
    )
    .expect("strict stock file policies");
    assert_eq!(fragment.repository.files.len(), 6);
    fragment.repository.files
}

pub(super) struct Fixture {
    pub(super) root: PathBuf,
    pub(super) inputs: BTreeMap<String, String>,
}

impl Fixture {
    pub(super) fn new(name: &str, policies: &[RepositoryFileRule]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "zrail-route-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        Self::at(root, policies)
    }

    pub(super) fn at(root: PathBuf, policies: &[RepositoryFileRule]) -> Self {
        assert!(!root.exists(), "never overwrite unrelated fixture data");
        let inputs: BTreeMap<_, _> = policies
            .iter()
            .flat_map(|policy| &policy.include)
            .map(|path| {
                let text = fs::read_to_string(
                    project()
                        .join("crates/zrail-testkit/tests/fixtures/rc9/route-sources/valid")
                        .join(path),
                )
                .expect("frozen route input");
                (path.clone(), text)
            })
            .collect();
        assert_eq!(inputs.len(), 11);
        for (path, source) in &inputs {
            let destination = root.join(path);
            fs::create_dir_all(destination.parent().expect("parent")).expect("fixture directories");
            fs::write(destination, source).expect("fresh fixture source");
        }
        Self { root, inputs }
    }

    pub(super) fn write(&self, path: &str, source: impl AsRef<[u8]>) {
        fs::write(self.root.join(path), source).expect("mutate only isolated input");
    }

    pub(super) fn original_accepts(&self) -> bool {
        let sources = self
            .inputs
            .keys()
            .filter(|path| path.as_str() != FACADE)
            .map(|path| fs::read_to_string(self.root.join(path)))
            .collect::<Result<Vec<_>, _>>();
        let facade = fs::read_to_string(self.root.join(FACADE));
        let (Ok(sources), Ok(facade)) = (sources, facade) else {
            return false;
        };
        std::panic::catch_unwind(|| super::legacy::check(&sources, &facade)).is_ok()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("remove only test-owned inputs");
    }
}
