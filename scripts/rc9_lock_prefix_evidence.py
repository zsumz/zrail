"""Bind exact-prefix translation without assigning range semantics to literals."""

import gzip
import copy
import tarfile
import tomllib

from rc9_lock_prefix_common import ROOT, POLICY, CASES, ORIGINS, PREFIX, SUITES, canonical, execution, parse, require, sha

INDEX = "docs/rc9/evidence/lock-prefix-index.json"
ARTIFACT = "docs/rc9/evidence/lock-prefix.json.gz"
LOGS = "docs/rc9/evidence/lock-prefix-execution-logs.tar.gz"
PRODUCER = "bea4b64848214e3d50cc3ba2fb72a528ceedc02d"
TREE = "444c972758ab1690c475a0b08ae2835679a354b1"
PAYLOAD = "bca8476b88cfca98b4ef6b8b217ce767e3e67c7574f7b3b32c58c964d3b342d8"
SCOPE = "five frozen prefix registry instances linked to unchanged native exact identities; 120 paired representation cases, 20 frozen literal mismatches, 120 converter cases and 95 rejected linkage mutations; no general registry or full repository qualification"
SOURCE = "tests/guardrails/protocol_provenance.rs"
PACKAGES = ["bornera", "bornera-core", "bornera-rustls", "kafka-wire", "kafka-wire-core"]
POLICIES = ["dependency:lock-package:kd-lock-" + name for name in PACKAGES]
FIXTURES = ["crates/zrail-rust/tests/rc9_lock_prefix/" + name for name in ["original.rs", "model.rs", "native.rs", "matrix.rs", "policy_test.rs", "qualification.rs"]]


GRAPH_ERRORS = {'frozen': None, 'bare': None, 'caret': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \'^\' while parsing major version number', 'tilde': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \'~\' while parsing major version number', 'gte': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \'>\' while parsing major version number', 'wildcard': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \'*\' while parsing major version number', 'empty': 'Cargo.lock package requires string version', 'space-before': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \' \' while parsing major version number', 'fullwidth': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \'＝\' while parsing major version number', 'double-equals': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \'=\' while parsing major version number', 'equals-empty': 'Cargo.lock package requires string version', 'equals-caret': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \'^\' while parsing major version number', 'equals-tilde': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \'~\' while parsing major version number', 'equals-gte': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \'>\' while parsing major version number', 'equals-wildcard': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \'*\' while parsing major version number', 'equals-leading-space': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \' \' while parsing major version number', 'equals-trailing-space': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \' \' after pre-release identifier', 'partial': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected end of input while parsing minor version number', 'leading-zero': 'Cargo.lock package "$PACKAGE" has invalid version: invalid leading zero in major version number', 'newline': 'Cargo.lock package "$PACKAGE" has invalid version: unexpected character \'\\n\' after pre-release identifier', 'stable': None, 'prerelease': None, 'build': None, 'prerelease-build': None}


def observation(rule, digest, nodes=83):
    return {"policy_id": "dependency:lock-package:" + rule["name"], "policy": rule,
            "scope": "whole-cargo-lock", "quality": "exact", "lock_sha256": digest,
            "lock_node_count": nodes, "observed_count": 1, "observed_sample": rule["assertion"]["identities"],
            "observed_omitted": 0, "exact_difference": {"missing": [], "unexpected_count": 0,
                "unexpected_sample": [], "unexpected_omitted": 0}, "satisfied": True}


def validate_semantics(report, files):
    require(type(report["schema"]) is int and report["schema"] == 1
            and type(report["original_helper_invocations"]) is int and report["original_helper_invocations"] == 5
            and report["full_repository_qualified"] is False, "overstated original execution or scope")
    origins_path = "crates/zrail-testkit/tests/fixtures/rc9/lock-packages-origins.json"
    origins = parse((ROOT / origins_path).read_bytes())
    require(report["original_origins_sha256"] == sha((ROOT / origins_path).read_bytes()), "changed original registry")
    inputs = {row["path"]: row["sha256"] for row in origins["fixtures"]}
    require(len(inputs) == 12 and report["inputs"] == inputs, "incomplete frozen inputs")
    for path, digest in inputs.items():
        require(files[("kafkars/kafka-driver", path)]["sha256"] == digest, "foreign frozen input")
    require(report["source_sha256"] == files[("kafkars/kafka-driver", SOURCE)]["sha256"], "changed original source")
    require(report["workspace_packages"] == {name: "." if name == "kafka-driver" else "crates/" + name
            for name in ["kafka-driver", "kafka-driver-core", "kafka-driver-probe", "kafka-driver-sim", "kafka-driver-transport"]}, "partial workspace")
    require(report["cases_sha256"] == sha((ROOT / CASES).read_bytes()), "changed cases")
    rules = tomllib.loads((ROOT / POLICY).read_text())["dependencies"]["lock_package"]
    rules.sort(key=lambda rule: rule["name"])
    require(["dependency:lock-package:" + rule["name"] for rule in rules] == POLICIES, "changed policy identities")
    require(canonical(report["observations"]) == canonical([observation(rule, inputs["Cargo.lock"]) for rule in rules]), "inexact frozen observations")
    cases = parse((ROOT / CASES).read_bytes())["cases"]
    require(len(cases) == 24 and len(report["fixtures"]) == 120, "partial prefix matrix")
    base = tomllib.loads((ROOT / "crates/zrail-testkit/tests/fixtures/good/zrail.toml").read_text())
    for row, (rule, case) in zip(report["fixtures"], [(rule, case) for rule in rules for case in cases]):
        identity = rule["assertion"]["identities"][0]
        authored = case["template"].replace("$VERSION", identity["version"])
        suffix = authored[1:] if authored.startswith("=") else None
        prefix_error = None if suffix is not None else rule["package"] + " guardrail version must be exact"
        literal = authored if suffix is None else suffix
        expected = {"policy_id": "dependency:lock-package:" + rule["name"], "case": case["name"],
                    "input_version": authored, "suffix": suffix, "prefix_error": prefix_error,
                    "original_helper_error": prefix_error, "native_version": literal}
        require(canonical({key: value for key, value in row.items() if key != "native"}) == canonical(expected), "changed prefix or paired original helper")
        candidate = copy.deepcopy(rule)
        candidate["assertion"]["identities"][0]["version"] = literal
        native = row["native"]
        contract = copy.deepcopy(base)
        contract["dependencies"]["lock_package"] = [candidate]
        lock = {"version": 4, "package": [{"name": rule["package"], **identity, "version": literal}]}
        require(tomllib.loads(native["contract_source"]) == contract and tomllib.loads(native["lock_source"]) == lock,
                "different serialized native authority or paired lock")
        require(native["contract_sha256"] == sha(native["contract_source"].encode())
                and native["lock_sha256"] == sha(native["lock_source"].encode()), "unbound native input bytes")
        contract_error = None if case["native_contract_accepted"] else "lock identities require literal version/source strings of at most 1024/4096 bytes"
        require(native["contract_error"] == contract_error, "unrelated contract error")
        expected_graph_error = GRAPH_ERRORS[case["name"]]
        if expected_graph_error is not None:
            expected_graph_error = expected_graph_error.replace("$PACKAGE", rule["package"])
        require(native["graph_error"] == expected_graph_error, "changed exact Cargo version failure")
        expected_observation = observation(candidate, native["lock_sha256"], 1) if case["native_contract_accepted"] and case["native_graph_accepted"] else None
        require(canonical(native["observation"]) == canonical(expected_observation), "skipped, partial or nonliteral native result")
    require(len(report["literals"]) == 20, "partial frozen literal comparisons")
    index = 0
    for rule in rules:
        original_identity = rule["assertion"]["identities"][0]
        version = original_identity["version"]
        for case, literal in [("caret", "^" + version), ("wildcard", "*"), ("different", "9.9.9"), ("build-metadata", version + "+build.1")]:
            row = report["literals"][index]; index += 1
            candidate = copy.deepcopy(rule)
            candidate["assertion"]["identities"][0]["version"] = literal
            observed = observation(candidate, inputs["Cargo.lock"])
            observed.update(satisfied=False, observed_sample=[original_identity], exact_difference={
                "missing": candidate["assertion"]["identities"], "unexpected_count": 1,
                "unexpected_sample": [original_identity], "unexpected_omitted": 0})
            error = row["original_helper_error"]
            require(type(error) is str and "assertion `left == right` failed" in error
                    and version in error and literal in error and "guardrail version must be exact" not in error, "wrong original equality failure")
            require(canonical(row) == canonical({"policy_id": "dependency:lock-package:" + rule["name"],
                    "case": case, "expected_version": literal, "original_helper_error": error,
                    "native_diagnostic": "DEP-LOCK-001", "observation": observed}), "range or build metadata treated nonliterally")
    validate_conversion(report, rules, cases)


def validate_conversion(report, rules, cases):
    conversion = report["conversion"]
    require(report["translated_sha256"] == report["policy_sha256"] == sha((ROOT / POLICY).read_bytes()), "changed generated fragment")
    error = "reviewed exact-version prefix must bind the unchanged literal lock identity"
    expected = [{"policy_id": "dependency:lock-package:" + rule["name"], "case": case["name"],
                 "input_version": case["template"].replace("$VERSION", rule["assertion"]["identities"][0]["version"]),
                 "error": None if case["name"] == "frozen" else error} for rule in rules for case in cases]
    names = ["missing", "duplicate", "source-hash", "source-instance", "selection-path", "selection-key",
             "registry-path", "registry-hash", "registry-selector", "registry-index", "prefix-meaning",
             "literal-meaning", "literal-type", "typed-null", "typed-integer", "typed-boolean", "typed-array", "typed-table", "typed-string-array"]
    rejected = [{"package": rule["package"], "case": name, "error": "one reviewed exact-version prefix assertion is required"
                 if name in ["missing", "duplicate"] else error} for rule in rules for name in names]
    require(canonical(conversion) == canonical({"section": "dependencies.lock_package", "rules": rules,
            "cases": expected, "linkage_rejections": rejected}), "partial converter execution or changed authority accepted")


def validate_executions(report):
    require(type(report["total_tests"]) is int and report["total_tests"] == 654 and len(report["executions"]) == 5, "incomplete exact suite inventory")
    for row, (short, (ignored, scope, cases)) in zip(report["executions"], SUITES.items()):
        name = PREFIX + short
        outcome = {"test": name, "passed": 1, "failed": 0, "ignored": 0, "measured": 0, "filtered": 653}
        require(row == {"test": name, "arguments": [name, "--exact"] + (["--ignored"] if ignored else []),
                        "scope": scope, "cases": cases, "outcome": outcome}
                and type(row["cases"]) is int
                and all(type(value) is int for key, value in row["outcome"].items() if key != "test"), "wrong suite, skipped test or fabricated count")


def validate_logs(report):
    suffixes = ["", ".native.json", ".build.log", ".compiler.log", ".listing.log", ".snapshots-before.log", ".snapshots-after.log"]
    suffixes += [".translated.toml", ".translation.log", ".conversion.log"]
    suffixes += [f".suite-{index:02}.log" for index in range(5)]
    expected = {f"prefix-{run}.json{suffix}" for run in ["a", "b"] for suffix in suffixes}
    with tarfile.open(ROOT / LOGS, "r:gz") as archive:
        members = archive.getmembers()
        require(len(members) == len(expected) and {item.name for item in members} == expected
                and all(item.isfile() and item.size <= 64 * 1024 * 1024 for item in members)
                and sum(item.size for item in members) <= 64 * 1024 * 1024, "foreign, duplicate or oversized logs")
        logs = {item.name: archive.extractfile(item).read() for item in members}
    for run in ["a", "b"]:
        prefix = f"prefix-{run}.json"
        require(sha(logs[prefix]) == PAYLOAD and parse(logs[prefix]) == report, "independent repeat differs")
        require(sha(logs[prefix + ".native.json"]) == report["native_payload_sha256"], "changed native report")
        native = parse(logs[prefix + ".native.json"])
        require(all(value == report[key] for key, value in native.items()), "native/aggregate mismatch")
        require(logs[prefix + ".translated.toml"] == (ROOT / POLICY).read_bytes(), "changed CLI output")
        converted = logs[prefix + ".conversion.log"].split(b"\n--- stderr ---\n", 1)[0]
        require(canonical(parse(converted)) == canonical(report["conversion"]), "converter/log mismatch")
        translation = logs[prefix + ".translation.log"].split(b"\n--- stderr ---\n", 1)[0]
        require(translation.endswith(b"provenance lock translation: 5 policies\n"), "missing CLI translation success")
        listing = logs[prefix + ".listing.log"].split(b"\n--- stderr ---\n", 1)[0]
        require(sha(listing) == report["listing_sha256"] and len(listing.splitlines()) == 654, "changed executable listing")
        for index, row in enumerate(report["executions"]):
            stdout = logs[prefix + f".suite-{index:02}.log"].split(b"\n--- stderr ---\n", 1)[0]
            require(execution(stdout, 0, row["test"], 654) == row["outcome"], "execution/log mismatch")


def verify_report(report, files):
    index = parse((ROOT / INDEX).read_bytes())
    require(index["repeat"] == {"runs": 2, "byte_identical": True} and index["fixture_cases"] == 120
            and index["original_helper_invocations"] == 5 and index["literal_cases"] == 20 and index["conversion_cases"] == 120 and index["linkage_rejections"] == 95 and index["assertions_verified"] == 5
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
    matches = [package for package in PACKAGES if assertion["id"] == "KD-PROVENANCE-" + package + "-VERSION-PREFIX"]
    require(len(matches) == 1, "unrelated prefix assertion")
    package = matches[0]
    policy = "dependency:lock-package:kd-lock-" + package
    rule, = [row["policy"] for row in report["observations"] if row["policy_id"] == policy]
    value = "=" + rule["assertion"]["identities"][0]["version"]
    keys = ["dependencies", package.replace("-", "_") + "_version"]
    require(assertion["repository"] == report["snapshot"]["repository"] and assertion["commit"] == report["snapshot"]["commit"]
            and assertion["disposition"] == "new engine capability", "foreign prefix assertion")
    source = {"path": SOURCE, "file_sha256": report["source_sha256"], "candidate_id": "kafkars/kafka-driver:" + SOURCE + ":144:28:failure-macro",
              "line": 144, "column": 28, "function": ["assert_locked"], "instance": value,
              "syntax_sha256": "600ca7106ce77e9d9b11128d9664dae69b51ec2e9826749d6b82d19130ef96ef"}
    registry = {"path": "guardrails.toml", "file_sha256": report["inputs"]["guardrails.toml"], "selector": keys, "index": None, "entry": value}
    require(canonical(assertion["source"]) == canonical(source) and assertion["policy_registry"] == registry, "changed exact registry linkage")
    line = {"bornera": 27, "bornera-core": 33, "bornera-rustls": 39, "kafka-wire": 15, "kafka-wire-core": 21}[package]
    require(assertion["invocations"] == ["kafkars/kafka-driver:" + SOURCE + f":{line}:5:helper-call-candidate"], "wrong helper invocation")
    require(assertion["selection"] == {"paths": ["guardrails.toml"], "keys": keys}
            and canonical(assertion["matching_semantics"]) == canonical({"kind": "UTF-8 str::strip_prefix", "prefix": "=", "remaining_text_used_as_exact_lock_version": True})
            and assertion["required_cardinality"] == {"scope": "each selected authored input"}, "changed prefix meaning")
    replacement = assertion["replacement"]
    require(replacement["policy_ids"] == [policy] and replacement["expected_diagnostics"] == []
            and replacement["implemented"] is True and replacement["verified"] is True
            and replacement["full_snapshot_verified"] is False and not assertion["blockers"]
            and replacement["qualification_scope"] == SCOPE and replacement["fixtures"] == FIXTURES, "overstated prefix linkage")
    require(replacement["prefix_evidence"] == {"artifact": ARTIFACT, "payload_sha256": PAYLOAD, "implementation_commit": PRODUCER}, "wrong prefix evidence link")
