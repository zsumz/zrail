"""Validate frozen trait/type/file inventory evidence without evaluating consumer source."""

import hashlib
from pathlib import Path
import runpy
import tomllib


SHARED = runpy.run_path(Path(__file__).with_name("rc9_method_evidence.py"))
ROOT, IMPL_IDS = SHARED["ROOT"], SHARED["IMPL_IDS"]
bound_inputs, inputs, observe = SHARED["bound_inputs"], SHARED["inputs"], SHARED["observe"]
production, require = SHARED["production"], SHARED["require"]
POLICY = "rust:inventory:kd-transport-impls"
OWNER = "src/reactor/direct_plaintext/rustls_transport.rs"
TYPE = "DirectRustlsTransport"
TRAITS = ["RegisteredTransport", "SlotTransport", "Source"]
EXPECTED = {f"{OWNER}:{name} for {TYPE}": 1 for name in TRAITS}


def members(counts):
    return None if counts is None else {key: 1 for key in counts}


def original_pairs(values):
    result = {}
    for value in values:
        path, type_name, trait = value.split(":")
        result[f"{path}:{trait} for {type_name}"] = 1
    return result


def cases(roots):
    result = {"valid": (dict(EXPECTED), {}, [])}
    syntax = [
        ("RegisteredTransport for State", 1), ("SlotTransport for Other", 1),
        ("Source for DirectRustlsTransport", 1), (None, 0), (None, 0), (None, 0),
        ("RegisteredTransport for State", 1), ("Source for DirectRustlsTransport", 1),
        ("Source for DirectRustlsTransport", 1), ("RegisteredTransport for State", 1),
        ("RegisteredTransport for State", 1), ("Source for DirectRustlsTransport", 1),
        (None, 0), (None, 0), (None, 0), (None, 0), (None, 0), (None, 0), (None, 0),
        ("RegisteredTransport for r#State", 1), (None, 0), (None, 0), (None, 0), (None, 0),
        ("RegisteredTransport for Alias", 1), ("Source for DirectRustlsTransport", 2),
        ("Source for DirectRustlsTransport", 3), ("RegisteredTransport for State", 2),
        ("Source for DirectRustlsTransport", 1), ("Source for DirectRustlsTransport", 1),
        ("Source for DirectRustlsTransport", 1), (None, 0), (None, 0),
    ]
    path = "src/rc9_adversary.rs"
    for index, (identity, count) in enumerate(syntax):
        result[f"syntax-{index:02}"] = (EXPECTED | ({path + ":" + identity: count} if count else {}),
                                       {path: True}, [])
    result["original-detector"] = (EXPECTED | {"src/reactor/rogue.rs:RegisteredTransport for Rogue": 1},
                                   {"src/reactor/rogue.rs": True}, [])
    for root in roots:
        fresh = root + "/rc9_fresh.rs"
        result["new-file-" + root] = (EXPECTED | ({fresh + ":Source for " + TYPE: 1}
                                                if production(fresh) else {}), {fresh: True}, [])
    result["test-path-exclusion"] = (dict(EXPECTED), {"src/rc9_test.rs": True}, [])
    for name in ["parse-malformed", "parse-expression-fragment"]:
        result[name] = (None, {path: False}, [])
    result["required-file-deleted"] = ({}, {}, [OWNER])
    for name in TRAITS:
        key = f"{OWNER}:{name} for {TYPE}"
        removed = {k: v for k, v in EXPECTED.items() if k != key}
        other = "RegisteredTransport" if name == "SlotTransport" else "SlotTransport"
        for case in ["impl-removed", "raw-trait", "comment-only", "macro-only"]:
            result[f"{case}-{name}"] = (dict(removed), {OWNER: True}, [])
        for case in ["qualified", "negative"]:
            result[f"{case}-{name}"] = (dict(EXPECTED), {OWNER: True}, [])
        result[f"duplicate-{name}"] = (EXPECTED | {key: 2}, {OWNER: True}, [])
        result[f"trait-exchanged-{name}"] = (removed | {f"{OWNER}:{other} for {TYPE}": 2}, {OWNER: True}, [])
        for case, type_name in [("type-exchanged", "Rc9Other"), ("raw-type", "r#" + TYPE)]:
            result[f"{case}-{name}"] = (removed | ({f"{OWNER}:{name} for {type_name}": 1}
                                                  if name != "Source" else {}), {OWNER: True}, [])
        result[f"moved-{name}"] = (removed | {f"src/rc9_moved.rs:{name} for {TYPE}": 1},
                                   {OWNER: True, "src/rc9_moved.rs": True}, [])
    require(len(result) == 78, "incomplete typed trait implementation matrix")
    return result


def verify(assertion, report, assertions, files):
    require(assertion["id"] in IMPL_IDS and report["schema"] == 1
            and report["legacy_measure"] == "distinct-trait-impl-identities", "wrong impl evidence measure")
    require(assertion["replacement"]["policy_ids"] == [POLICY]
            and assertion["replacement"]["expected_diagnostics"] == ["RUST-INVENTORY-001"],
            "wrong impl policy or diagnostic")
    live = assertions["KD-TRANSPORT-IMPLS"]
    require((live["repository"], live["commit"]) == (assertion["repository"], assertion["commit"]),
            "mixed impl snapshots")
    require(original_pairs(live["required_cardinality"]["exact_set"]) == EXPECTED,
            "changed original trait/type/file set")
    roots = live["selection"]["roots"]["roots"]
    policy = report["observation"]["policy"]
    owners = [{"path": OWNER, "name": f"{name} for {TYPE}"} for name in TRAITS]
    require(policy["name"] == "kd-transport-impls" and policy["reason"].strip()
            and policy["include"] == sorted(root + "/**/*.rs" for root in roots)
            and policy["exclude"] == ["**/*_test.rs", "tests/**"] and policy["world"] == "authored"
            and policy["subject"] == {"kind": "written-trait-impls", "names": TRAITS,
                                       "implementing_types": {"Source": [TYPE]}}
            and policy["assertion"] == {"kind": "exact-owners", "owners": owners},
            "policy differs from original trait implementation set")
    policy_bytes = (ROOT / "docs/rc9/policies/kafka-driver.transport-impls.fragment.toml").read_bytes()
    authored, = tomllib.loads(policy_bytes.decode())["source"]["rust"]["inventories"]
    for key in ["include", "exclude"]:
        authored[key].sort()
    require(report["policy_sha256"] == hashlib.sha256(policy_bytes).hexdigest() and authored == policy,
            "unbound trait implementation policy bytes")
    origins = ROOT / "crates/zrail-testkit/tests/fixtures/rc9/transport-impls-origins.json"
    require(report["fixture_origins_sha256"] == hashlib.sha256(origins.read_bytes()).hexdigest(),
            "unbound original trait implementation assertion")
    baseline = bound_inputs(report, live, files, roots)
    selected_inputs = {path: row for path, row in baseline.items() if production(path)}
    require(len(selected_inputs) == 479 and report["legacy_counts"] == EXPECTED, "wrong original impl set")
    observe(report["observation"], policy, selected_inputs, EXPECTED)
    detector = original_pairs(assertions["KD-TRANSPORT-DETECTOR-IMPLS"]["required_cardinality"]["exact_set"])
    require(detector == {"src/reactor/rogue.rs:RegisteredTransport for Rogue": 1}
            and report["detector_counts"] == detector, "original detector trait/type mismatch")
    expected_cases = cases(roots)
    rows = report["fixtures"]
    require(len(rows) == len(expected_cases) and {row["case"] for row in rows} == set(expected_cases),
            "missing, extra or duplicate impl case")
    for row in rows:
        counts, parsed, removed = expected_cases[row["case"]]
        changed = inputs(row["changed_inputs"])
        require(row["policy_id"] == POLICY and changed.keys() == parsed.keys()
                and row["legacy_fixture_parses"] == parsed and row["removed_inputs"] == removed,
                "wrong impl mutation or original parser outcome")
        require(all(path not in baseline or value != baseline[path] for path, value in changed.items()),
                "impl fixture did not change input")
        require(row["legacy_counts"] == members(counts), "wrong original trait/type membership")
        accepted = members(counts) == EXPECTED
        diagnostic = None if accepted else "RUST-INVENTORY-002" if counts is None else "RUST-INVENTORY-001"
        require(row["native_accepted"] == row["legacy_accepted"] == accepted
                and row["diagnostic"] == diagnostic, "wrong impl outcome or unrelated diagnostic")
        expected_inputs = {path: value for path, value in (baseline | changed).items()
                           if path not in removed and production(path)}
        if counts is None:
            require(row["native"] is None, "partial impl inventory escaped parse failure")
        else:
            observe(row["native"], policy, expected_inputs, counts)
        if row["case"] == "original-detector":
            require(changed["src/reactor/rogue.rs"]["sha256"] == report["detector_source_sha256"],
                    "original impl detector bytes changed")
