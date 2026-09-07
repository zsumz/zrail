//! Trusted compilation of the exact source-only test retains every `include_str!` precondition.

use std::{fs, path::PathBuf, process::Command};

use serde_json::{Value, json};
use zrail_core::sha256_hex;

use super::model::{self, FACADE, Fixture};

pub(super) const TEST: &str = "route_modules_cannot_own_a_selector_or_legacy_routing_capability";
pub(super) const SOURCE: &str = "src/reactor/direct_plaintext/cluster_runtime/route_state_test.rs";
pub(super) const SOURCE_SHA: &str =
    "f9d1bcc036579e23d55b7d9f23d4d2aa95929f1fbe514b6a1397799107675be3";

pub(super) struct Compiler {
    root: PathBuf,
    executable: PathBuf,
    pub(super) harness_sha256: String,
}

impl Compiler {
    pub(super) fn new(fixture: &Fixture) -> Self {
        let source = fs::read_to_string(
            model::project()
                .join("crates/zrail-testkit/tests/fixtures/rc9/route-sources/valid")
                .join(SOURCE),
        )
        .expect("frozen mixed test");
        assert_eq!(sha256_hex(source.as_bytes()), SOURCE_SHA);
        let start = source
            .find(&format!("#[test]\nfn {TEST}()"))
            .expect("exact test");
        let harness = &source[start..];
        let original = syn::parse_file(&source).expect("original AST");
        let extracted = syn::parse_file(harness).expect("extracted AST");
        assert_eq!(extracted.items.len(), 1);
        let function = original
            .items
            .iter()
            .find(|item| matches!(item, syn::Item::Fn(function) if function.sig.ident == TEST));
        assert_eq!(
            function,
            extracted.items.first(),
            "unchanged whole test AST"
        );
        fixture.write(SOURCE, harness);
        let compiler = Command::new("rustup")
            .args(["which", "rustc"])
            .current_dir(model::project())
            .output()
            .expect("resolve the project-pinned compiler before entering an external fixture");
        assert!(compiler.status.success());
        Self {
            root: fixture.root.clone(),
            executable: PathBuf::from(
                String::from_utf8(compiler.stdout)
                    .expect("compiler path")
                    .trim(),
            ),
            harness_sha256: sha256_hex(harness.as_bytes()),
        }
    }

    pub(super) fn compile(&self, failure: Option<(&str, bool)>) -> Value {
        // Explicit rustc invocation belongs only to this trusted qualification producer.
        let arguments = [
            "--edition=2024",
            "--test",
            "--error-format=json",
            "--color=never",
            SOURCE,
            "-o",
            "route-legacy-tests",
        ];
        let output = Command::new(&self.executable)
            .args(arguments)
            .current_dir(&self.root)
            .output()
            .expect("trusted pinned compiler");
        assert_eq!(output.status.success(), failure.is_none());
        assert!(output.stdout.is_empty());
        let errors = String::from_utf8(output.stderr.clone())
            .expect("compiler JSON")
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("compiler diagnostic"))
            .filter(|row| {
                row["level"] == "error" && !row["spans"].as_array().expect("spans").is_empty()
            })
            .collect::<Vec<_>>();
        if let Some((path, invalid_utf8)) = failure {
            assert_eq!(
                errors.len(),
                1,
                "only the intended include may fail: {errors:?}"
            );
            let error = &errors[0];
            let message = error["message"].as_str().expect("error message");
            assert!(
                message.contains(if invalid_utf8 {
                    "utf-8"
                } else {
                    "couldn't read"
                }),
                "{message}"
            );
            let literal = if path == FACADE {
                "../cluster_runtime.rs"
            } else {
                path.rsplit('/').next().expect("input name")
            };
            let located =
                error["spans"]
                    .as_array()
                    .expect("spans")
                    .iter()
                    .any(|span| {
                        if span["is_primary"] != true {
                            return false;
                        }
                        if invalid_utf8 {
                            span["file_name"] == literal
                                && span["byte_start"] == 0
                                && span["label"] == "byte `255` is not valid utf-8"
                        } else {
                            span["file_name"] == SOURCE
                                && span["text"].as_array().expect("source lines").iter().any(
                                    |line| {
                                        line["text"]
                                            .as_str()
                                            .expect("line")
                                            .contains(&format!("include_str!(\"{literal}\")"))
                                    },
                                )
                        }
                    });
            assert!(
                located,
                "failure must identify the selected include input: {error}"
            );
        } else {
            assert!(errors.is_empty());
        }
        let execution = failure.is_none().then(|| {
            let binary = self.root.join("route-legacy-tests");
            let list = Command::new(&binary)
                .args(["--list", "--format=terse"])
                .output()
                .expect("original runnable test inventory");
            assert!(list.status.success());
            assert_eq!(
                String::from_utf8(list.stdout.clone()).expect("test list"),
                format!("{TEST}: test\n")
            );
            let run = Command::new(&binary)
                .args(["--exact", TEST, "--nocapture"])
                .output()
                .expect("execute original source guard");
            assert!(run.status.success());
            let text = String::from_utf8(run.stdout).expect("test outcome");
            assert!(text.contains("1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"));
            json!({"test": TEST, "passed": 1, "failed": 0, "ignored": 0, "filtered": 0,
                   "binary_sha256": sha256_hex(&fs::read(binary).expect("test bytes")),
                   "list_sha256": sha256_hex(&list.stdout)})
        });
        json!({"executable": self.executable,
               "executable_sha256": sha256_hex(&fs::read(&self.executable).expect("compiler bytes")),
               "arguments": arguments, "exit_code": output.status.code(),
               "stderr_sha256": sha256_hex(&output.stderr), "errors": errors,
               "execution": execution})
    }
}
