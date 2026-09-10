"""Exercise the real frozen translator; fixture proposals never become accepted authority."""

import copy
import json
from pathlib import Path
import runpy
import sys

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parent.parent
PACKAGES = ["bornera", "bornera-core", "bornera-rustls", "kafka-wire", "kafka-wire-core"]


def qualify():
    converter = runpy.run_path(ROOT / "scripts/rc9-provenance-policies")["locked"]
    rows = json.loads((ROOT / "crates/zrail-testkit/tests/fixtures/rc9/assertions.json").read_text())["reviewed_assertions"]
    cases = json.loads((ROOT / "crates/zrail-testkit/tests/fixtures/rc9/lock-prefix-cases.json").read_text())["cases"]
    section, rules = converter(rows)
    assert section == "dependencies.lock_package" and len(rules) == 5
    results, rejected = [], []
    for package in PACKAGES:
        original, = [row for row in rows if row["id"] == "KD-PROVENANCE-" + package + "-VERSION-PREFIX"]
        rule, = [rule for rule in rules if rule["package"] == package]
        version = rule["assertion"]["identities"][0]["version"]

        def changed(row):
            return [row if value["id"] == original["id"] else value for value in rows]

        def result(proposal):
            try:
                output = converter(proposal)
            except ValueError as error:
                return str(error)
            assert output == (section, rules), "changed native authority"
            return None

        for case in cases:
            value = case["template"].replace("$VERSION", version)
            proposal = copy.deepcopy(original)
            proposal["source"]["instance"] = proposal["policy_registry"]["entry"] = value
            error = result(changed(proposal))
            assert (error is None) == (case["name"] == "frozen")
            results.append({"policy_id": "dependency:lock-package:kd-lock-" + package,
                            "case": case["name"], "input_version": value, "error": error})
        alterations = [
            ("source-hash", ["source", "file_sha256"], "0" * 64),
            ("source-instance", ["source", "instance"], "=other"),
            ("selection-path", ["selection", "paths"], ["other.toml"]),
            ("selection-key", ["selection", "keys"], ["dependencies", "other_version"]),
            ("registry-path", ["policy_registry", "path"], "other.toml"),
            ("registry-hash", ["policy_registry", "file_sha256"], "0" * 64),
            ("registry-selector", ["policy_registry", "selector"], ["other"]),
            ("registry-index", ["policy_registry", "index"], 0),
            ("prefix-meaning", ["matching_semantics", "prefix"], "^"),
            ("literal-meaning", ["matching_semantics", "remaining_text_used_as_exact_lock_version"], False),
            ("literal-type", ["matching_semantics", "remaining_text_used_as_exact_lock_version"], 1),
        ]
        proposals = [("missing", [row for row in rows if row["id"] != original["id"]]),
                     ("duplicate", rows + [original])]
        for name, path, value in alterations:
            proposal = copy.deepcopy(original)
            proposal[path[0]][path[1]] = value
            proposals.append((name, changed(proposal)))
        for name, value in [("null", None), ("integer", 0), ("boolean", False),
                            ("array", []), ("table", {}), ("string-array", ["=" + version])]:
            proposal = copy.deepcopy(original)
            proposal["source"]["instance"] = proposal["policy_registry"]["entry"] = value
            proposals.append(("typed-" + name, changed(proposal)))
        for name, proposal in proposals:
            error = result(proposal)
            assert error is not None
            rejected.append({"package": package, "case": name, "error": error})
    assert len(results) == 120 and len(rejected) == 95
    return {"section": section, "rules": rules, "cases": results, "linkage_rejections": rejected}


if __name__ == "__main__":
    print(json.dumps(qualify(), indent=2, sort_keys=True))
