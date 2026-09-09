"""Validate bound raw detector evidence; never promote fixture policies into a consumer contract."""

import gzip
import tarfile
import tomllib

from rc9_dependency_detector_common import ROOT, POLICY, CASES, ORIGINS, PREFIX, SUITES, canonical, execution, parse, require, sha

INDEX = "docs/rc9/evidence/dependency-detector-index.json"
ARTIFACT = "docs/rc9/evidence/dependency-detector.json.gz"
LOGS = "docs/rc9/evidence/dependency-detector-execution-logs.tar.gz"
PRODUCER = "2a066fe2c9588bab3ccf737e0ff830111d7f9b79"
TREE = "9e14b141c64f48c7e60b4dae58dd87ebbd61246d"
PAYLOAD = "1b690baa1b241d2133f1dfbbecb23bc7ed8d7084171c111418431615a0645feb"
SCOPE = "two frozen inline detector assertions; fixture-only raw line policies, not a consumer contract or Cargo graph; complete name sets and duplicate-line counts"
IDENTITIES = {"KD-DEP-DETECTOR-EXACT-NAME": ("tokio", False, "kd-dep-detector-exact-name"),
              "KD-DEP-DETECTOR-PRESENT-NAME": ("bytes", True, "kd-dep-detector-present-name")}
FIXTURES = ["model.rs", "original_test.rs", "policy_test.rs", "qualification.rs"]


def policies():
    rows = tomllib.loads((ROOT / POLICY).read_text())["repository"]["files"]
    result = {}
    for row in rows:
        row = {"entry": "file", "exclude": [], **row}
        row["predicate"] = {"case": "sensitive", "count": None, **row["predicate"]}
        result["repository:file:" + row["name"]] = row
    require(list(result) == ["repository:file:" + row[2] for row in IDENTITIES.values()], "wrong fixture policies")
    return result


def validate_fixtures(report):
    matrix = parse((ROOT / CASES).read_bytes())
    require(matrix["schema"] == 1 and len(matrix["cases"]) == 32, "wrong matrix")
    cases = matrix["cases"]
    require(len({case["name"] for case in cases}) == 32 and len(report["fixtures"]) == 32
            and [row["case"] for row in report["fixtures"]] == [case["name"] for case in cases], "missing, duplicate or reordered case")
    policy_map = policies()
    for row, case in zip(report["fixtures"], cases):
        source = case["source"].encode()
        require(row["source_sha256"] == sha(source) and row["original_names"] == case["names"]
                and case["names"] == sorted(set(case["names"])), "changed input or complete original set")
        require([observed["policy_id"] for observed in row["observations"]] == list(policy_map), "missing native observation")
        expected = ["tokio" not in case["names"], "bytes" in case["names"]]
        diagnostics = [[identity, "REP-FILE-004"] for identity, satisfied in zip(policy_map, expected) if not satisfied]
        require(row["diagnostics"] == diagnostics, "unintended diagnostic")
        for observed, satisfied, count in zip(row["observations"], expected, case["counts"]):
            require(observed["policy"] == policy_map[observed["policy_id"]] and observed["analysis"] == "exact"
                    and observed["claim"] == "raw-utf8-lines" and observed["satisfied"] is satisfied
                    and observed["checkout_path"] is None and observed["reference"] is None, "overstated native policy")
            require(len(observed["entries"]) == 1, "incomplete physical fixture selection")
            entry = observed["entries"][0]
            require(entry["path"] == "detector.txt" and entry["kind"] == "file" and entry["resolved_path"] is None
                    and entry["sha256"] == sha(source) and type(entry["bytes"]) is int and entry["bytes"] == len(source)
                    and entry["satisfied"] is satisfied, "changed native input or outcome")
            offsets = entry["literal_offsets"]
            require(type(entry["literal_count"]) is int and entry["literal_count"] == count
                    and type(entry["omitted_literal_offsets"]) is int and entry["omitted_literal_offsets"] == max(count - 16, 0)
                    and len(offsets) == min(count, 16) and offsets == sorted(set(offsets))
                    and all(type(offset) is int and 0 <= offset < len(source) for offset in offsets), "changed count or offset sampling")


def validate_executions(report):
    require(type(report["total_tests"]) is int and report["total_tests"] == 634 and len(report["executions"]) == 4, "incomplete exact suite inventory")
    for row, (short, (ignored, scope, cases)) in zip(report["executions"], SUITES.items()):
        name = PREFIX + short
        outcome = {"test": name, "passed": 1, "failed": 0, "ignored": 0, "measured": 0, "filtered": 633}
        require(row == {"test": name, "arguments": [name, "--exact"] + (["--ignored"] if ignored else []),
                        "scope": scope, "cases": cases, "outcome": outcome}
                and type(row["cases"]) is int
                and all(type(value) is int for key, value in row["outcome"].items() if key != "test"), "wrong suite, skipped test or fabricated count")


def validate_logs(report):
    suffixes = ["", ".native.json", ".build.log", ".compiler.log", ".listing.log", ".snapshots-before.log", ".snapshots-after.log"]
    suffixes += [f".suite-{index:02}.log" for index in range(4)]
    expected = {f"detector-{run}.json{suffix}" for run in ["a", "b"] for suffix in suffixes}
    with tarfile.open(ROOT / LOGS, "r:gz") as archive:
        members = archive.getmembers()
        require(len(members) == len(expected) and {item.name for item in members} == expected
                and all(item.isfile() and item.size <= 64 * 1024 * 1024 for item in members)
                and sum(item.size for item in members) <= 64 * 1024 * 1024, "foreign, duplicate or oversized logs")
        logs = {item.name: archive.extractfile(item).read() for item in members}
    for run in ["a", "b"]:
        prefix = f"detector-{run}.json"
        require(sha(logs[prefix]) == PAYLOAD and parse(logs[prefix]) == report, "independent repeat differs")
        require(sha(logs[prefix + ".native.json"]) == report["native_payload_sha256"], "changed native report")
        native = parse(logs[prefix + ".native.json"])
        require(all(value == report[key] for key, value in native.items()), "native/aggregate mismatch")
        listing = logs[prefix + ".listing.log"].split(b"\n--- stderr ---\n", 1)[0]
        require(sha(listing) == report["listing_sha256"] and len(listing.splitlines()) == 634, "changed executable listing")
        for index, row in enumerate(report["executions"]):
            stdout = logs[prefix + f".suite-{index:02}.log"].split(b"\n--- stderr ---\n", 1)[0]
            require(execution(stdout, 0, row["test"], 634) == row["outcome"], "execution/log mismatch")


def verify_report(report, files):
    index = parse((ROOT / INDEX).read_bytes())
    require(index["repeat"] == {"runs": 2, "byte_identical": True} and index["cases"] == 32
            and index["native_comparisons"] == 64 and index["assertions_verified"] == 2
            and index["full_repository_qualified"] is False, "overstated index scope")
    require(index["payload_sha256"] == PAYLOAD and sha(canonical(report)) == PAYLOAD, "changed immutable report")
    require(set(index["archives"]) == {ARTIFACT, LOGS}, "missing evidence archives")
    for path, digest in index["archives"].items():
        require(sha((ROOT / path).read_bytes()) == digest, "changed archive")
    with gzip.open(ROOT / ARTIFACT, "rb") as stream:
        require(sha(stream.read(64 * 1024 * 1024 + 1)) == PAYLOAD, "changed canonical artifact")
    require(type(report["schema"]) is int and report["schema"] == 1 and report["full_repository_qualified"] is False
            and type(report["original_assertions_executed"]) is int and report["original_assertions_executed"] == 2, "overstated original execution")
    require(report["implementation_commit"] == PRODUCER and report["implementation_tree"] == TREE, "wrong signed producer")
    for key in ["snapshot", "implementation_commit", "implementation_tree", "test_binary_sha256", "binary_path",
                "rustc_version", "cargo_lock_sha256", "listing_sha256", "producer_inputs", "native_payload_sha256", "limitations", "build_command"]:
        require(report[key] == index[key], "unbound producer: " + key)
    require(report["policy_path"] == POLICY and report["policy_sha256"] == sha((ROOT / POLICY).read_bytes())
            and report["cases_sha256"] == sha((ROOT / CASES).read_bytes())
            and report["origins_sha256"] == sha((ROOT / ORIGINS).read_bytes()), "changed policy, matrix or origin registry")
    origins = parse((ROOT / ORIGINS).read_bytes())["extractions"]
    require(report["source_sha256"] == files[("kafkars/kafka-driver", "tests/guardrails/dependency.rs")]["sha256"]
            and report["original_helper_sha256"] == origins[0]["extracted_sha256"]
            and report["original_detector_sha256"] == origins[1]["extracted_sha256"], "changed original source binding")
    validate_fixtures(report)
    validate_executions(report)
    validate_logs(report)


def verify(assertion, report, files):
    verify_report(report, files)
    require(assertion["id"] in IDENTITIES and assertion["repository"] == report["snapshot"]["repository"]
            and assertion["commit"] == report["snapshot"]["commit"] and assertion["disposition"] == "new engine capability", "unrelated detector")
    name, present, policy = IDENTITIES[assertion["id"]]
    source = assertion["source"]
    require(source["path"] == "tests/guardrails/dependency.rs" and source["file_sha256"] == report["source_sha256"]
            and source["function"] == ["package_extraction_matches_exact_names_only"] and source["instance"] is None, "wrong frozen detector")
    require(assertion["selection"] == {"paths": [], "keys": []}
            and assertion["matching_semantics"] == {"kind": "legacy lockfile_packages BTreeSet contains", "name": name,
                "present": present, "fixture": parse((ROOT / CASES).read_bytes())["cases"][0]["source"]}
            and assertion["matching_semantics"]["present"] is present
            and assertion["required_cardinality"] == {"present": present}
            and assertion["required_cardinality"]["present"] is present, "detector relabeled as live policy")
    replacement = assertion["replacement"]
    require(replacement["policy_ids"] == ["repository:file:" + policy] and replacement["expected_diagnostics"] == ["REP-FILE-004"]
            and replacement["implemented"] is True and replacement["verified"] is True
            and replacement["full_snapshot_verified"] is False and not assertion["blockers"]
            and replacement["qualification_scope"] == SCOPE, "overstated detector linkage")
    require(all("crates/zrail-rust/tests/rc9_dependency_detector/" + file in replacement["fixtures"] for file in FIXTURES), "missing fixtures")
    require(replacement["detector_evidence"] == {"artifact": ARTIFACT, "payload_sha256": PAYLOAD,
                                               "implementation_commit": PRODUCER}, "wrong detector evidence link")
