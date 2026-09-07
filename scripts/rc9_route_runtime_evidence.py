"""Bind retained routing assertions to executed originals and an explicitly patched native plan."""

import json
import shlex
import sys

sys.dont_write_bytecode = True

from rc9_route_runtime_common import (
    FULL_NAME, NAME, PATCH, PATCHED, PIN, POLICY, PRODUCER, PROFILE, ROOT,
    input_paths, parse, policy, require, sha,
)
from rc9_route_runtime_mutations import receipt_cases

INDEX = "docs/rc9/evidence/route-runtime-index.json"
IDS = {"CONTROL-ATTEMPTS", "CONTROL-LANES", "CONTROL-OWNERS", "CONTROL-WORK", "FIRST-ROUTE",
       "INITIAL-LANES", "LONG-POLL-WORK", "PENDING-EMPTY", "QUEUED-REQUESTS", "REUSED-ATTEMPTS",
       "REUSED-LANES", "REUSED-OWNERS", "REUSED-RESOLUTION", "REUSED-SUBMISSION", "SECOND-ROUTE"}


def compact(value):
    return json.dumps(value, separators=(",", ":"), ensure_ascii=False).encode()


def ordered(value, keys):
    require(set(value) == set(keys), "unexpected plan field")
    return {key: value[key] for key in keys}


def validate_plan(plan, mirror, index):
    execution = ordered(mirror["execution"], ["command", "package", "default_features", "features", "target", "toolchain"])
    require(plan["schema"] == 1 and len(plan["mirrors"]) == 1, "wrong mirror plan scope")
    item, = plan["mirrors"]
    require(item == {"policy_id": index["policy_id"], "production": mirror["production"],
            "test": mirror["test"], "test_name": NAME, "receipt": mirror["receipt"],
            "inputs": mirror["inputs"], "input_sha256": index["input_sha256"],
            "execution": execution, "execution_group": sha(compact(execution))}, "plan differs from reviewed mirror")
    metrics = ordered(plan["analysis"], ["physical_rust_files", "physical_facts", "base_contexts", "derived_contexts",
                                        "projection_files", "projection_work", "projected_facts"])
    require(metrics == index["analysis"] and all(type(value) is int for value in metrics.values()), "changed source analysis")
    item = ordered(item, ["policy_id", "production", "test", "test_name", "receipt", "inputs", "input_sha256", "execution", "execution_group"])
    item["execution"] = execution
    require(sha(compact([1, plan["contract_sha256"], metrics, [item]])) == plan["plan_sha256"]
            == index["plan_sha256"], "unbound canonical plan")
    return item


def validate_execution(record, command, expected_name, expected):
    require(set(record) == {"arguments", "outcome", "binary_sha256", "listing_sha256", "source_root", "binary_path"}
            and record["arguments"] == command and record["binary_sha256"] == expected["binary_sha256"]
            and record["source_root"] == expected["source_root"] and record["binary_path"] == expected["binary_path"]
            and record["listing_sha256"] == sha(f"{expected_name}: test\n".encode()), "unbound exact Cargo execution")
    require(record["outcome"] == {"test": expected_name, "passed": 1, "failed": 0, "ignored": 0, "measured": 0, "filtered": 468}
            and all(type(value) is int for key, value in record["outcome"].items() if key != "test"), "wrong executed outcome")


def validate_mutations(rows, mirror, artifact, index):
    expected_receipts = receipt_cases(parse(artifact["source"]))
    inputs = [mirror["production"], mirror["test"], "src/reactor/direct_plaintext/cluster_runtime/route_test_support.rs",
              "Cargo.toml", "Cargo.lock", "README.md"]
    names = [case[0] for case in expected_receipts] + ["input-drift:" + path for path in inputs]
    names += ["test-renamed", "test-ignored"] + ["contract-execution-" + key for key in ["command", "features", "target", "toolchain"]] + ["plan-digest"]
    require(len(rows) == 30 and [row["case"] for row in rows] == names, "missing, duplicate or reordered negative cases")
    for row, (name, payload, diagnostic) in zip(rows, expected_receipts):
        require(row == {"case": name, "path": mirror["receipt"], "input_sha256": None if payload is None else sha(payload),
                "exit_code": 1, "error": None, "diagnostics": [diagnostic]}, "wrong receipt rejection or mutated payload")
    for row in rows[len(expected_receipts):]:
        require(row == index["plan_rejections"][row["case"]], "wrong source/context rejection or input binding")


def verify(assertion, report, files):
    index = parse((ROOT / INDEX).read_bytes())
    for path, digest in index["archives"].items():
        require(path.startswith("docs/rc9/evidence/") and ".." not in path.split("/"), "foreign evidence archive")
        require(sha((ROOT / path).read_bytes()) == digest, "changed execution or diagnostic archive")
    require(sha((json.dumps(report, indent=2, sort_keys=True) + "\n").encode()) == index["payload_sha256"], "changed immutable qualification payload")
    require(type(report["schema"]) is int and report["schema"] == 1 and report["snapshot"] == PIN
            and report["full_repository_qualified"] is False and report["untouched_snapshot_mirror_verified"] is False
            and report["patched_snapshot_mirror_verified"] is True, "overstated original or complete qualification")
    for key in ["implementation_commit", "implementation_tree", "zrail_binary_sha256", "cargo_lock_sha256", "rustc_version"]:
        require(report[key] == index[key], "unbound committed producer: " + key)
    for key, path in [("policy_sha256", POLICY), ("profile_sha256", PROFILE), ("patch_sha256", PATCH)]:
        require(report[key] == sha((ROOT / path).read_bytes()) == index[key], "unbound current review input")
    mirror = policy()
    paths = input_paths(mirror)
    frozen = {path: row["sha256"] for (repo, path), row in files.items() if repo == PIN["repository"]}
    require(sorted(frozen) == paths and report["original_inputs"] == frozen, "incomplete original tracked tree")
    require(report["patched_inputs"] == frozen | {PATCHED: index["patch_after_sha256"]}
            and frozen[PATCHED] == index["patch_before_sha256"], "undeclared downstream source mutation")
    require(report["original_planning"] == index["original_planning"] and report["unresolved"] == index["unresolved"], "original completeness boundary changed")
    item = validate_plan(report["plan"], mirror, index)
    command = shlex.split(mirror["execution"]["command"])
    validate_execution(report["original_execution"], command, FULL_NAME, index["original_execution"])
    validate_execution(report["patched_execution"], command, FULL_NAME, index["patched_execution"])
    require(report["original_execution"]["binary_path"] != report["patched_execution"]["binary_path"]
            and "/original/" in report["original_execution"]["binary_path"]
            and "/patched/" in report["patched_execution"]["binary_path"], "shared source-build outputs")
    helpers = ["measured_request_binds_permit_correlation_before_exact_frame_and_typed_completion",
               "measurement_failure_settles_typed_completion_without_a_context",
               "unsupported_version_settles_the_typed_completion_exactly_once",
               "binding_failure_leaves_the_context_as_the_only_typed_failure_owner"]
    require(len(report["helper_execution"]) == len(helpers), "missing helper regression execution")
    for record, test in zip(report["helper_execution"], helpers):
        name = "request::bornera_test::" + test
        validate_execution(record, [name if arg == FULL_NAME else arg for arg in command], name, index["patched_execution"])
    require(report["results"] == {"schema": 1, "plan_sha256": report["plan"]["plan_sha256"], "producer": PRODUCER,
            "groups": [{"execution_group": item["execution_group"], "tests": [{"policy_id": item["policy_id"], "status": "passed"}]}]}, "wrong trusted results")
    artifact = report["receipt_artifact"]
    require(artifact["path"] == mirror["receipt"] and artifact["policy_id"] == item["policy_id"]
            and artifact["sha256"] == sha(artifact["source"].encode()), "unbound receipt bytes")
    require(parse(artifact["source"]) == {"schema": 2, "producer": PRODUCER, "input_sha256": item["input_sha256"],
            "execution": mirror["execution"], "tests": [{"id": NAME, "status": "passed"}]}, "unbound exact receipt")
    require(report["verification"] == index["verification"] and report["verification"]["exit_code"] == 0
            and report["verification"]["output"]["report"]["status"] == "pass"
            and not report["verification"]["output"]["report"]["findings"], "native receipt verification failed")
    validate_mutations(report["mutations"], mirror, artifact, index)
    require(assertion["id"].removeprefix("KD-ROUTE-RUNTIME-") in IDS
            and assertion["disposition"] == "behavioral evidence retained"
            and assertion["repository"] == PIN["repository"] and assertion["commit"] == PIN["commit"]
            and assertion["source"]["path"] == mirror["test"]
            and assertion["source"]["function"] == [NAME]
            and assertion["source"]["file_sha256"] == frozen[mirror["test"]], "unrelated runtime assertion")
    require(assertion["selection"] == {"rust_test": NAME, "paths": [mirror["test"]],
            "package": "kafka-driver", "target": "library-unit-tests"}
            and assertion["required_cardinality"] == {"required_test_outcomes": 1, "ignored": False},
            "changed retained execution requirement")
    replacement = assertion["replacement"]
    require(replacement["policy_ids"] == [item["policy_id"]]
            and replacement["expected_diagnostics"] == ["RECEIPT-001", "RECEIPT-002", "RECEIPT-003", "RECEIPT-004", "RECEIPT-005", "RECEIPT-007"]
            and replacement["required_source_patch"] == PATCH
            and replacement["qualification_scope"] == "original execution and proposed patched-snapshot native mirror"
            and replacement["full_snapshot_verified"] is False, "overstated assertion linkage or omitted patch prerequisite")
