//! Source failure stages do not stand in for independent native schema outcomes.

use super::{original, projection};
use serde::Deserialize;
use serde_json::Value;
use std::{fs, io::Read as _, path::PathBuf};
use zrail_core::{load_contract, sha256_hex};

const CASES: &str = "crates/zrail-testkit/tests/fixtures/rc9/registry-cases.json.gz";
const ORIGINS: &str = "crates/zrail-testkit/tests/fixtures/rc9/registry-origins.json";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Matrix {
    schema: u32,
    source: Vec<Source>,
    native: Vec<Native>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    name: String,
    mode: String,
    bytes: Vec<u8>,
    original_stage: String,
    normalized: Option<Value>,
    conversion_stage: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Native {
    name: String,
    mode: String,
    bytes: Vec<u8>,
    accepted: bool,
}

fn project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates")
        .parent()
        .expect("project")
        .to_path_buf()
}

fn matrix() -> Matrix {
    let bytes = fs::read(project().join(CASES)).expect("compressed cases");
    let mut source = Vec::new();
    flate2::read::GzDecoder::new(bytes.as_slice())
        .take(16 * 1024 * 1024)
        .read_to_end(&mut source)
        .expect("bounded cases");
    let matrix: Matrix = serde_json::from_slice(&source).expect("typed matrix");
    assert_eq!(matrix.schema, 1);
    matrix
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir()
            .canonicalize()
            .expect("canonical temp")
            .join(format!(
                "zrail-registry-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
        assert!(!root.exists(), "fresh test-owned directory");
        fs::create_dir(&root).expect("test-owned directory");
        Self(root)
    }

    fn put(&self, name: &str, mode: &str, bytes: &[u8]) {
        let path = self.0.join(name);
        match mode {
            "file" => fs::write(path, bytes).expect("test input"),
            "directory" => fs::create_dir(path).expect("non-file input"),
            "missing" => assert!(!path.exists()),
            _ => panic!("unknown case mode"),
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove only test-owned directory");
    }
}

pub(super) fn source_binding() {
    let origins: Value =
        serde_json::from_slice(&fs::read(project().join(ORIGINS)).expect("origins"))
            .expect("origin JSON");
    let copied =
        fs::read_to_string(project().join("crates/zrail-rust/tests/rc9_registry/original.rs"))
            .expect("exact original");
    for row in origins["extractions"].as_array().expect("extractions") {
        let snippet = row["excerpt"].as_str().expect("exact excerpt");
        assert!(copied.contains(snippet));
        assert_eq!(sha256_hex(snippet.as_bytes()), row["extracted_sha256"]);
    }
    assert_eq!(
        usize::BITS,
        64,
        "qualification covers supported 64-bit targets"
    );
}

pub(super) fn source_cases() {
    let matrix = matrix();
    assert_eq!(matrix.source.len(), 249);
    for case in matrix.source {
        let fixture = Fixture::new();
        fixture.put("guardrails.toml", &case.mode, &case.bytes);
        let result = std::panic::catch_unwind(|| original::load_guardrails(&fixture.0));
        let (stage, normalized) = match result {
            Ok(config) => ("pass", Some(projection::value(config))),
            Err(payload) => {
                let text = payload
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| payload.downcast_ref::<&str>().copied())
                    .expect("original error");
                let stage = if text.starts_with("read ") {
                    "read"
                } else if text.starts_with("parse guardrails.toml:") {
                    "parse"
                } else if text.contains("unsupported guardrails.toml schema") {
                    "schema"
                } else {
                    panic!("unrelated original failure: {text}")
                };
                (stage, None)
            }
        };
        assert_eq!(stage, case.original_stage, "{}", case.name);
        assert_eq!(
            normalized, case.normalized,
            "{}: complete typed fields",
            case.name
        );
        assert!(
            case.conversion_stage == stage
                || (stage == "pass" && case.conversion_stage == "authority")
        );
    }
}

pub(super) fn native_cases() {
    let matrix = matrix();
    assert_eq!(matrix.native.len(), 27);
    let mut accepted = Vec::new();
    for case in matrix.native {
        let fixture = Fixture::new();
        fixture.put("zrail.toml", &case.mode, &case.bytes);
        let result = load_contract(&fixture.0, std::path::Path::new("zrail.toml"));
        assert_eq!(
            result.is_ok(),
            case.accepted,
            "{}: {:?}",
            case.name,
            result.as_ref().err()
        );
        if case.name == "known-traversal-contract-blocker" {
            assert!(result.as_ref().expect_err("tracked blocker").to_string()
                .contains("file assertion \"kd-source-traversal\" has an empty or vacuous count constraint"));
        }
        if case.accepted {
            let contract = load_contract(&fixture.0, std::path::Path::new("zrail.toml"))
                .expect("accepted carrier")
                .contract;
            assert_eq!(contract.repository.files.len(), 118);
            assert_eq!(contract.dependencies.lock_packages.len(), 5);
            assert_eq!(
                contract
                    .source
                    .rust
                    .budgets
                    .as_ref()
                    .expect("budgets")
                    .overrides
                    .len(),
                3
            );
            let mut value = serde_json::to_value(contract).expect("complete native contract");
            value["schema"] = Value::from(1);
            accepted.push(value);
        }
    }
    assert_eq!(accepted.len(), 2);
    assert_eq!(
        accepted[0], accepted[1],
        "schema 2 adds no policy authority to this carrier"
    );
}
