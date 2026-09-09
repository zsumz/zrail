"""Deterministic source and native schema fixtures; no output or policy is installed."""

import copy
import gzip
import json
import sys
import tomllib

from rc9_registry import ROOT, REGISTRY, FIELDS, Rejected, parse_bytes, convert, document, frozen

CASES = "crates/zrail-testkit/tests/fixtures/rc9/registry-cases.json.gz"


def parent(value, path):
    for key in path[:-1]:
        value = value[key]
    return value


def source_cases():
    base = frozen()
    rows = []

    def add(name, text, mode="file"):
        source = text.encode() if isinstance(text, str) else text
        if mode != "file":
            original, normalized, converted = "read", None, "read"
        else:
            try:
                normalized = parse_bytes(source)
                original = "pass"
            except Rejected as error:
                original, normalized = error.stage, None
            if normalized is None:
                converted = original
            else:
                try:
                    convert(normalized)
                    converted = "pass"
                except Rejected as error:
                    converted = error.stage
        rows.append(dict(name=name, mode=mode, bytes=list(source), original_stage=original,
                         normalized=normalized, conversion_stage=converted))

    add("frozen", (ROOT / REGISTRY).read_bytes())
    add("comment", "# formatting is not interpreted authority\n" + document(base))
    add("missing-file", b"", "missing")
    add("directory", b"", "directory")
    add("invalid-utf8", b"\xff")
    for name, source in [("empty", ""), ("malformed", "[paths"),
                         ("duplicate", document(base) + "schema = 1\n"),
                         ("trailing", document(base) + "invalid")]:
        add(name, source)
    paths = [[key] for key in FIELDS]
    paths += [[section, key] for section in ["paths", "budgets", "dependencies"] for key in FIELDS[section]]
    paths += [["capabilities", index, key] for index in range(4) for key in ["root", "forbidden"]]
    for path in paths:
        name = ".".join(map(str, path))
        value = copy.deepcopy(base)
        del parent(value, path)[path[-1]]
        add("missing-" + name, document(value))
        for label, replacement in [("bool", False), ("integer", 0), ("string", ""), ("array", []), ("table", {})]:
            value = copy.deepcopy(base)
            if type(parent(value, path)[path[-1]]) is type(replacement):
                continue
            parent(value, path)[path[-1]] = replacement
            add("type-" + name + "-" + label, document(value))
    for path in [["paths"], ["budgets"], ["dependencies"], *[["capabilities", index] for index in range(4)], []]:
        value = copy.deepcopy(base)
        target = value
        for key in path:
            target = target[key]
        target["ignored_by_original"] = {"value": [1, "mixed", True]}
        add("unknown-" + (".".join(map(str, path)) or "root"), document(value))
    for version in [0, 2, 3, -1, 2**32 - 1, 2**32, "1", True, 1.0]:
        value = copy.deepcopy(base); value["schema"] = version
        add("schema-" + type(version).__name__ + "-" + str(version), document(value))
    for ceiling in [0, -1, 2**32, 2**63-1, 2**63, 2**64-1, 2**64, 1.0]:
        value = copy.deepcopy(base); value["budgets"]["production"] = ceiling
        add("budget-" + type(ceiling).__name__ + "-" + str(ceiling), document(value))
    value = copy.deepcopy(base); value["ignored_integer"] = 2**63
    add("unknown-out-of-range-integer", document(value))
    arrays = [["paths", "rust_roots"], ["capabilities"]]
    arrays += [["dependencies", key] for key in ["banned", "core_allowed", "driver_allowed", "probe_allowed", "transport_allowed"]]
    arrays += [["capabilities", index, "forbidden"] for index in range(4)]
    for path in arrays:
        for label in ["empty", "mixed", "duplicate", "reorder"]:
            value = copy.deepcopy(base); owner = parent(value, path); array = owner[path[-1]]
            owner[path[-1]] = [] if label == "empty" else array + [False] if label == "mixed" else array + array[:1] if label == "duplicate" else list(reversed(array))
            add(label + "-" + ".".join(map(str, path)), document(value))
    for key in [key for key in base["dependencies"] if key.endswith(("_version", "_checksum"))]:
        value = copy.deepcopy(base); value["dependencies"][key] = ""
        add("empty-" + key, document(value))
    assert len(rows) == len({row["name"] for row in rows})
    return rows


def native_cases():
    policy, _ = convert(frozen())
    rows = []

    def add(name, source, accepted=False, mode="file"):
        rows.append(dict(name=name, mode=mode, bytes=list(source.encode() if isinstance(source, str) else source), accepted=accepted))

    add("schema-1", document(policy), True)
    for version in [2, 0, 3, -1, "1", True, 1.0]:
        value = copy.deepcopy(policy); value["schema"] = version
        add("schema-" + type(version).__name__ + "-" + str(version), document(value), type(version) is int and version == 2)
    for key in ["schema", "adapters", "repository", "dependencies", "source"]:
        value = copy.deepcopy(policy); del value[key]
        add("missing-" + key, document(value))
    for path in [[], ["repository"], ["dependencies"], ["source", "rust"],
                 ["repository", "files", 0], ["dependencies", "lock_package", 0],
                 ["source", "rust", "budgets"]]:
        value = copy.deepcopy(policy); target = value
        for key in path:
            target = target[key]
        target["ignored_by_original"] = True
        add("unknown-" + (".".join(map(str, path)) or "root"), document(value))
    for name, source in [("empty", ""), ("malformed", "[repository"),
                         ("duplicate", document(policy) + "schema = 1\n"), ("invalid-utf8", b"\xff")]:
        add(name, source)
    add("missing-file", b"", mode="missing")
    add("directory", b"", mode="directory")
    value = copy.deepcopy(policy)
    fragment = tomllib.loads((ROOT / "docs/rc9/policies/kafka-driver.traversal.fragment.toml").read_text())
    row, = [row for row in fragment["repository"]["files"] if row["name"] == "kd-source-traversal"]
    value["repository"]["files"].append(row)
    add("known-traversal-contract-blocker", document(value))
    return rows


def matrix():
    return {"schema": 1, "source": source_cases(), "native": native_cases()}


if __name__ == "__main__":
    result = matrix()
    payload = json.dumps(result, sort_keys=True, separators=(",", ":")).encode()
    if sys.argv[1:] == ["--write-fixture"]:
        # Reproducible generated data, never hand-authored policy or evidence.
        (ROOT / CASES).write_bytes(gzip.compress(payload, mtime=0))
        print(f"generated {len(result['source'])} source and {len(result['native'])} native cases")
    elif sys.argv[1:]:
        raise SystemExit("expected --write-fixture or no arguments")
    else:
        print(payload.decode())
