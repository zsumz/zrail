"""Validate corrected retired-tree evidence without executing consumer code."""

import gzip
import sys
import tarfile
import tomllib

sys.dont_write_bytecode = True

from rc9_retired_boundary_common import (
    ROOT, POLICY, PREFIX, QUALIFIER, SUITES, canonical, execution, parse, require, sha,
)
from rc9_retired_evidence import CASES, MODULES, BACKEND, CONSTRUCTION, validate_observations

INDEX = "docs/rc9/evidence/retired-boundary-index.json"
ARTIFACT = "docs/rc9/evidence/retired-boundary.json.gz"
LOGS = "docs/rc9/evidence/retired-boundary-execution-logs.tar.gz"
PRODUCER = "147fa9b1efd0fa26f75c6066726d9d0866355979"
TREE = "58b62cf9ad4e69153efe7515e9f11865bbf9c62c"
PAYLOAD = "8a9462a6d77f55c1294853bd770eec8a25a65f7445eb3b83d681c1210114591f"
NAMES = {"broker_set", "plaintext", "poller", "resource", "tcp", "timer", "tls"}
SCOPE = "corrected non-directory predicates; Unix physical boundaries and explicitly injected errors; no full-policy or exact error-path parity"
FIXTURES = ["retired_boundary_test.rs", "retired_boundary_unix_test.rs", "retired_entry_test.rs"]


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
    require({p["name"].removeprefix("kd-retired-tree-") for p in result.values()
             if p["name"].startswith("kd-retired-tree-") and p["entry"] == "non-directory"} == NAMES,
            "incomplete corrected tree selectors")
    return result


def validate_ordinary(report, outer, files):
    require(type(report["schema"]) is int and report["schema"] == 1
            and report["full_repository_qualified"] is False, "overstated ordinary scope")
    for key in ["snapshot", "implementation_commit", "implementation_tree", "test_binary_sha256",
                "policy_sha256", "rustc_version", "cargo_lock_sha256"]:
        require(report[key] == outer[key], "ordinary producer or policy differs: " + key)
    require(report["fixture_origins_sha256"] == sha((ROOT / "crates/zrail-testkit/tests/fixtures/rc9/declarations-origins.json").read_bytes()),
            "unbound frozen origins")
    require(sha(canonical(report)) == outer["ordinary_sha256"], "changed ordinary payload")
    baseline = {path: files[("kafkars/kafka-driver", path)]["sha256"] for path in [MODULES, BACKEND, CONSTRUCTION]}
    require(report["input_hashes"] == baseline, "changed frozen ordinary inputs")
    expected = CASES["matrix"](baseline)
    require(len(report["fixtures"]) == len(expected) == 144
            and [row["case"] for row in report["fixtures"]] == list(expected), "incomplete ordinary case matrix")
    require(report["baseline"] == report["fixtures"][0], "changed frozen acceptance")
    policy_map = policies()
    for row in report["fixtures"]:
        inputs, errors, quantity = expected[row["case"]]
        require(row["inputs"] == inputs and row["diagnostics"] == errors, "wrong ordinary input or diagnostic")
        require(row["native_accepted"] is (not errors) and row["legacy_accepted"] is (not errors), "wrong ordinary outcome")
        validate_observations(row, policy_map, inputs, errors, quantity)


def validate_executions(report):
    expected = [(PREFIX + name, *spec) for name, spec in SUITES.items()]
    expected.append((QUALIFIER, True, "ordinary-frozen", 144))
    require(type(report["total_tests"]) is int and report["total_tests"] == 622, "wrong producer test inventory")
    require(len(report["executions"]) == len(expected) == 13, "missing or duplicated suites")
    for row, (name, ignored, scope, cases) in zip(report["executions"], expected):
        outcome = {"test": name, "passed": 1, "failed": 0, "ignored": 0, "measured": 0, "filtered": 621}
        require(row == {"test": name, "ignored_switch": ignored, "scope": scope, "cases": cases, "outcome": outcome}
                and type(row["ignored_switch"]) is bool and type(row["cases"]) is int
                and all(type(value) is int for key, value in row["outcome"].items() if key != "test"),
                "wrong suite identity, scope, ignored switch or execution outcome")


def validate_logs(report):
    # Never extract a supplied archive; accept only the exact bounded file set.
    suffixes = ["", ".ordinary.json", ".build.log", ".compiler.log", ".listing.log",
                ".snapshots-before.log", ".snapshots-after.log"]
    suffixes += [f".suite-{index:02}.log" for index in range(13)]
    expected = {f"boundary-{run}.json{suffix}" for run in ["a", "b"] for suffix in suffixes}
    with tarfile.open(ROOT / LOGS, "r:gz") as archive:
        members = archive.getmembers()
        require(len(members) == len(expected) and {item.name for item in members} == expected
                and all(item.isfile() and item.size <= 64 * 1024 * 1024 for item in members)
                and sum(item.size for item in members) <= 64 * 1024 * 1024, "foreign, duplicate or oversized execution logs")
        logs = {item.name: archive.extractfile(item).read() for item in members}
    for run in ["a", "b"]:
        prefix = f"boundary-{run}.json"
        require(sha(logs[prefix]) == PAYLOAD and parse(logs[prefix]) == report, "repeat differs from bound report")
        require(sha(logs[prefix + ".ordinary.json"]) == report["ordinary_sha256"], "changed original ordinary report")
        listing = logs[prefix + ".listing.log"].split(b"\n--- stderr ---\n", 1)[0]
        require(sha(listing) == report["listing_sha256"] and len(listing.splitlines()) == report["total_tests"],
                "unbound executable test listing")
        for index, row in enumerate(report["executions"]):
            stdout = logs[prefix + f".suite-{index:02}.log"].split(b"\n--- stderr ---\n", 1)[0]
            require(execution(stdout, 0, row["test"], report["total_tests"]) == row["outcome"], "log/outcome mismatch")


def verify_report(report, files):
    index = parse((ROOT / INDEX).read_bytes())
    require(index["repeat"] == {"runs": 2, "byte_identical": True}
            and index["cases"] == {"ordinary_frozen": 144, "ordinary_synthetic": 144,
                                   "physical": 147, "injected": 84}
            and index["assertions_verified"] == 7 and index["full_repository_qualified"] is False,
            "overstated index scope")
    require(index["payload_sha256"] == PAYLOAD and sha(canonical(report)) == PAYLOAD, "changed immutable boundary report")
    require(set(index["archives"]) == {ARTIFACT, LOGS}, "missing evidence archives")
    for path, digest in index["archives"].items():
        require(sha((ROOT / path).read_bytes()) == digest, "changed evidence archive")
    with gzip.open(ROOT / ARTIFACT, "rb") as stream:
        require(sha(stream.read(64 * 1024 * 1024 + 1)) == PAYLOAD, "changed canonical artifact")
    require(type(report["schema"]) is int and report["schema"] == 1
            and report["full_repository_qualified"] is False, "overstated boundary scope")
    require(report["implementation_commit"] == PRODUCER and report["implementation_tree"] == TREE,
            "wrong signed producer")
    for key in ["snapshot", "implementation_commit", "implementation_tree", "test_binary_sha256", "binary_path",
                "policy_sha256", "rustc_version", "cargo_lock_sha256", "listing_sha256", "producer_inputs",
                "ordinary_sha256", "limitations", "build_command"]:
        require(report[key] == index[key], "unbound producer field: " + key)
    require(report["policy_path"] == POLICY and report["policy_sha256"] == sha((ROOT / POLICY).read_bytes()),
            "changed corrected policy")
    validate_executions(report)
    validate_ordinary(report["ordinary"], report, files)
    validate_logs(report)


def verify(assertion, report, files):
    verify_report(report, files)
    name = assertion["id"].removeprefix("KD-RETIRED-TREE-")
    require(name in NAMES and assertion["repository"] == report["snapshot"]["repository"]
            and assertion["commit"] == report["snapshot"]["commit"]
            and assertion["disposition"] == "new engine capability", "unrelated assertion")
    source = assertion["source"]
    require(source["path"] == "tests/guardrails/release_graph.rs" and source["instance"] == name
            and source["file_sha256"] == files[(assertion["repository"], source["path"])]["sha256"]
            and source["function"] == ["legacy_transport_module_trees_are_absent"], "wrong original source")
    require(assertion["selection"] == {"root": "src/reactor/" + name, "extension": "rs"}
            and assertion["required_cardinality"] == {"exact": 0}
            and type(assertion["required_cardinality"]["exact"]) is int, "changed physical assertion")
    replacement = assertion["replacement"]
    require(replacement["policy_ids"] == ["repository:file:kd-retired-tree-" + name]
            and replacement["expected_diagnostics"] == ["REP-FILE-001", "REP-FILE-006"]
            and replacement["implemented"] is True and replacement["verified"] is True
            and replacement["full_snapshot_verified"] is False and not assertion["blockers"]
            and replacement["qualification_scope"] == SCOPE, "overstated or incomplete assertion linkage")
    require(all("crates/zrail-rust/tests/rc9_declarations/" + file in replacement["fixtures"] for file in FIXTURES),
            "missing physical or injected fixture linkage")
    require(replacement["retired_boundary_evidence"] == {"artifact": ARTIFACT, "payload_sha256": PAYLOAD,
            "implementation_commit": PRODUCER}, "wrong corrected evidence linkage")
    require(replacement["retired_parity_evidence"] == {"artifact": "docs/rc9/evidence/retired-parity.json.gz",
            "payload_sha256": "79b9363ed2a7f39378e14dfe00a886b79b093ff6f30580e7aaf968042830dc83",
            "implementation_commit": "06b0f185f072bbd9211816a0694f0812ab4188d6"}, "historical evidence relabeled")
