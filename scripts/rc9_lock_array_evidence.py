"""Bind the exact lock-array precondition without conflating other graph validation stages."""

import collections
import gzip
import tarfile
import tomllib

from rc9_lock_array_common import ROOT, POLICY, CASES, ORIGINS, PREFIX, SUITES, canonical, execution, parse, require, sha

INDEX = "docs/rc9/evidence/lock-array-index.json"
ARTIFACT = "docs/rc9/evidence/lock-array.json.gz"
LOGS = "docs/rc9/evidence/lock-array-execution-logs.tar.gz"
PRODUCER = "a385ebb5e01d8e92fbcbff834b29b3f5c5080238"
TREE = "86436f9057ab0025667cf0a914c78c4263cdbb9d"
PAYLOAD = "0a8700af9ed45f74581a41106c904f6a63a66025009186d2fbf58563ecf8125a"
SCOPE = "one whole-lock array-type precondition; 24 precise failure-stage fixtures, five separate empty-array required-count failures and five exact frozen identities over 83 nodes; no package-name or version-prefix qualification"
SOURCE = "tests/guardrails/protocol_provenance.rs"
POLICIES = ["dependency:lock-package:kd-lock-" + name for name in ["bornera", "bornera-core", "bornera-rustls", "kafka-wire", "kafka-wire-core"]]
FIXTURES = ["crates/zrail-rust/tests/rc9_lock_array/" + name for name in ["original.rs", "model.rs", "policy_test.rs", "qualification.rs"]]


def validate_semantics(report, files):
    require(type(report["schema"]) is int and report["schema"] == 1
            and type(report["original_helper_invocations"]) is int and report["original_helper_invocations"] == 5
            and report["full_repository_qualified"] is False, "overstated original execution or scope")
    origins_path = "crates/zrail-testkit/tests/fixtures/rc9/lock-packages-origins.json"
    origins = parse((ROOT / origins_path).read_bytes())
    require(report["original_origins_sha256"] == sha((ROOT / origins_path).read_bytes()), "changed complete original registry")
    inputs = {row["path"]: row["sha256"] for row in origins["fixtures"]}
    require(len(inputs) == 12 and report["inputs"] == inputs, "incomplete original inputs")
    for path, digest in inputs.items():
        require(files[("kafkars/kafka-driver", path)]["sha256"] == digest, "foreign frozen input")
    require(report["source_sha256"] == files[("kafkars/kafka-driver", SOURCE)]["sha256"], "changed original source")
    require(report["workspace_packages"] == {name: "." if name == "kafka-driver" else "crates/" + name
            for name in ["kafka-driver", "kafka-driver-core", "kafka-driver-probe", "kafka-driver-sim", "kafka-driver-transport"]}, "partial workspace")
    require(report["cases_sha256"] == sha((ROOT / CASES).read_bytes()), "changed matrix")
    matrix = parse((ROOT / CASES).read_bytes())["cases"]
    require(len(matrix) == len(report["fixtures"]) == len({row["case"] for row in report["fixtures"]}) == 24, "partial or duplicate matrix")
    require(collections.Counter(row["stage"] for row in matrix) == {
        "array-rejected": 10, "array-accepted": 5, "later-package-validation": 6,
        "earlier-version-validation": 3}, "changed failure-stage coverage")
    for row, case in zip(report["fixtures"], matrix):
        expected = {"case": case["name"], "source_sha256": sha(case["source"].encode()),
                    **{key: case[key] for key in ["array_count", "native_count", "native_error", "stage"]}}
        error = row["original_error"]
        if case["array_count"] is None:
            require(type(error) is str and bool(error), "missing original array rejection")
            expected_error = "index not found" if case["name"] in ["missing", "wrong-case", "nested-only"] else "Cargo.lock must contain package entries"
            require(error == expected_error, "unrelated original array failure")
        else:
            require(error is None, "array-typed input rejected by original precondition")
        expected["original_error"] = error
        require(canonical(row) == canonical(expected), "changed input, count or precise failure stage")
    rules = tomllib.loads((ROOT / POLICY).read_text())["dependencies"]["lock_package"]
    rules.sort(key=lambda rule: rule["name"])
    require(["dependency:lock-package:" + rule["name"] for rule in rules] == POLICIES, "changed required policies")
    require(len(report["observations"]) == len(report["empty_counts"]) == 5, "partial native inventory")
    for rule, observed, empty in zip(rules, report["observations"], report["empty_counts"]):
        policy = "dependency:lock-package:" + rule["name"]
        require(rule["assertion"]["kind"] == "exact" and len(rule["assertion"]["identities"]) == 1, "changed identity authority")
        identities = rule["assertion"]["identities"]
        base = {"policy_id": policy, "policy": rule, "scope": "whole-cargo-lock", "quality": "exact",
                "observed_omitted": 0, "exact_difference": {"missing": [], "unexpected_count": 0,
                    "unexpected_sample": [], "unexpected_omitted": 0}}
        expected = {**base, "lock_sha256": inputs["Cargo.lock"], "lock_node_count": 83,
                    "observed_count": 1, "observed_sample": identities, "satisfied": True}
        require(canonical(observed) == canonical(expected), "inexact frozen whole-lock observation")
        expected.update(lock_sha256=sha(b"version = 4\npackage = []\n"), lock_node_count=0,
                        observed_count=0, observed_sample=[], satisfied=False,
                        exact_difference={**base["exact_difference"], "missing": identities})
        error = empty["original_error"]
        require(type(error) is str and "must resolve exactly once" in error
                and "must contain package entries" not in error and rule["package"] in error, "wrong original count failure")
        require(canonical(empty) == canonical({"policy_id": policy, "original_error": error,
                "native_diagnostic": "DEP-LOCK-001", "observed": expected}), "count rejection is not array rejection")


def validate_executions(report):
    require(type(report["total_tests"]) is int and report["total_tests"] == 644 and len(report["executions"]) == 5, "incomplete exact suite inventory")
    for row, (short, (ignored, scope, cases)) in zip(report["executions"], SUITES.items()):
        name = PREFIX + short
        outcome = {"test": name, "passed": 1, "failed": 0, "ignored": 0, "measured": 0, "filtered": 643}
        require(row == {"test": name, "arguments": [name, "--exact"] + (["--ignored"] if ignored else []),
                        "scope": scope, "cases": cases, "outcome": outcome}
                and type(row["cases"]) is int
                and all(type(value) is int for key, value in row["outcome"].items() if key != "test"), "wrong suite, skipped test or fabricated count")


def validate_logs(report):
    suffixes = ["", ".native.json", ".build.log", ".compiler.log", ".listing.log", ".snapshots-before.log", ".snapshots-after.log"]
    suffixes += [f".suite-{index:02}.log" for index in range(5)]
    expected = {f"array-{run}.json{suffix}" for run in ["a", "b"] for suffix in suffixes}
    with tarfile.open(ROOT / LOGS, "r:gz") as archive:
        members = archive.getmembers()
        require(len(members) == len(expected) and {item.name for item in members} == expected
                and all(item.isfile() and item.size <= 64 * 1024 * 1024 for item in members)
                and sum(item.size for item in members) <= 64 * 1024 * 1024, "foreign, duplicate or oversized logs")
        logs = {item.name: archive.extractfile(item).read() for item in members}
    for run in ["a", "b"]:
        prefix = f"array-{run}.json"
        require(sha(logs[prefix]) == PAYLOAD and parse(logs[prefix]) == report, "independent repeat differs")
        require(sha(logs[prefix + ".native.json"]) == report["native_payload_sha256"], "changed native report")
        native = parse(logs[prefix + ".native.json"])
        require(all(value == report[key] for key, value in native.items()), "native/aggregate mismatch")
        listing = logs[prefix + ".listing.log"].split(b"\n--- stderr ---\n", 1)[0]
        require(sha(listing) == report["listing_sha256"] and len(listing.splitlines()) == 644, "changed executable listing")
        for index, row in enumerate(report["executions"]):
            stdout = logs[prefix + f".suite-{index:02}.log"].split(b"\n--- stderr ---\n", 1)[0]
            require(execution(stdout, 0, row["test"], 644) == row["outcome"], "execution/log mismatch")


def verify_report(report, files):
    index = parse((ROOT / INDEX).read_bytes())
    require(index["repeat"] == {"runs": 2, "byte_identical": True} and index["fixture_cases"] == 24
            and index["original_helper_invocations"] == 5 and index["empty_count_checks"] == 5 and index["assertions_verified"] == 1
            and index["full_repository_qualified"] is False, "overstated index scope")
    require(index["payload_sha256"] == PAYLOAD and sha(canonical(report)) == PAYLOAD, "changed immutable report")
    require(set(index["archives"]) == {ARTIFACT, LOGS}, "missing evidence archives")
    for path, digest in index["archives"].items():
        require(sha((ROOT / path).read_bytes()) == digest, "changed archive")
    with gzip.open(ROOT / ARTIFACT, "rb") as stream:
        require(sha(stream.read(64 * 1024 * 1024 + 1)) == PAYLOAD, "changed canonical artifact")
    require(report["implementation_commit"] == PRODUCER and report["implementation_tree"] == TREE, "wrong signed producer")
    for key in ["snapshot", "implementation_commit", "implementation_tree", "test_binary_sha256", "binary_path",
                "rustc_version", "cargo_lock_sha256", "listing_sha256", "producer_inputs", "native_payload_sha256", "limitations", "build_command"]:
        require(report[key] == index[key], "unbound producer: " + key)
    require(report["policy_path"] == POLICY and report["policy_sha256"] == sha((ROOT / POLICY).read_bytes())
            and report["origins_sha256"] == sha((ROOT / ORIGINS).read_bytes())
            and report["cases_sha256"] == sha((ROOT / CASES).read_bytes()), "changed existing policy or origin registry")
    validate_semantics(report, files)
    validate_executions(report)
    validate_logs(report)


def verify(assertion, report, files):
    verify_report(report, files)
    require(assertion["id"] == "KD-PROVENANCE-LOCK-PACKAGE-ARRAY"
            and assertion["repository"] == report["snapshot"]["repository"]
            and assertion["commit"] == report["snapshot"]["commit"]
            and assertion["disposition"] == "new engine capability", "unrelated array assertion")
    source = {"path": SOURCE, "file_sha256": report["source_sha256"],
              "candidate_id": "kafkars/kafka-driver:" + SOURCE + ":147:28:failure-macro",
              "line": 147, "column": 28, "function": ["assert_locked"], "instance": None,
              "syntax_sha256": "49c50a609f542b294e89d5ff5849771f81930e24d9dfb05b1a0a73890fb4be22"}
    require(canonical(assertion["source"]) == canonical(source), "changed exact source assertion")
    require(assertion["invocations"] == ["kafkars/kafka-driver:" + SOURCE + f":{line}:5:helper-call-candidate"
            for line in [15, 21, 27, 33, 39]], "partial original invocations")
    require(assertion["selection"] == {"paths": ["Cargo.lock"], "keys": ["package"]}
            and assertion["matching_semantics"] == {"kind": "TOML array type precondition",
                "empty_array": "accepted by this precondition; required package counts reject it"}
            and assertion["required_cardinality"] == {"scope": "each selected authored input"}, "overstated array precondition")
    replacement = assertion["replacement"]
    require(replacement["policy_ids"] == POLICIES and replacement["expected_diagnostics"] == []
            and replacement["implemented"] is True and replacement["verified"] is True
            and replacement["full_snapshot_verified"] is False and not assertion["blockers"]
            and replacement["qualification_scope"] == SCOPE, "overstated array linkage")
    require(replacement["fixtures"] == FIXTURES, "changed proof fixtures")
    require(replacement["array_evidence"] == {"artifact": ARTIFACT, "payload_sha256": PAYLOAD,
                                             "implementation_commit": PRODUCER}, "wrong array evidence link")
