"""Immutable execution evidence cannot be relabeled as untouched or full qualification."""

import copy
import gzip
import json
from pathlib import Path
import runpy
import sys
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent
VERIFY = runpy.run_path(ROOT / "scripts/rc9_route_runtime_evidence.py")["verify"]
DATA = ROOT / "crates/zrail-testkit/tests/fixtures/rc9"


class RuntimeEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        ledger = json.loads((DATA / "assertions.json").read_bytes())
        cls.assertions = [row for row in ledger["reviewed_assertions"] if row["id"].startswith("KD-ROUTE-RUNTIME-")]
        with gzip.open(ROOT / "docs/rc9/evidence/route-runtime.json.gz", "rb") as stream:
            cls.report = json.load(stream)
        with gzip.open(DATA / "census.json.gz", "rb") as stream:
            census = json.load(stream)
        cls.files = {(row["repository"], row["path"]): row for row in census["files"]}

    def test_all_fifteen_assertions_bind_actual_execution_and_conditional_native_receipts(self):
        self.assertEqual(len(self.assertions), 15)
        for assertion in self.assertions:
            VERIFY(assertion, self.report, self.files)

    def test_mutated_producers_inputs_outcomes_receipts_and_scope_are_rejected(self):
        mutations = [
            (["schema"], 2), (["implementation_commit"], "0" * 40),
            (["implementation_tree"], "0" * 40), (["snapshot", "commit"], "0" * 40),
            (["full_repository_qualified"], True), (["untouched_snapshot_mirror_verified"], True),
            (["patched_snapshot_mirror_verified"], False), (["zrail_binary_sha256"], "0" * 64),
            (["cargo_lock_sha256"], "0" * 64), (["policy_sha256"], "0" * 64),
            (["profile_sha256"], "0" * 64), (["patch_sha256"], "0" * 64),
            (["original_inputs", "Cargo.toml"], "0" * 64), (["patched_inputs", "Cargo.toml"], "0" * 64),
            (["rustc_version"], "rustc other"), (["original_execution", "outcome", "passed"], 0),
            (["patched_execution", "outcome", "ignored"], 1), (["patched_execution", "outcome", "passed"], True),
            (["original_execution", "listing_sha256"], "0" * 64), (["patched_execution", "arguments"], []),
            (["original_execution", "source_root"], "/unrelated"), (["patched_execution", "binary_path"], "/unrelated"),
            (["helper_execution"], []), (["unresolved"], []),
            (["plan", "analysis", "physical_rust_files"], 1), (["plan", "mirrors", 0, "inputs"], []),
            (["plan", "mirrors", 0, "input_sha256"], "0" * 64), (["plan", "plan_sha256"], "0" * 64),
            (["results", "groups"], []), (["receipt_artifact", "source"], "{}"),
            (["verification", "exit_code"], 1), (["mutations"], []),
            (["mutations", 0, "exit_code"], 0), (["mutations", 1, "diagnostics"], ["LOCK-028"]),
        ]
        self.assertEqual(len(mutations), 34)
        for path, value in mutations:
            with self.subTest(path=path):
                report = copy.deepcopy(self.report)
                parent = report
                for key in path[:-1]:
                    parent = parent[key]
                parent[path[-1]] = value
                with self.assertRaises(ValueError):
                    VERIFY(self.assertions[0], report, self.files)
        for key, value in [("policy_ids", ["unrelated"]), ("expected_diagnostics", ["LOCK-028"]),
                           ("required_source_patch", None), ("qualification_scope", "untouched snapshot"),
                           ("full_snapshot_verified", True)]:
            with self.subTest(assertion=key):
                assertion = copy.deepcopy(self.assertions[0])
                assertion["replacement"][key] = value
                with self.assertRaises(ValueError):
                    VERIFY(assertion, self.report, self.files)


if __name__ == "__main__":
    unittest.main()
