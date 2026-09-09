//! Unrelated entries exercise the complete original guard and all native workspace nodes.

use super::model;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};
use zrail_core::sha256_hex;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Matrix {
    schema: u32,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    position: String,
    entry: String,
    original_error: Option<String>,
    native_error: Option<String>,
    stage: String,
}
#[derive(Serialize)]
pub(super) struct Row {
    case: String,
    lock_source: String,
    inputs: BTreeMap<String, String>,
    original_error: Option<String>,
    native_error: Option<String>,
    observations: Vec<crate::GovernedLockPackage>,
    stage: String,
}

pub(super) fn run(root: &Path) -> Vec<Row> {
    let matrix: Matrix = serde_json::from_slice(
        &fs::read(model::project().join(model::MUTATIONS)).expect("mutations"),
    )
    .expect("typed mutations");
    assert_eq!(matrix.schema, 1);
    assert_eq!(matrix.cases.len(), 16);
    let fixture = model::Fixture::new();
    let inputs = model::lock_model::inputs(root);
    for (path, bytes) in &inputs {
        let destination = fixture.0.join(path);
        fs::create_dir_all(destination.parent().expect("parent")).expect("parents");
        fs::write(destination, bytes).expect("frozen copy");
    }
    let source = std::str::from_utf8(&inputs["Cargo.lock"]).expect("lock UTF-8");
    let (rules, _) = model::lock_model::policies();
    let workspace = model::lock_model::workspace(&fixture.0);
    matrix
        .cases
        .into_iter()
        .map(|case| {
            let offset = match case.position.as_str() {
                "none" => {
                    assert!(case.entry.is_empty());
                    source.len()
                }
                "append" => source.len(),
                "prepend" => source.find("[[package]]").expect("first package"),
                "middle" => {
                    source
                        .match_indices("[[package]]")
                        .nth(40)
                        .expect("middle node")
                        .0
                }
                _ => panic!("unknown mutation position"),
            };
            let source = format!("{}{}{}", &source[..offset], case.entry, &source[offset..]);
            fs::write(fixture.0.join("Cargo.lock"), &source).expect("mutated lock");
            let original = model::original_check(&fixture.0);
            assert_eq!(
                original.as_ref().err(),
                case.original_error.as_ref(),
                "{} original",
                case.name
            );
            let native = model::lock_model::native(&fixture.0, &workspace, &rules);
            assert_eq!(
                native.as_ref().err(),
                case.native_error.as_ref(),
                "{} native",
                case.name
            );
            if let Ok(rows) = &native {
                assert_eq!(rows.len(), 5);
                assert!(
                    rows.iter()
                        .all(|row| row.satisfied && row.observed_count == 1)
                );
            }
            let mut hashes = model::lock_model::hashes(&inputs);
            hashes.insert("Cargo.lock".into(), sha256_hex(source.as_bytes()));
            Row {
                case: case.name,
                lock_source: source,
                inputs: hashes,
                original_error: original.err(),
                native_error: native.as_ref().err().cloned(),
                observations: native.unwrap_or_default(),
                stage: case.stage,
            }
        })
        .collect()
}
