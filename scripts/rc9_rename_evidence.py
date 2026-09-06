"""Validate frozen import-rename evidence without evaluating repository source."""

import hashlib
from pathlib import Path
import runpy
import tomllib


SHARED = runpy.run_path(Path(__file__).with_name("rc9_method_evidence.py"))
ROOT, RENAME_IDS = SHARED["ROOT"], SHARED["RENAME_IDS"]
bound_inputs, inputs, observe = SHARED["bound_inputs"], SHARED["inputs"], SHARED["observe"]
production, require = SHARED["production"], SHARED["require"]
POLICY = "rust:inventory:kd-transport-renames"
NAMES = ["ConnectionSet", "DirectSet", "RegisteredTransport", "SlotTransport", "Source"]
DETECTOR = dict(zip(NAMES, ["Set", "SetAlias", "Rt", "St", "IoSource"]))


def members(counts):
    return None if counts is None else {key: 1 for key in counts}


def cases(roots):
    result = {"valid": ({}, {}, [])}
    syntax = [
        {"Source as Io": 1}, {"Source as Source": 1}, {"Source as _": 1},
        {"Source as Io": 1, "Source as Other": 1, "ConnectionSet as Set": 1},
        {"Source as Io": 2}, {"ConnectionSet as Set": 1}, {"Source as Io": 1},
        {"Source as Io": 1}, {}, {}, {}, {"Source as r#Io": 1}, {}, {}, {}, {},
        {"Source as Io": 2}, {"Source as Io": 3}, {"Source as Io": 1}, {"Source as Io": 1}, {}, {},
    ]
    path = "src/rc9_adversary.rs"
    for index, counts in enumerate(syntax):
        result[f"syntax-{index:02}"] = ({path + ":" + name: count for name, count in counts.items()},
                                       {path: True}, [])
    result["original-detector"] = ({f"src/reactor/rogue.rs:{name} as {alias}": 1
                                   for name, alias in DETECTOR.items()}, {"src/reactor/rogue.rs": True}, [])
    for root in roots:
        fresh = root + "/rc9_fresh.rs"
        result["new-file-" + root] = ({fresh + ":Source as Io": 1} if production(fresh) else {},
                                    {fresh: True}, [])
    result["test-path-exclusion"] = ({}, {"src/rc9_test.rs": True}, [])
    for name in ["parse-malformed", "parse-expression-fragment"]:
        result[name] = (None, {path: False}, [])
    for name in NAMES:
        for case, alias, count in [
            ("new-alias", "Alias", 1), ("same-name", name, 1), ("underscore", "_", 1),
            ("raw-source", "Alias", 0), ("raw-alias", "r#Alias", 1), ("duplicate", "Alias", 2),
            ("destination-only", name, 0), ("grouped", "Alias", 1), ("function-import", "Alias", 1),
        ]:
            result[f"{case}-{name}"] = ({f"{path}:{name} as {alias}": count} if count else {},
                                       {path: True}, [])
    result["selected-file-deleted"] = ({}, {}, ["src/reactor/direct_plaintext/set_owner.rs"])
    require(len(result) == 79, "incomplete typed rename matrix")
    return result


def verify(assertion, report, assertions, files):
    require(assertion["id"] in RENAME_IDS and report["schema"] == 1
            and report["legacy_measure"] == "distinct-rename-identities", "wrong rename evidence measure")
    require(assertion["replacement"]["policy_ids"] == [POLICY]
            and assertion["replacement"]["expected_diagnostics"] == ["RUST-INVENTORY-001"],
            "wrong rename policy or diagnostic")
    live = assertions["KD-TRANSPORT-RENAMES"]
    require((live["repository"], live["commit"]) == (assertion["repository"], assertion["commit"]),
            "mixed rename snapshots")
    require(live["required_cardinality"]["exact_set"] == []
            and live["matching_semantics"]["guarded"] == NAMES, "changed original rename requirement")
    roots = live["selection"]["roots"]["roots"]
    policy = report["observation"]["policy"]
    require(policy["name"] == "kd-transport-renames" and policy["reason"].strip()
            and policy["include"] == sorted(root + "/**/*.rs" for root in roots)
            and policy["exclude"] == ["**/*_test.rs", "tests/**"] and policy["world"] == "authored"
            and policy["subject"] == {"kind": "written-import-renames", "names": NAMES}
            and policy["assertion"] == {"kind": "exact-owners", "owners": []},
            "policy differs from original rename ban")
    policy_bytes = (ROOT / "docs/rc9/policies/kafka-driver.transport-renames.fragment.toml").read_bytes()
    authored, = tomllib.loads(policy_bytes.decode())["source"]["rust"]["inventories"]
    for key in ["include", "exclude"]:
        authored[key].sort()
    require(report["policy_sha256"] == hashlib.sha256(policy_bytes).hexdigest() and authored == policy,
            "unbound rename policy bytes")
    origins = ROOT / "crates/zrail-testkit/tests/fixtures/rc9/transport-renames-origins.json"
    require(report["fixture_origins_sha256"] == hashlib.sha256(origins.read_bytes()).hexdigest(),
            "unbound original rename assertion")
    baseline = bound_inputs(report, live, files, roots)
    selected_inputs = {path: row for path, row in baseline.items() if production(path)}
    require(len(selected_inputs) == 479 and report["legacy_counts"] == {}, "wrong original rename set")
    observe(report["observation"], policy, selected_inputs, {})
    detector = {f"src/reactor/rogue.rs:{name} as {alias}": 1 for name, alias in DETECTOR.items()}
    require(assertions["KD-TRANSPORT-DETECTOR-RENAMES"]["required_cardinality"]["exact_set"] == sorted(detector)
            and report["detector_counts"] == detector, "original detector rename mismatch")
    expected_cases = cases(roots)
    rows = report["fixtures"]
    require(len(rows) == len(expected_cases) and {row["case"] for row in rows} == set(expected_cases),
            "missing, extra or duplicate rename case")
    for row in rows:
        counts, parsed, removed = expected_cases[row["case"]]
        changed = inputs(row["changed_inputs"])
        require(row["policy_id"] == POLICY and changed.keys() == parsed.keys()
                and row["legacy_fixture_parses"] == parsed and row["removed_inputs"] == removed,
                "wrong rename mutation or original parser outcome")
        require(all(path not in baseline or value != baseline[path] for path, value in changed.items()),
                "rename fixture did not change input")
        require(row["legacy_counts"] == members(counts), "wrong original rename membership")
        accepted = counts == {}
        diagnostic = None if accepted else "RUST-INVENTORY-002" if counts is None else "RUST-INVENTORY-001"
        require(row["native_accepted"] == row["legacy_accepted"] == accepted
                and row["diagnostic"] == diagnostic, "wrong rename outcome or unrelated diagnostic")
        expected_inputs = {path: value for path, value in (baseline | changed).items()
                           if path not in removed and production(path)}
        if counts is None:
            require(row["native"] is None, "partial rename inventory escaped parse failure")
        else:
            observe(row["native"], policy, expected_inputs, counts)
        if row["case"] == "original-detector":
            require(changed["src/reactor/rogue.rs"]["sha256"] == report["detector_source_sha256"],
                    "original rename detector bytes changed")
