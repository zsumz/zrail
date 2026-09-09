"""Bind the fixed manifest-parent proof without inventing new license authority."""

import gzip
import tarfile
import tomllib

from rc9_metadata_parent_common import ROOT, POLICY, ORIGINS, PREFIX, SUITES, canonical, execution, parse, require, sha

INDEX = "docs/rc9/evidence/metadata-parent-index.json"
ARTIFACT = "docs/rc9/evidence/metadata-parent.json.gz"
LOGS = "docs/rc9/evidence/metadata-parent-execution-logs.tar.gz"
PRODUCER = "f40fd5c3efda8619cf7dbf14662fa90d7e7a39c9"
TREE = "3a5ad04d3dbdfbd64b007469af65ac4eb5d20e8f"
PAYLOAD = "6b595acb5c42228c37d78c0eefe327649ed6679af7e2bda2ebcb08856f8e18fb"
SCOPE = "one fixed-registry lexical parent precondition; three exact existing license-policy mappings, twelve absolute-anchor checks; no arbitrary discovery or full repository qualification"
PACKAGES = [
    ("kafka-driver", "Cargo.toml", "", "LICENSE"),
    ("kafka-driver-core", "crates/kafka-driver-core/Cargo.toml", "crates/kafka-driver-core", "crates/kafka-driver-core/LICENSE"),
    ("kafka-driver-transport", "crates/kafka-driver-transport/Cargo.toml", "crates/kafka-driver-transport", "crates/kafka-driver-transport/LICENSE"),
]
POLICIES = ["repository:file:kd-metadata-" + name + "-license-copy" for name, *_ in PACKAGES]
FIXTURES = ["crates/zrail-rust/tests/rc9_metadata_parent/" + name for name in ["model.rs", "policy_test.rs", "qualification.rs"]]


def validate_semantics(report, files):
    require(type(report["schema"]) is int and report["schema"] == 1
            and report["original_metadata_body_executed"] is True
            and type(report["original_parent_instances"]) is int and report["original_parent_instances"] == 3
            and report["full_repository_qualified"] is False, "overstated execution or scope")
    expected = [{"root_shape": shape, "name": name, "manifest": manifest, "parent": parent,
                 "license": license, "policy_id": policy}
                for shape in ["filesystem-root", "checkout", "nested-unicode", "lexical-only"]
                for (name, manifest, parent, license), policy in zip(PACKAGES, POLICIES)]
    require(report["path_proof"] == expected, "partial, reordered or changed path mapping")
    origins = parse((ROOT / ORIGINS).read_bytes())
    inputs = {row["path"]: row["sha256"] for row in origins["fixtures"]}
    require(len(inputs) == 11 and report["inputs"] == inputs, "incomplete original inputs")
    for path, digest in inputs.items():
        require(files[("kafkars/kafka-driver", path)]["sha256"] == digest, "foreign input bytes")
    require(report["source_sha256"] == files[("kafkars/kafka-driver", "tests/guardrails/release_metadata.rs")]["sha256"], "changed original source")
    rules = tomllib.loads((ROOT / POLICY).read_text())["repository"]["files"]
    rules = [{"entry": "file", "exclude": [], **row} for row in rules if row["name"].endswith("-license-copy")]
    require(["repository:file:" + rule["name"] for rule in rules] == POLICIES, "changed policy identities")
    require([row["policy_id"] for row in report["observations"]] == sorted(POLICIES), "missing or duplicate native observation")
    for rule, (_, _, _, license), policy in zip(rules, PACKAGES, POLICIES):
        require(rule["include"] == [license] and rule["exclude"] == [] and rule["entry"] == "file"
                and rule["predicate"] == {"kind": "bytes-equal", "other": "LICENSE", "utf8": True}
                and rule["predicate"]["utf8"] is True, "changed translated license authority")
        observed, = [row for row in report["observations"] if row["policy_id"] == policy]
        require(observed["policy"] == rule and observed["analysis"] == "exact" and observed["satisfied"] is True
                and observed["claim"] == "utf8-file-bytes" and observed["checkout_path"] is None
                and len(observed["entries"]) == 1, "partial or overstated native license input")
        for entry, path in [(observed["entries"][0], license), (observed["reference"], "LICENSE")]:
            require(entry["path"] == path and entry["kind"] == "file" and entry["resolved_path"] is None
                    and entry["sha256"] == inputs[path] and type(entry["bytes"]) is int and entry["bytes"] == 11357
                    and entry["valid_utf8"] is True and entry["satisfied"] is True, "unbound or invalid license bytes")


def validate_executions(report):
    require(type(report["total_tests"]) is int and report["total_tests"] == 639 and len(report["executions"]) == 5, "incomplete exact suite inventory")
    for row, (short, (ignored, scope, cases)) in zip(report["executions"], SUITES.items()):
        name = PREFIX + short
        outcome = {"test": name, "passed": 1, "failed": 0, "ignored": 0, "measured": 0, "filtered": 638}
        require(row == {"test": name, "arguments": [name, "--exact"] + (["--ignored"] if ignored else []),
                        "scope": scope, "cases": cases, "outcome": outcome}
                and type(row["cases"]) is int
                and all(type(value) is int for key, value in row["outcome"].items() if key != "test"), "wrong suite, skipped test or fabricated count")


def validate_logs(report):
    suffixes = ["", ".native.json", ".build.log", ".compiler.log", ".listing.log", ".snapshots-before.log", ".snapshots-after.log"]
    suffixes += [f".suite-{index:02}.log" for index in range(5)]
    expected = {f"parent-{run}.json{suffix}" for run in ["a", "b"] for suffix in suffixes}
    with tarfile.open(ROOT / LOGS, "r:gz") as archive:
        members = archive.getmembers()
        require(len(members) == len(expected) and {item.name for item in members} == expected
                and all(item.isfile() and item.size <= 64 * 1024 * 1024 for item in members)
                and sum(item.size for item in members) <= 64 * 1024 * 1024, "foreign, duplicate or oversized logs")
        logs = {item.name: archive.extractfile(item).read() for item in members}
    for run in ["a", "b"]:
        prefix = f"parent-{run}.json"
        require(sha(logs[prefix]) == PAYLOAD and parse(logs[prefix]) == report, "independent repeat differs")
        require(sha(logs[prefix + ".native.json"]) == report["native_payload_sha256"], "changed native report")
        native = parse(logs[prefix + ".native.json"])
        require(all(value == report[key] for key, value in native.items()), "native/aggregate mismatch")
        listing = logs[prefix + ".listing.log"].split(b"\n--- stderr ---\n", 1)[0]
        require(sha(listing) == report["listing_sha256"] and len(listing.splitlines()) == 639, "changed executable listing")
        for index, row in enumerate(report["executions"]):
            stdout = logs[prefix + f".suite-{index:02}.log"].split(b"\n--- stderr ---\n", 1)[0]
            require(execution(stdout, 0, row["test"], 639) == row["outcome"], "execution/log mismatch")


def verify_report(report, files):
    index = parse((ROOT / INDEX).read_bytes())
    require(index["repeat"] == {"runs": 2, "byte_identical": True} and index["path_checks"] == 12
            and index["original_parent_instances"] == 3 and index["assertions_verified"] == 1
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
            and report["origins_sha256"] == sha((ROOT / ORIGINS).read_bytes()), "changed existing policy or origin registry")
    validate_semantics(report, files)
    validate_executions(report)
    validate_logs(report)


def verify(assertion, report, files):
    verify_report(report, files)
    require(assertion["id"] == "KD-METADATA-PARENT" and assertion["repository"] == report["snapshot"]["repository"]
            and assertion["commit"] == report["snapshot"]["commit"] and assertion["disposition"] == "existing native rail", "unrelated parent assertion")
    source = assertion["source"]
    require(source["path"] == "tests/guardrails/release_metadata.rs" and source["file_sha256"] == report["source_sha256"]
            and source["function"] == ["public_package_metadata_is_complete"]
            and source["instance"] == [[name, manifest] for name, manifest, *_ in PACKAGES], "wrong fixed registry")
    require(assertion["selection"] == {"paths": [row[1] for row in PACKAGES]}
            and assertion["matching_semantics"] == {"kind": "Path::parent after joining an absolute workspace root",
                "proof": "Each hardcoded relative path is nonempty and ends in Cargo.toml. This branch cannot fail for the frozen registry; normalized repository file policy paths preserve the precondition."}
            and assertion["required_cardinality"] == {"exact": 3}
            and type(assertion["required_cardinality"]["exact"]) is int, "overstated parent precondition")
    replacement = assertion["replacement"]
    require(replacement["policy_ids"] == POLICIES and replacement["expected_diagnostics"] == []
            and replacement["implemented"] is True and replacement["verified"] is True
            and replacement["full_snapshot_verified"] is False and not assertion["blockers"]
            and replacement["qualification_scope"] == SCOPE, "overstated parent linkage")
    require(replacement["fixtures"] == FIXTURES, "changed proof fixtures")
    require(replacement["parent_evidence"] == {"artifact": ARTIFACT, "payload_sha256": PAYLOAD,
                                              "implementation_commit": PRODUCER}, "wrong parent evidence link")
