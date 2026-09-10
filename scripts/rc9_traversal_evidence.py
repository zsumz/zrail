"""Bind the three traversal preconditions, without claiming full-policy or exact ordering parity."""

import gzip
from pathlib import PurePosixPath
import tarfile

from rc9_traversal_common import ROOT, ROOTS, POLICY, PREFIX, SUITES, canonical, execution, parse, require, sha
from rc9_traversal_cases import policies, validate_physical, validate_injected

INDEX = "docs/rc9/evidence/traversal-index.json"
ARTIFACT = "docs/rc9/evidence/traversal.json.gz"
LOGS = "docs/rc9/evidence/traversal-execution-logs.tar.gz"
FAILED_LOGS = "docs/rc9/evidence/traversal-initial-failure-logs.tar.gz"
PRODUCER = "ec8ac46392ab08e2b6cf80c3fb944961e89b52d7"
TREE = "39831085492981052d680632c9db2fc60e2d86d3"
PAYLOAD = "8f38573e3888798cbff42bb023cc8315c9a60096c023df35ce56d344d61777e1"
SCOPE = "three-precondition bundle; complete physical path multisets; separate native ordering and fail-closed differences; injected errors are not observed OS errors; no full-policy proof"
OPERATIONS = {"DIRECTORY": "fs::read_dir succeeds", "ENTRY": "every read_dir entry result succeeds", "TYPE": "every DirEntry::file_type succeeds"}
FIXTURES = ["policy_test.rs", "unix_test.rs", "injected_test.rs", "injected_oracle.rs", "qualification.rs"]


def validate_frozen(report, files):
    repository = "kafkars/kafka-driver"
    require(report["roots"] == ROOTS, "changed selected roots")
    for field, path in [("guardrails_sha256", "guardrails.toml"), ("support_sha256", "tests/guardrails/support.rs")]:
        require(report[field] == files[(repository, path)]["sha256"], "changed frozen source/configuration")
    selected = {path: row for (repo, path), row in files.items() if repo == repository
                and any(path.startswith(root + "/") for root in ROOTS)}
    require(all(row["mode"] in ["100644", "100755"] for row in selected.values()), "unqualified frozen entry mode")
    rust = {path: row["sha256"] for path, row in selected.items() if PurePosixPath(path).suffix == ".rs"}
    require(len(rust) == 772 and report["frozen_inputs"] == rust, "missing or changed frozen Rust input")
    require(report["frozen_paths"] == sorted(rust, key=lambda path: PurePosixPath(path).parts)
            and report["native_frozen_paths"] == sorted(rust), "changed complete multiset or observed ordering")
    entries = {path: "file" for path in selected}
    for path in selected:
        for parent in PurePosixPath(path).parents:
            if any(str(parent) == root or str(parent).startswith(root + "/") for root in ROOTS):
                entries[str(parent)] = "directory"
    require(len(entries) == 846, "changed complete frozen physical tree")
    policy_map = policies()
    require([row["policy_id"] for row in report["frozen_observations"]] == list(policy_map), "incomplete frozen policies")
    for observed in report["frozen_observations"]:
        identity = observed["policy_id"]
        require(observed["policy"] == policy_map[identity] and observed["analysis"] == "exact"
                and observed["claim"] == "physical-paths" and observed["satisfied"] is True, "wrong frozen observation")
        expected = {root: "directory" for root in ROOTS} if identity.endswith("roots") else entries
        require([(entry["path"], entry["kind"]) for entry in observed["entries"]] == sorted(expected.items()), "partial frozen traversal")
        require(all(entry["sha256"] is None and entry["bytes"] is None and entry["satisfied"] is True
                    for entry in observed["entries"]), "fabricated physical content inspection")


def validate_executions(report):
    require(type(report["total_tests"]) is int and report["total_tests"] == 629, "wrong executable inventory")
    require(len(report["executions"]) == len(SUITES) == 7, "incomplete exact suites")
    for row, (short, (ignored, scope, cases)) in zip(report["executions"], SUITES.items()):
        name = PREFIX + short
        outcome = {"test": name, "passed": 1, "failed": 0, "ignored": 0, "measured": 0, "filtered": 628}
        require(row == {"test": name, "arguments": [name, "--exact"] + (["--ignored"] if ignored else []),
                        "scope": scope, "cases": cases, "outcome": outcome}
                and type(row["cases"]) is int
                and all(type(value) is int for key, value in row["outcome"].items() if key != "test"), "wrong suite, skip or synthetic outcome")


def validate_logs(report):
    suffixes = ["", ".native.json", ".build.log", ".compiler.log", ".listing.log", ".snapshots-before.log", ".snapshots-after.log"]
    suffixes += [f".suite-{index:02}.log" for index in range(7)]
    expected = {f"traversal-{run}.json{suffix}" for run in ["b", "c"] for suffix in suffixes}
    with tarfile.open(ROOT / LOGS, "r:gz") as archive:
        members = archive.getmembers()
        require(len(members) == len(expected) and {item.name for item in members} == expected
                and all(item.isfile() and item.size <= 64 * 1024 * 1024 for item in members)
                and sum(item.size for item in members) <= 64 * 1024 * 1024, "foreign, duplicate or oversized execution logs")
        logs = {item.name: archive.extractfile(item).read() for item in members}
    added = {"binary_path", "total_tests", "listing_sha256", "build_command", "policy_path", "native_payload_sha256", "producer_inputs", "executions"}
    native = {key: value for key, value in report.items() if key not in added}
    require(sha(canonical(native)) == report["native_payload_sha256"], "unbound native report")
    for run in ["b", "c"]:
        prefix = f"traversal-{run}.json"
        require(sha(logs[prefix]) == PAYLOAD and parse(logs[prefix]) == report, "different independent repeat")
        require(sha(logs[prefix + ".native.json"]) == report["native_payload_sha256"], "changed native payload")
        listing = logs[prefix + ".listing.log"].split(b"\n--- stderr ---\n", 1)[0]
        require(sha(listing) == report["listing_sha256"] and len(listing.splitlines()) == 629, "changed executable listing")
        for index, row in enumerate(report["executions"]):
            stdout = logs[prefix + f".suite-{index:02}.log"].split(b"\n--- stderr ---\n", 1)[0]
            require(execution(stdout, 0, row["test"], 629) == row["outcome"], "execution log mismatch")


def verify_report(report, files):
    index = parse((ROOT / INDEX).read_bytes())
    require(index["repeat"] == {"runs": 2, "byte_identical": True}
            and index["cases"] == {"physical": 133, "injected": 96, "frozen_rust_files": 772}
            and index["assertions_verified"] == 3 and index["full_repository_qualified"] is False, "overstated index scope")
    require(index["payload_sha256"] == PAYLOAD and sha(canonical(report)) == PAYLOAD, "changed immutable traversal report")
    require(set(index["archives"]) == {ARTIFACT, LOGS, FAILED_LOGS}, "missing evidence archives")
    for path, digest in index["archives"].items():
        require(sha((ROOT / path).read_bytes()) == digest, "changed evidence archive")
    with gzip.open(ROOT / ARTIFACT, "rb") as stream:
        require(sha(stream.read(64 * 1024 * 1024 + 1)) == PAYLOAD, "changed canonical artifact")
    require(type(report["schema"]) is int and report["schema"] == 1 and report["full_repository_qualified"] is False,
            "overstated traversal scope")
    require(report["implementation_commit"] == PRODUCER and report["implementation_tree"] == TREE, "wrong signed producer")
    for key in ["snapshot", "implementation_commit", "implementation_tree", "test_binary_sha256", "binary_path",
                "policy_sha256", "rustc_version", "cargo_lock_sha256", "listing_sha256", "producer_inputs",
                "native_payload_sha256", "limitations", "build_command"]:
        require(report[key] == index[key], "unbound producer field: " + key)
    require(report["policy_path"] == POLICY and report["policy_sha256"] == sha((ROOT / POLICY).read_bytes()), "changed policy bundle")
    validate_frozen(report, files)
    validate_physical(report["physical"])
    validate_injected(report["injected"])
    validate_executions(report)
    validate_logs(report)


def verify(assertion, report, files):
    verify_report(report, files)
    name = assertion["id"].removeprefix("KD-TRAVERSAL-")
    require(name in OPERATIONS and assertion["repository"] == report["snapshot"]["repository"]
            and assertion["commit"] == report["snapshot"]["commit"]
            and assertion["disposition"] == "existing native rail", "unrelated assertion")
    source = assertion["source"]
    require(source["path"] == "tests/guardrails/support.rs" and source["instance"] is None
            and source["file_sha256"] == files[(assertion["repository"], source["path"])]["sha256"]
            and source["function"] == ["collect_rust_files"], "wrong frozen source")
    require(assertion["selection"]["roots"] == ROOTS and assertion["selection"]["extension"] == ".rs"
            and assertion["matching_semantics"]["operation"] == OPERATIONS[name], "changed original operation or selection")
    replacement = assertion["replacement"]
    require(replacement["policy_ids"] == list(policies()) and replacement["expected_diagnostics"] == ["REP-FILE-002", "REP-FILE-006"]
            and replacement["implemented"] is True and replacement["verified"] is True
            and replacement["full_snapshot_verified"] is False and not assertion["blockers"]
            and replacement["qualification_scope"] == SCOPE, "overstated assertion linkage")
    require(all("crates/zrail-rust/tests/rc9_traversal/" + file in replacement["fixtures"] for file in FIXTURES), "missing fixture linkage")
    require(replacement["traversal_evidence"] == {"artifact": ARTIFACT, "payload_sha256": PAYLOAD,
                                                "implementation_commit": PRODUCER}, "wrong traversal evidence linkage")
