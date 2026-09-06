"""Bind frozen owner-set evidence while checking syntax quantities independently of membership."""

import hashlib
from pathlib import Path
import runpy
import tomllib


SHARED = runpy.run_path(Path(__file__).with_name("rc9_method_evidence.py"))
ROOT, OWNER_IDS = SHARED["ROOT"], SHARED["OWNER_IDS"]
bound_inputs, inputs, observe = SHARED["bound_inputs"], SHARED["inputs"], SHARED["observe"]
production, require = SHARED["production"], SHARED["require"]


POLICY = "rust:inventory:kd-transport-owners"
OWNER = "src/reactor/direct_plaintext/set_owner.rs"
SUBJECT = "ConnectionSet"


def members(counts):
    return None if counts is None else {key: 1 for key in counts}


def cases(roots):
    # Six source paths: DirectSet's aliased type, construction, and four selector methods.
    # These quantities validate native evidence; the original assertion requires only membership.
    expected = {OWNER + ":" + SUBJECT: 6}
    result = {"valid": (dict(expected), {}, [])}
    syntax = [2, 2, 1, 1, 1, 2, 2, 0, 0, 1, 1, 1, 1, 0, 1, 2, 2, 0, 0, 3, 1, 0, 0, 0, 3, 2]
    for index, quantity in enumerate(syntax):
        result[f"syntax-{index:02}"] = (expected | ({"src/rc9_adversary.rs:ConnectionSet": quantity}
                                                  if quantity else {}), {"src/rc9_adversary.rs": True}, [])
    result["original-detector"] = (expected | {"src/reactor/rogue.rs:ConnectionSet": 1},
                                   {"src/reactor/rogue.rs": True}, [])
    for root in roots:
        path = root + "/rc9_fresh.rs"
        result["new-file-" + root] = (expected | ({path + ":ConnectionSet": 1} if production(path) else {}),
                                    {path: True}, [])
    result["test-path-exclusion"] = (dict(expected), {"src/rc9_test.rs": True}, [])
    for name in ["parse-malformed", "parse-expression-fragment"]:
        result[name] = (None, {"src/rc9_adversary.rs": False}, [])
    result["required-file-deleted"] = ({}, {}, [OWNER])
    result["owner-removed"] = ({}, {OWNER: True}, [])
    result["owner-moved"] = ({"src/rc9_moved.rs:ConnectionSet": 1}, {OWNER: True, "src/rc9_moved.rs": True}, [])
    result["owner-duplicated"] = ({OWNER + ":ConnectionSet": 7}, {OWNER: True}, [])
    for name, quantity in [("comment", 0), ("raw", 0), ("qualified", 1), ("reference", 1), ("types", 1)]:
        result[f"owner-{name}-only"] = ({OWNER + ":ConnectionSet": quantity} if quantity else {},
                                       {OWNER: True}, [])
    require(len(result) == 46, "incomplete typed owner matrix")
    return result


def verify(assertion, report, assertions, files):
    require(assertion["id"] in OWNER_IDS and report["schema"] == 1
            and report["legacy_measure"] == "distinct-owner-files", "wrong owner evidence measure")
    require(assertion["replacement"]["policy_ids"] == [POLICY]
            and assertion["replacement"]["expected_diagnostics"] == ["RUST-INVENTORY-001"],
            "wrong owner policy or diagnostic")
    live = assertions["KD-TRANSPORT-OWNERS"]
    require((live["repository"], live["commit"]) == (assertion["repository"], assertion["commit"]),
            "mixed owner snapshots")
    require(live["required_cardinality"]["exact_set"] == [OWNER]
            and live["matching_semantics"]["value"] == SUBJECT, "changed original ownership requirement")
    roots = live["selection"]["roots"]["roots"]
    policy = report["observation"]["policy"]
    require(policy["name"] == "kd-transport-owners" and policy["reason"].strip()
            and policy["include"] == sorted(root + "/**/*.rs" for root in roots)
            and policy["exclude"] == ["**/*_test.rs", "tests/**"] and policy["world"] == "authored"
            and policy["subject"] == {"kind": "written-paths-containing", "names": [SUBJECT]}
            and policy["assertion"] == {"kind": "exact-owners", "owners": [{"path": OWNER, "name": SUBJECT}]},
            "policy differs from original owner set")
    policy_bytes = (ROOT / "docs/rc9/policies/kafka-driver.transport-owners.fragment.toml").read_bytes()
    authored, = tomllib.loads(policy_bytes.decode())["source"]["rust"]["inventories"]
    for key in ["include", "exclude"]:
        authored[key].sort()
    require(report["policy_sha256"] == hashlib.sha256(policy_bytes).hexdigest() and authored == policy,
            "unbound owner policy bytes")
    origins = ROOT / "crates/zrail-testkit/tests/fixtures/rc9/transport-owners-origins.json"
    require(report["fixture_origins_sha256"] == hashlib.sha256(origins.read_bytes()).hexdigest(),
            "unbound original owner assertion")
    baseline = bound_inputs(report, live, files, roots)
    selected_inputs = {path: row for path, row in baseline.items() if production(path)}
    require(len(selected_inputs) == 479, "incomplete physical owner selection")
    expected = {OWNER + ":ConnectionSet": 6}
    require(report["legacy_counts"] == members(expected), "wrong original owner set")
    observe(report["observation"], policy, selected_inputs, expected)
    detector = assertions["KD-TRANSPORT-DETECTOR-OWNERS"]["required_cardinality"]["exact_set"]
    require(detector == ["src/reactor/rogue.rs"]
            and report["detector_counts"] == {path + ":ConnectionSet": 1 for path in detector},
            "original detector ownership mismatch")
    expected_cases = cases(roots)
    rows = report["fixtures"]
    require(len(rows) == len(expected_cases) and {row["case"] for row in rows} == set(expected_cases),
            "missing, extra or duplicate owner case")
    for row in rows:
        counts, parsed, removed = expected_cases[row["case"]]
        changed = inputs(row["changed_inputs"])
        require(row["policy_id"] == POLICY and changed.keys() == parsed.keys()
                and row["legacy_fixture_parses"] == parsed and row["removed_inputs"] == removed,
                "wrong owner mutation or original parser outcome")
        require(all(path not in baseline or value != baseline[path] for path, value in changed.items()),
                "owner fixture did not change input")
        require(row["legacy_counts"] == members(counts), "wrong original membership map")
        accepted = members(counts) == members(expected)
        diagnostic = None if accepted else "RUST-INVENTORY-002" if counts is None else "RUST-INVENTORY-001"
        require(row["native_accepted"] == row["legacy_accepted"] == accepted
                and row["diagnostic"] == diagnostic, "wrong owner outcome or unrelated diagnostic")
        expected_inputs = {path: value for path, value in (baseline | changed).items()
                           if path not in removed and production(path)}
        if counts is None:
            require(row["native"] is None, "partial owner inventory escaped parse failure")
        else:
            observe(row["native"], policy, expected_inputs, counts)
        if row["case"] == "original-detector":
            require(changed["src/reactor/rogue.rs"]["sha256"] == report["detector_source_sha256"],
                    "original owner detector bytes changed")
