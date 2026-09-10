"""Bind the exact lock-name precondition with explicit stricter native validation."""

import collections
import gzip
import re
import tarfile
import tomllib

from rc9_lock_names_common import ROOT, POLICY, CASES, MUTATIONS, ORIGINS, PREFIX, SUITES, canonical, execution, parse, require, sha

INDEX = "docs/rc9/evidence/lock-names-index.json"
ARTIFACT = "docs/rc9/evidence/lock-names.json.gz"
LOGS = "docs/rc9/evidence/lock-names-execution-logs.tar.gz"
PRODUCER = "9ee1bced403ea609d931718357ab847ed896c400"
TREE = "916596d7e24de0577224e3ebf084b0bd06e54ac9"
PAYLOAD = "372a5e4baf3b4e22f3dd6a10e666c1c5faf0f484245bae1c1059c467d4ec01d9"
SCOPE = "one all-entry whole-lock name-index precondition; 32 selection-stage cases and 16 complete-original frozen mutations preserve missing-name failures and explicit stricter non-string/blank native validation; no version-prefix or full repository qualification"
SOURCE = "tests/guardrails/protocol_provenance.rs"
POLICIES = ["dependency:lock-package:kd-lock-" + name for name in ["bornera", "bornera-core", "bornera-rustls", "kafka-wire", "kafka-wire-core"]]
FIXTURES = ["crates/zrail-rust/tests/rc9_lock_names/" + name for name in ["original.rs", "model.rs", "mutations.rs", "policy_test.rs", "qualification.rs"]]


def native_observations(rules, digest, count):
    return [{"policy_id": "dependency:lock-package:" + rule["name"], "policy": rule,
             "scope": "whole-cargo-lock", "quality": "exact", "lock_sha256": digest,
             "lock_node_count": count, "observed_count": 1,
             "observed_sample": rule["assertion"]["identities"], "observed_omitted": 0,
             "exact_difference": {"missing": [], "unexpected_count": 0,
                 "unexpected_sample": [], "unexpected_omitted": 0}, "satisfied": True}
            for rule in rules]


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
    require(report["cases_sha256"] == sha((ROOT / CASES).read_bytes())
            and report["mutations_sha256"] == sha((ROOT / MUTATIONS).read_bytes()), "changed case registries")
    matrix = parse((ROOT / CASES).read_bytes())["cases"]
    require(len(matrix) == len(report["fixtures"]) == len({row["case"] for row in report["fixtures"]}) == 32, "partial or duplicate matrix")
    require(collections.Counter(row["stage"] for row in matrix) == {
        "same-selection": 13, "missing-name": 6, "native-stricter-name": 10,
        "later-validation": 3}, "changed failure-stage coverage")
    for row, case in zip(report["fixtures"], matrix):
        expected = {"case": case["name"], "source_sha256": sha(case["source"].encode()),
                    **{key: case[key] for key in ["original_count", "original_error", "native_count", "native_error", "stage"]}}
        require(canonical(row) == canonical(expected), "changed complete selection or exact error stage")
    rules = tomllib.loads((ROOT / POLICY).read_text())["dependencies"]["lock_package"]
    rules.sort(key=lambda rule: rule["name"])
    require(["dependency:lock-package:" + rule["name"] for rule in rules] == POLICIES, "changed required policies")
    require(all(rule["assertion"]["kind"] == "exact" and len(rule["assertion"]["identities"]) == 1 for rule in rules), "changed identity authority")
    require(canonical(report["observations"]) == canonical(native_observations(rules, inputs["Cargo.lock"], 83)), "inexact frozen whole-lock observations")
    source = (ROOT / "crates/zrail-testkit/tests/fixtures/rc9/lock-packages/valid/Cargo.lock").read_text()
    require(sha(source.encode()) == inputs["Cargo.lock"], "foreign mutation source")
    offsets = [match.start() for match in re.finditer(re.escape("[[package]]"), source)]
    require(len(offsets) == 83, "partial frozen mutation scope")
    mutations = parse((ROOT / MUTATIONS).read_bytes())["cases"]
    require(len(mutations) == len(report["mutations"]) == len({row["case"] for row in report["mutations"]}) == 16, "partial or duplicate mutations")
    require(collections.Counter(row["stage"] for row in mutations) == {
        "same-acceptance": 5, "missing-name": 3, "native-stricter-name": 8}, "changed full-guard differences")
    for row, case in zip(report["mutations"], mutations):
        offset = {"none": len(source), "append": len(source), "prepend": offsets[0], "middle": offsets[40]}[case["position"]]
        changed = source[:offset] + case["entry"] + source[offset:]
        digest = sha(changed.encode())
        observed = [] if case["native_error"] is not None else native_observations(rules, digest, 83 + bool(case["entry"]))
        expected = {"case": case["name"], "lock_source": changed, "inputs": {**inputs, "Cargo.lock": digest},
                    "original_error": case["original_error"], "native_error": case["native_error"],
                    "stage": case["stage"], "observations": observed}
        require(canonical(row) == canonical(expected), "changed full guard, mutated inputs or native completeness")


def validate_executions(report):
    require(type(report["total_tests"]) is int and report["total_tests"] == 649 and len(report["executions"]) == 5, "incomplete exact suite inventory")
    for row, (short, (ignored, scope, cases)) in zip(report["executions"], SUITES.items()):
        name = PREFIX + short
        outcome = {"test": name, "passed": 1, "failed": 0, "ignored": 0, "measured": 0, "filtered": 648}
        require(row == {"test": name, "arguments": [name, "--exact"] + (["--ignored"] if ignored else []),
                        "scope": scope, "cases": cases, "outcome": outcome}
                and type(row["cases"]) is int
                and all(type(value) is int for key, value in row["outcome"].items() if key != "test"), "wrong suite, skipped test or fabricated count")


def validate_logs(report):
    suffixes = ["", ".native.json", ".build.log", ".compiler.log", ".listing.log", ".snapshots-before.log", ".snapshots-after.log"]
    suffixes += [f".suite-{index:02}.log" for index in range(5)]
    expected = {f"names-{run}.json{suffix}" for run in ["a", "b"] for suffix in suffixes}
    with tarfile.open(ROOT / LOGS, "r:gz") as archive:
        members = archive.getmembers()
        require(len(members) == len(expected) and {item.name for item in members} == expected
                and all(item.isfile() and item.size <= 64 * 1024 * 1024 for item in members)
                and sum(item.size for item in members) <= 64 * 1024 * 1024, "foreign, duplicate or oversized logs")
        logs = {item.name: archive.extractfile(item).read() for item in members}
    for run in ["a", "b"]:
        prefix = f"names-{run}.json"
        require(sha(logs[prefix]) == PAYLOAD and parse(logs[prefix]) == report, "independent repeat differs")
        require(sha(logs[prefix + ".native.json"]) == report["native_payload_sha256"], "changed native report")
        native = parse(logs[prefix + ".native.json"])
        require(all(value == report[key] for key, value in native.items()), "native/aggregate mismatch")
        listing = logs[prefix + ".listing.log"].split(b"\n--- stderr ---\n", 1)[0]
        require(sha(listing) == report["listing_sha256"] and len(listing.splitlines()) == 649, "changed executable listing")
        for index, row in enumerate(report["executions"]):
            stdout = logs[prefix + f".suite-{index:02}.log"].split(b"\n--- stderr ---\n", 1)[0]
            require(execution(stdout, 0, row["test"], 649) == row["outcome"], "execution/log mismatch")


def verify_report(report, files):
    index = parse((ROOT / INDEX).read_bytes())
    require(index["repeat"] == {"runs": 2, "byte_identical": True} and index["fixture_cases"] == 32
            and index["original_helper_invocations"] == 5 and index["frozen_mutation_cases"] == 16 and index["assertions_verified"] == 1
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
            and report["cases_sha256"] == sha((ROOT / CASES).read_bytes())
            and report["mutations_sha256"] == sha((ROOT / MUTATIONS).read_bytes()), "changed existing policy or origin registry")
    validate_semantics(report, files)
    validate_executions(report)
    validate_logs(report)


def verify(assertion, report, files):
    verify_report(report, files)
    require(assertion["id"] == "KD-PROVENANCE-LOCK-PACKAGE-NAMES"
            and assertion["repository"] == report["snapshot"]["repository"]
            and assertion["commit"] == report["snapshot"]["commit"]
            and assertion["disposition"] == "new engine capability", "unrelated name assertion")
    source = {"path": SOURCE, "file_sha256": report["source_sha256"],
              "candidate_id": "kafkars/kafka-driver:" + SOURCE + ":15:5:helper-call-candidate",
              "line": 15, "column": 5, "function": ["protocol_lock_entries_match_the_audited_registry_artifacts"], "instance": None,
              "syntax_sha256": "000d08ec719cc2e46aa7073c510b98954ce361103c6e6c1ad04c09e9af322b0c"}
    require(canonical(assertion["source"]) == canonical(source), "changed exact source assertion")
    require(assertion["invocations"] == ["kafkars/kafka-driver:" + SOURCE + f":{line}:5:helper-call-candidate"
            for line in [15, 21, 27, 33, 39]], "partial original invocations")
    matching = {"kind": "TOML indexing precondition inside whole-array filter",
                "source_expression": 'package["name"].as_str() == Some(name)', "helper_line": 150,
                "missing_name": "panics even on an otherwise unrelated node",
                "present_non_string_name": "ignored by name matching"}
    require(assertion["selection"] == {"paths": ["Cargo.lock"], "keys": ["package", "each entry", "name"]}
            and canonical(assertion["matching_semantics"]) == canonical(matching)
            and assertion["required_cardinality"] == {"scope": "each selected authored input"}, "overstated name precondition")
    replacement = assertion["replacement"]
    require(replacement["policy_ids"] == POLICIES and replacement["expected_diagnostics"] == []
            and replacement["implemented"] is True and replacement["verified"] is True
            and replacement["full_snapshot_verified"] is False and not assertion["blockers"]
            and replacement["qualification_scope"] == SCOPE, "overstated name linkage")
    require(replacement["fixtures"] == FIXTURES, "changed proof fixtures")
    require(replacement["names_evidence"] == {"artifact": ARTIFACT, "payload_sha256": PAYLOAD,
                                             "implementation_commit": PRODUCER}, "wrong name evidence link")
