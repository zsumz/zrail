"""Validate authored transport evidence bindings; never evaluate consumer source."""

import collections
import hashlib
from pathlib import Path
import runpy
import tomllib


IDS = {"KD-TRANSPORT-METHODS", "KD-TRANSPORT-DETECTOR-METHODS",
       "KD-TRANSPORT-PARSE-FIXTURE", "KD-TRANSPORT-PARSE-PRODUCTION"}
EXPRESSION_IDS = {"KD-TRANSPORT-ASSOCIATED", "KD-TRANSPORT-DETECTOR-ASSOCIATED"}
OWNER_IDS = {"KD-TRANSPORT-OWNERS", "KD-TRANSPORT-DETECTOR-OWNERS"}
RENAME_IDS = {"KD-TRANSPORT-RENAMES", "KD-TRANSPORT-DETECTOR-RENAMES"}
POLICY = "rust:inventory:kd-transport-methods"
ROOT = Path(__file__).resolve().parent.parent


def require(condition, message):
    if not condition:
        raise ValueError("transport evidence: " + message)


def inputs(rows):
    paths = [row["path"] for row in rows]
    require(paths == sorted(set(paths)), "duplicate or noncanonical input paths")
    for row in rows:
        require(set(row) == {"path", "sha256", "bytes"} and type(row["bytes"]) is int
                and row["bytes"] >= 0 and len(row["sha256"]) == 64
                and all(c in "0123456789abcdef" for c in row["sha256"]), "invalid input identity")
    return {row["path"]: row for row in rows}


def count_map(rows):
    keys = [(row["path"], row["name"]) for row in rows]
    require(keys == sorted(set(keys)), "duplicate or noncanonical file/subject counts")
    require(all(type(row["count"]) is int and row["count"] > 0 for row in rows), "invalid quantity")
    return {f"{row['path']}:{row['name']}": row["count"] for row in rows}


def selected(path, roots):
    return path.endswith(".rs") and any(path.startswith(root + "/") for root in roots)


def production(path):
    return not path.startswith("tests/") and not path.endswith("_test.rs")


def observe(report, policy, expected_inputs, expected_counts):
    claim = {"written-methods": "authored-rust-method-call-syntax",
             "written-expression-paths": "authored-rust-expression-path-syntax",
             "written-paths-containing": "authored-rust-path-membership-syntax",
             "written-import-renames": "authored-rust-import-rename-syntax"}[policy["subject"]["kind"]]
    require(report["policy_id"] == "rust:inventory:" + policy["name"] and report["policy"] == policy
            and report["claim"] == claim
            and report["quality"] == "exact", "changed policy or overstated syntax claim")
    require(inputs(report["inputs"]) == expected_inputs, "unbound selected physical inputs")
    require(count_map(report["counts"]) == expected_counts, "wrong exact occurrence map")
    total = sum(expected_counts.values())
    require(report["observed_count"] == total
            and len(report["occurrence_sample"]) == min(total, 16)
            and report["occurrences_omitted"] == max(0, total - 16), "truncation changed quantities")
    locations = set()
    sampled = collections.Counter()
    for row in report["occurrence_sample"]:
        require(row["path"] in expected_inputs and f"{row['path']}:{row['name']}" in expected_counts,
                "sample lies outside counted inputs")
        span_keys = ["line", "column", "end_line", "end_column"]
        require(set(row["span"]) == set(span_keys)
                and all(type(row["span"][key]) is int and row["span"][key] >= 1 for key in span_keys),
                "invalid exact sample span")
        span = tuple(row["span"][key] for key in span_keys)
        require(span[:2] < span[2:], "empty or reversed syntax span")
        key = (row["path"], row["name"], span)
        require(key not in locations, "duplicate sampled physical occurrence")
        locations.add(key)
        sampled[f"{row['path']}:{row['name']}"] += 1
    require(all(count <= expected_counts[key] for key, count in sampled.items()),
            "samples exceed complete occurrence quantities")
    assertion = policy["assertion"]
    if assertion["kind"] == "exact-owners":
        accepted = set(expected_counts) == {row["path"] + ":" + row["name"] for row in assertion["owners"]}
    else:
        accepted = expected_counts == count_map(assertion["counts"])
    require(report["satisfied"] == accepted,
            "incorrect exact-map acceptance")


def bound_inputs(report, live, files, roots):
    frozen = {path: row for (repository, path), row in files.items()
              if repository == live["repository"] and selected(path, roots)}
    baseline = inputs(report["all_source_inputs"])
    require(len(baseline) == len(frozen) == 772 and baseline.keys() == frozen.keys()
            and all(row["sha256"] == frozen[path]["sha256"] for path, row in baseline.items()),
            "source inputs differ from frozen census")
    require(report["registry_sha256"] == files[(live["repository"], "guardrails.toml")]["sha256"],
            "unbound source registry")
    return baseline


def cases(expected, roots):
    result = {"valid": (dict(expected), {}, [])}
    syntax = [{"poll_io": 1}, {"poll_io": 2}, {}, {}, {}, {}, {"poll_io": 2},
              {"poll_io": 2}, {"poll_io": 1, "wake_handle": 1}, *[{"poll_io": 1}] * 10,
              {"poll_io": 2}]
    require(len(syntax) == 20, "invalid typed syntax matrix")
    for index, addition in enumerate(syntax):
        counts = expected | {"src/rc9_adversary.rs:" + name: n for name, n in addition.items()}
        result[f"syntax-{index:02}"] = (counts, {"src/rc9_adversary.rs": True}, [])
    result["original-detector"] = (expected | {"src/reactor/rogue.rs:poll_io": 1},
                                   {"src/reactor/rogue.rs": True}, [])
    for index, key in enumerate(sorted(expected)):
        path, method = key.rsplit(":", 1)
        removed = dict(expected)
        removed[key] -= 1
        if removed[key] == 0:
            del removed[key]
        result[f"remove-{index:02}"] = (removed, {path: True}, [])
        result[f"duplicate-{index:02}"] = (expected | {key: expected[key] + 1}, {path: True}, [])
        other = "pulse_handle" if method == "wake_handle" else "wake_handle"
        key_other = path + ":" + other
        result[f"same-count-substitution-{index:02}"] = (
            removed | {key_other: removed.get(key_other, 0) + 1}, {path: True}, [])
        result[f"same-count-relocation-{index:02}"] = (
            removed | {"src/rc9_moved.rs:" + method: 1}, {path: True, "src/rc9_moved.rs": True}, [])
    for root in roots:
        path = root + "/rc9_fresh.rs"
        result["new-file-" + root] = (expected | ({path + ":poll_io": 1} if production(path) else {}),
                                    {path: True}, [])
    result["test-path-exclusion"] = (dict(expected), {"src/rc9_test.rs": True}, [])
    for name in ["parse-malformed", "parse-expression-fragment"]:
        result[name] = (None, {"src/rc9_adversary.rs": False}, [])
    deleted = "src/reactor/backend.rs"
    result["required-file-deleted"] = ({key: count for key, count in expected.items()
                                       if not key.startswith(deleted + ":")}, {}, [deleted])
    require(len(result) == 76, "incomplete typed mutation matrix")
    return result



def expression_cases(expected, roots):
    require(len(expected) == 8 and set(expected.values()) == {1}, "changed frozen expression inventory")
    result = {"valid": (dict(expected), {}, [])}
    syntax = [{"ConnectionSet::new": 2}, {"ConnectionSet::new": 1}, {"Source::register": 1},
              {}, {}, {}, {}, {"ConnectionSet::new": 1}, {}, {"ConnectionSet::new": 1},
              {"Source::register": 1}, {"ConnectionSet::new": 1, "Source::register": 1},
              {"ConnectionSet::new": 1}, {"Source::register": 1, "DirectSet::new": 1},
              {"DirectSet::new": 1}, {"ConnectionSet::new": 1, "Source::register": 1},
              {"Source::register": 3}, {}, {}, {}, {"ConnectionSet::new": 1, "DirectSet::poll_io": 1},
              {"ConnectionSet::new": 1}]
    for index, counts in enumerate(syntax):
        result[f"syntax-{index:02}"] = (expected | {"src/rc9_adversary.rs:" + k: n for k, n in counts.items()},
                                       {"src/rc9_adversary.rs": True}, [])
    detector = {"src/reactor/rogue.rs:" + suffix: 1 for suffix in
                ["ConnectionSet::new", "DirectSet::new", "DirectSet::poll_io",
                 "DirectSet::turn_component", "DirectSet::wake_handle", "DirectSet::pulse_handle"]}
    result["original-detector"] = (expected | detector, {"src/reactor/rogue.rs": True}, [])
    for index, key in enumerate(sorted(expected)):
        path, suffix = key.split(":", 1)
        removed = {k: n for k, n in expected.items() if k != key}
        result[f"remove-{index:02}"] = (removed, {path: True}, [])
        result[f"duplicate-{index:02}"] = (expected | {key: 2}, {path: True}, [])
        other = "Source::register" if suffix.startswith("ConnectionSet::") else "ConnectionSet::new"
        result[f"same-count-substitution-{index:02}"] = (removed | {path + ":" + other: 1}, {path: True}, [])
        result[f"same-count-relocation-{index:02}"] = (removed | {"src/rc9_moved.rs:" + suffix: 1},
                                                     {path: True, "src/rc9_moved.rs": True}, [])
        for case in ["call-to-reference", "qualified"]:
            result[f"{case}-{index:02}"] = (dict(expected), {path: True}, [])
    for root in roots:
        path = root + "/rc9_fresh.rs"
        result["new-file-" + root] = (expected | ({path + ":ConnectionSet::new": 1} if production(path) else {}),
                                    {path: True}, [])
    result["test-path-exclusion"] = (dict(expected), {"src/rc9_test.rs": True}, [])
    for case in ["parse-malformed", "parse-expression-fragment"]:
        result[case] = (None, {"src/rc9_adversary.rs": False}, [])
    for path in {key.split(":", 1)[0] for key in expected}:
        result["required-file-deleted-" + path] = ({k: n for k, n in expected.items() if not k.startswith(path + ":")}, {}, [path])
    require(len(result) == 83, "incomplete typed expression mutation matrix")
    return result


def verify(assertion, report, assertions, files):
    if assertion["id"] in RENAME_IDS:
        verify_renames = runpy.run_path(ROOT / "scripts/rc9_rename_evidence.py")["verify"]
        return verify_renames(assertion, report, assertions, files)
    if assertion["id"] in OWNER_IDS:
        verify_owners = runpy.run_path(ROOT / "scripts/rc9_owner_evidence.py")["verify"]
        return verify_owners(assertion, report, assertions, files)
    require(assertion["id"] in IDS | EXPRESSION_IDS and report["schema"] == 1, "unsupported assertion/report")
    expression = assertion["id"] in EXPRESSION_IDS
    family = "expression-paths" if expression else "methods"
    policy_id = "rust:inventory:kd-transport-" + family
    require(assertion["replacement"]["policy_ids"] == [policy_id]
            and assertion["replacement"]["expected_diagnostics"] ==
            ["RUST-INVENTORY-002" if "PARSE-" in assertion["id"] else "RUST-INVENTORY-001"],
            "wrong assertion policy or diagnostic")
    live = assertions["KD-TRANSPORT-ASSOCIATED" if expression else "KD-TRANSPORT-METHODS"]
    require(live["repository"] == assertion["repository"] and live["commit"] == assertion["commit"],
            "mixed source snapshots")
    roots = live["selection"]["roots"]["roots"]
    expected = live["required_cardinality"]["exact_map"]
    policy = report["observation"]["policy"]
    subject_key = "suffixes" if expression else "names"
    subjects = live["matching_semantics"]["selected_suffixes" if expression else "methods"]
    require(policy["name"] == "kd-transport-" + family and policy["reason"].strip()
            and policy["include"] == sorted(root + "/**/*.rs" for root in roots)
            and policy["exclude"] == ["**/*_test.rs", "tests/**"] and policy["world"] == "authored"
            and policy["subject"] == {"kind": "written-" + family, subject_key: sorted(subjects)}
            and policy["assertion"]["kind"] == "exact-counts"
            and count_map(policy["assertion"]["counts"]) == expected, "policy differs from frozen assertion")
    policy_bytes = (ROOT / f"docs/rc9/policies/kafka-driver.transport-{family}.fragment.toml").read_bytes()
    authored, = tomllib.loads(policy_bytes.decode())["source"]["rust"]["inventories"]
    for key in ["include", "exclude"]:
        authored[key].sort()
    authored["subject"][subject_key].sort()
    authored["assertion"]["counts"].sort(key=lambda row: (row["path"], row["name"]))
    require(report["policy_sha256"] == hashlib.sha256(policy_bytes).hexdigest() and authored == policy,
            "unbound translated policy bytes")
    baseline = bound_inputs(report, live, files, roots)
    selected_inputs = {path: row for path, row in baseline.items() if production(path)}
    require(len(selected_inputs) == 479 and report["legacy_counts"] == expected, "incomplete baseline")
    observe(report["observation"], policy, selected_inputs, expected)
    detector_id = "KD-TRANSPORT-DETECTOR-ASSOCIATED" if expression else "KD-TRANSPORT-DETECTOR-METHODS"
    detector = assertions[detector_id]["required_cardinality"]["exact_map"]
    required_detector = ({"src/reactor/rogue.rs:" + suffix: 1 for suffix in ["ConnectionSet::new", "DirectSet::new", "DirectSet::poll_io", "DirectSet::turn_component", "DirectSet::wake_handle", "DirectSet::pulse_handle"]}
                         if expression else {"src/reactor/rogue.rs:poll_io": 1})
    require(report["detector_counts"] == detector == required_detector,
            "original detector quantity mismatch")
    expected_cases = expression_cases(expected, roots) if expression else cases(expected, roots)
    rows = report["fixtures"]
    require(len(rows) == len(expected_cases) and {r["case"] for r in rows} == set(expected_cases),
            "missing, extra, or duplicate transport fixture")
    for row in rows:
        counts, parsed, removed = expected_cases[row["case"]]
        changed = inputs(row["changed_inputs"])
        require(row["policy_id"] == policy_id and changed.keys() == parsed.keys()
                and row["legacy_fixture_parses"] == parsed and row["removed_inputs"] == removed,
                "wrong source mutation or original parser outcome")
        require(all(path not in baseline or value != baseline[path] for path, value in changed.items()),
                "fixture did not mutate the selected input")
        expected_inputs = {path: value for path, value in (baseline | changed).items()
                           if path not in removed and production(path)}
        require(row["legacy_counts"] == counts, "wrong original count map")
        accepted = counts == expected
        diagnostic = None if accepted else "RUST-INVENTORY-002" if counts is None else "RUST-INVENTORY-001"
        require(row["native_accepted"] == row["legacy_accepted"] == accepted
                and row["diagnostic"] == diagnostic, "wrong fixture outcome or unrelated diagnostic")
        if counts is None:
            require(row["native"] is None, "partial inventory escaped strict parse rejection")
        else:
            observe(row["native"], policy, expected_inputs, counts)
        if row["case"] == "original-detector":
            require(changed["src/reactor/rogue.rs"]["sha256"] == report["detector_source_sha256"],
                    "detector fixture bytes mismatch")
