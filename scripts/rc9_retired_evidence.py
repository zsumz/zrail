"""Validate exact retired-backend evidence, without executing consumer code."""

import hashlib
from pathlib import Path
import runpy
import tomllib

ROOT = Path(__file__).resolve().parent.parent
CASES = runpy.run_path(ROOT / "scripts/rc9_retired_cases.py")
MODULES, BACKEND, CONSTRUCTION = [CASES[key] for key in ["MODULES", "BACKEND", "CONSTRUCTION"]]
POLICY = "docs/rc9/policies/kafka-driver.retired.fragment.toml"


def require(condition, message):
    if not condition:
        raise ValueError("retired evidence: " + message)


def policies():
    fragment = tomllib.loads((ROOT / POLICY).read_text())
    result = {}
    for policy in fragment["repository"]["files"]:
        policy = {"entry": "file", "exclude": [], **policy}
        if policy["predicate"]["kind"] == "literal":
            policy["predicate"] = {"case": "sensitive", "count": None, "normalization": "none", **policy["predicate"]}
        result["repository:file:" + policy["name"]] = policy
    for policy in fragment["source"]["rust"]["inventories"]:
        policy = {"exclude": [], **policy}
        policy["assertion"] = {"minimum": 0, **policy["assertion"]}
        result["rust:inventory:" + policy["name"]] = policy
    return result


def verify_report(report, files):
    require(type(report["schema"]) is int and report["schema"] == 1
            and report["full_repository_qualified"] is False, "overstated report scope")
    require(report["snapshot"] == {"name": "kafka-driver", "repository": "kafkars/kafka-driver",
            "commit": "a45be8071e6a663cd4ae3142cc5304ece9fdf45e", "tree": "5a6dbfe18b7f8a22c6cf902803f0f1f222480b2b"}, "wrong snapshot")
    require(report["implementation_commit"] == "06b0f185f072bbd9211816a0694f0812ab4188d6", "wrong qualified producer")
    require(report["implementation_tree"] == "77cf32ad242a74ad59887c98648ee77475516863"
            and report["test_binary_sha256"] == "949cbb0b38f5b68059ff5bd41d6faf48bcad45549b4b4d2493c7a61b461c8413"
            and report["cargo_lock_sha256"] == "ed9e20de4243fb159096670d50e413e8978538731fcfe62ee77ab8c88346636e"
            and report["fixture_origins_sha256"] == "f1b1afab2c17104f1d5b0387bfd6ec9d3015bf8b733d1b684c490451c37c4e0a",
            "unbound producer inputs")
    require(report["policy_sha256"] == hashlib.sha256((ROOT / POLICY).read_bytes()).hexdigest(), "unbound policy")
    require(report["rustc_version"] == "rustc 1.97.1 (8bab26f4f 2026-07-14)\nbinary: rustc\ncommit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452\ncommit-date: 2026-07-14\nhost: aarch64-apple-darwin\nrelease: 1.97.1\nLLVM version: 22.1.6", "unbound compiler")
    baseline = {path: files[("kafkars/kafka-driver", path)]["sha256"] for path in [MODULES, BACKEND, CONSTRUCTION]}
    require(report["input_hashes"] == baseline, "unbound original source inputs")
    expected = CASES["matrix"](baseline)
    require(len(report["fixtures"]) == len(expected)
            and [row["case"] for row in report["fixtures"]] == list(expected), "missing or reordered cases")
    require(report["baseline"] == report["fixtures"][0], "baseline differs from original acceptance")
    policy_map = policies()
    for row in report["fixtures"]:
        inputs, errors, quantity = expected[row["case"]]
        require(row["inputs"] == inputs and row["diagnostics"] == errors, "changed input or unrelated failure")
        require(row["native_accepted"] is (not errors) and row["legacy_accepted"] is (not errors), "wrong original/native outcome")
        validate_observations(row, policy_map, inputs, errors, quantity)


def validate_observations(row, policy_map, inputs, errors, quantity):
    failure = errors[0] if errors else None
    file_ids = sorted(key for key in policy_map if key.startswith("repository:"))
    if row["case"] == "input-construction-utf8":
        file_ids = []
    require([item["policy_id"] for item in row["files"]] == file_ids, "incomplete file observations")
    for item in row["files"]:
        identity = item["policy_id"]
        policy = policy_map[identity]
        literal = policy["predicate"]["kind"] == "literal"
        require(item["policy"] == policy and item["analysis"] == "exact"
                and item["claim"] == ("raw-utf8-text" if literal else "physical-paths")
                and item["satisfied"] is not (bool(failure) and failure[0] == identity), "wrong file policy or outcome")
        if literal:
            paths = [CONSTRUCTION] if CONSTRUCTION in inputs else []
        elif identity.endswith("inputs"):
            paths = sorted(set(inputs) & {MODULES, BACKEND, CONSTRUCTION})
        else:
            root = "src/reactor/" + policy["name"].removeprefix("kd-retired-tree-") + "/"
            paths = sorted(path for path in inputs if path.startswith(root)
                           and path.endswith(".rs") and Path(path).name != ".rs")
        require([entry["path"] for entry in item["entries"]] == paths, "changed file selection")
        for entry in item["entries"]:
            require(entry["kind"] == "file" and entry["resolved_path"] is None, "unbound link identity")
            if literal:
                count = int(bool(failure) and failure[0] == identity)
                require(entry["sha256"] == inputs[entry["path"]]
                        and type(entry["literal_count"]) is int and entry["literal_count"] == count
                        and entry["satisfied"] is (count == 0), "wrong raw input or count")
    inventory_ids = ["rust:inventory:kd-retired-modules", "rust:inventory:kd-retired-backend"]
    if failure and failure[1] == "RUST-INVENTORY-002":
        inventory_ids.remove(failure[0])
    require([item["policy_id"] for item in row["inventories"]] == inventory_ids, "incomplete Rust observations")
    for item in row["inventories"]:
        identity = item["policy_id"]
        policy = policy_map[identity]
        path = policy["include"][0]
        selected = {path: inputs[path]} if path in inputs else {}
        require(item["policy"] == policy and item["quality"] == "exact"
                and item["claim"] == ("authored-rust-file-module-syntax" if path == MODULES
                                      else "authored-rust-file-enum-variant-syntax"), "overstated syntax claim")
        require({entry["path"]: entry["sha256"] for entry in item["inputs"]} == selected
                and len(item["inputs"]) == len(selected), "unbound parsed input")
        count = quantity if failure and failure[0] == identity else 0
        name = row["case"].split("-")[1] if path == MODULES and count else "ReactorBackend::Legacy"
        counts = [{"path": path, "name": name, "count": count}] if count else []
        require(item["counts"] == counts and type(item["observed_count"]) is int
                and item["observed_count"] == count and item["satisfied"] is (count == 0)
                and len(item["occurrence_sample"]) == count and item["occurrences_omitted"] == 0,
                "wrong declaration identity or quantity")


def verify(assertion, report, files, assertions):
    verify_report(report, files)
    identity = assertion["id"]
    require(identity.startswith("KD-RETIRED-"), "unrelated assertion")
    replacement = assertion["replacement"]
    if "-TREE-" in identity:
        require(not replacement["verified"], "tree failure-path qualification remains open")
        return
    if "-MODULE-" in identity:
        require(assertions["KD-RETIRED-PARSE-MODULES"]["replacement"]["verified"], "unverified module input precondition")
        native = ["rust:inventory:kd-retired-modules"]
        diagnostics = ["RUST-INVENTORY-001"]
    elif identity.endswith("BACKEND-VARIANT"):
        require(assertions["KD-RETIRED-PARSE-BACKEND"]["replacement"]["verified"], "unverified backend input precondition")
        native = ["rust:inventory:kd-retired-backend"]
        diagnostics = ["RUST-INVENTORY-001"]
    elif "-CONSTRUCTION-" in identity:
        native = ["repository:file:kd-retired-construction-" + identity.split("-CONSTRUCTION-")[1], "repository:file:kd-retired-inputs"]
        diagnostics = ["REP-FILE-002", "REP-FILE-004", "REP-FILE-006"]
    else:
        require(identity in {"KD-RETIRED-PARSE-MODULES", "KD-RETIRED-PARSE-BACKEND"}, "unknown assertion")
        native = ["repository:file:kd-retired-inputs", "rust:inventory:kd-retired-" + identity.rsplit("-", 1)[1].lower()]
        diagnostics = ["REP-FILE-002", "RUST-INVENTORY-002"]
    require(replacement["policy_ids"] == native and replacement["expected_diagnostics"] == diagnostics,
            "wrong assertion policy or diagnostic linkage")
