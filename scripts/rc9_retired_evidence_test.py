"""Tampered rc9 slice artifacts must not become replacement authority."""

import copy
import gzip
import json
from pathlib import Path
import runpy
import unittest

ROOT = Path(__file__).resolve().parent.parent
VERIFY = runpy.run_path(ROOT / "scripts/rc9_retired_evidence.py")["verify_report"]


class RetiredEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with gzip.open(ROOT / "docs/rc9/evidence/retired-parity.json.gz", "rb") as stream:
            cls.report = json.load(stream)
        with gzip.open(ROOT / "crates/zrail-testkit/tests/fixtures/rc9/census.json.gz", "rb") as stream:
            cls.files = {(row["repository"], row["path"]): row for row in json.load(stream)["files"]}

    def test_all_original_and_native_outcomes_are_bound(self):
        VERIFY(self.report, self.files)

    def test_changed_policy_inputs_identity_diagnostics_and_quantities_fail(self):
        changes = [
            (["schema"], True), (["full_repository_qualified"], True),
            (["implementation_commit"], "0" * 40), (["implementation_tree"], "0" * 40),
            (["snapshot", "commit"], "0" * 40), (["rustc_version"], "other"),
            (["test_binary_sha256"], "0" * 64), (["fixture_origins_sha256"], "0" * 64),
            (["policy_sha256"], "0" * 64), (["cargo_lock_sha256"], "0" * 64),
            (["input_hashes"], {}), (["baseline"], {}),
            (["fixtures", 1, "case"], "valid"), (["fixtures", 1, "inputs"], {}),
            (["fixtures", 1, "native_accepted"], True), (["fixtures", 1, "legacy_accepted"], True),
            (["fixtures", 1, "diagnostics"], [["rust:inventory:kd-retired-modules", "LOCK-028"]]),
            (["fixtures", 1, "inventories", 0, "counts"], []),
            (["fixtures", 1, "inventories", 0, "observed_count"], 0),
            (["fixtures", 1, "inventories", 0, "observed_count"], True),
            (["fixtures", 1, "inventories", 0, "inputs"], []),
            (["fixtures", 1, "inventories", 0, "claim"], "semantic-identity"),
            (["fixtures", 1, "inventories", 0, "quality"], "unresolved"),
            (["fixtures", 1, "inventories", 0, "satisfied"], True),
            (["fixtures", 1, "inventories", 0, "policy", "include"], ["unrelated.rs"]),
            (["fixtures", 1, "inventories", 0, "occurrence_sample"], []),
            (["fixtures", 1, "files", 0, "entries", 0, "sha256"], "0" * 64),
            (["fixtures", 1, "files", 0, "entries", 0, "literal_count"], 1),
            (["fixtures", 1, "files", 0, "entries", 0, "literal_count"], False),
            (["fixtures", 1, "files", 0, "claim"], "semantic-identity"),
            (["fixtures", 1, "files", 0, "policy", "predicate", "mode"], "contains"),
            (["fixtures", 1, "files", 0, "entries"], []),
            (["fixtures", 1, "files"], []), (["fixtures", 1, "inventories"], []),
        ]
        self.assertEqual(len(changes), 34)
        for path, value in changes:
            with self.subTest(path=path):
                report = copy.deepcopy(self.report)
                parent = report
                for key in path[:-1]:
                    parent = parent[key]
                parent[path[-1]] = value
                with self.assertRaises(ValueError):
                    VERIFY(report, self.files)
        for mutation in ["missing", "duplicate", "reordered"]:
            with self.subTest(mutation=mutation):
                report = copy.deepcopy(self.report)
                if mutation == "missing":
                    report["fixtures"].pop()
                elif mutation == "duplicate":
                    report["fixtures"].append(report["fixtures"][-1])
                else:
                    report["fixtures"].reverse()
                with self.assertRaises(ValueError):
                    VERIFY(report, self.files)


if __name__ == "__main__":
    unittest.main()
