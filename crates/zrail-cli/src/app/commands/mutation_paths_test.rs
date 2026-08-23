//! Baseline and update never replace entry or imported contract authority.

use std::{fs, path::PathBuf};

use crate::app::{
    args::{BaselineOptions, CommonOptions},
    output::OutputFormat,
};

use super::super::{baseline, update, update_fixture_test};

#[derive(Clone, Copy, Debug)]
enum Collision {
    Entry,
    NormalizedEntry,
    Imported,
    #[cfg(unix)]
    ExistingAlias,
}

#[test]
fn baseline_rejects_every_contract_source_as_its_lock_destination() {
    for collision in collisions() {
        let fixture = Fixture::new("baseline", collision);
        let result = baseline(&BaselineOptions {
            common: fixture.common(),
            dry_run: false,
            accept_grants: true,
            rule: None,
        });

        fixture.assert_refused_and_unchanged(result.map(|result| result.exit_code));
    }
}

#[test]
fn update_rejects_every_contract_source_as_its_lock_destination() {
    for collision in collisions() {
        let fixture = Fixture::new("update", collision);
        let mut options = update_fixture_test::options(&fixture.root);
        options.common = fixture.common();
        options.accept_grants = true;

        fixture.assert_refused_and_unchanged(update(&options).map(|result| result.exit_code));
    }
}

fn collisions() -> Vec<Collision> {
    let mut collisions = vec![
        Collision::Entry,
        Collision::NormalizedEntry,
        Collision::Imported,
    ];
    #[cfg(unix)]
    collisions.push(Collision::ExistingAlias);
    collisions
}

struct Fixture {
    root: PathBuf,
    config: PathBuf,
    lock: PathBuf,
    watched: Vec<(PathBuf, Vec<u8>)>,
}

impl Fixture {
    fn new(command: &str, collision: Collision) -> Self {
        let root = update_fixture_test::fixture_root(&format!("collision-{command}-{collision:?}"));
        update_fixture_test::reset(&root);
        update_fixture_test::write_fixture(&root);
        fs::write(
            root.join("crates/fixture/src/lib.rs"),
            format!("//! fixture\n{}", "// legacy\n".repeat(90)),
        )
        .expect("grow ratchetable source");

        let mut config = PathBuf::from("zrail.toml");
        let lock = match collision {
            Collision::Entry => PathBuf::from("zrail.toml"),
            Collision::NormalizedEntry => {
                config = PathBuf::from("./zrail.toml");
                PathBuf::from("zrail.toml")
            }
            Collision::Imported => {
                fs::create_dir_all(root.join("architecture")).expect("create contract directory");
                fs::write(
                    root.join("architecture/rust.toml"),
                    "# imported contract source\n",
                )
                .expect("write imported contract");
                let entry = fs::read_to_string(root.join("zrail.toml")).expect("read entry");
                fs::write(
                    root.join("zrail.toml"),
                    format!("imports = [\"architecture/rust.toml\"]\n{entry}"),
                )
                .expect("add contract import");
                PathBuf::from("architecture/rust.toml")
            }
            #[cfg(unix)]
            Collision::ExistingAlias => {
                std::os::unix::fs::symlink("zrail.toml", root.join("architecture.lock"))
                    .expect("create existing alias");
                PathBuf::from("architecture.lock")
            }
        };
        let watched_paths = ["zrail.toml", "architecture/rust.toml"];
        let watched = watched_paths
            .into_iter()
            .map(|path| root.join(path))
            .filter(|path| path.exists())
            .map(|path| {
                let bytes = fs::read(&path).expect("read architecture source");
                (path, bytes)
            })
            .collect();
        Self {
            root,
            config,
            lock,
            watched,
        }
    }

    fn common(&self) -> CommonOptions {
        CommonOptions {
            root: self.root.clone(),
            config: self.config.clone(),
            lock: self.lock.clone(),
            format: OutputFormat::Human,
            ..CommonOptions::default()
        }
    }

    fn assert_refused_and_unchanged(self, result: Result<i32, crate::app::error::CliError>) {
        let error = result.expect_err("overlapping mutation must fail");
        assert!(error.message.contains("overlaps contract source"));
        for (path, original) in &self.watched {
            assert_eq!(
                fs::read(path).expect("reread architecture source"),
                *original
            );
        }
        update_fixture_test::reset(&self.root);
    }
}
