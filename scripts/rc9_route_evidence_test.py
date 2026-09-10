"""Adversarial route evidence checks do not execute the frozen consumer."""

import copy
import gzip
import json
from pathlib import Path
import runpy
import unittest


ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / "crates/zrail-testkit/tests/fixtures/rc9"
VERIFY = runpy.run_path(ROOT / "scripts/rc9_route_evidence.py")["verify"]
FACADE = "src/reactor/direct_plaintext/cluster_runtime.rs"


class RouteEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        ledger = json.loads((DATA / "assertions.json").read_bytes())
        cls.assertions = [row for row in ledger["reviewed_assertions"]
                          if row["id"].startswith("KD-ROUTE-") and "-RUNTIME-" not in row["id"]]
        with gzip.open(ROOT / "docs/rc9/evidence/route-source-typed-parity.json.gz", "rb") as stream:
            payload = stream.read(64 * 1024 * 1024 + 1)
        if len(payload) > 64 * 1024 * 1024:
            raise ValueError("oversized route report")
        cls.report = json.loads(payload)
        with gzip.open(DATA / "census.json.gz", "rb") as stream:
            census = json.load(stream)
        cls.files = {(row["repository"], row["path"]): row for row in census["files"]}

    def test_all_sixteen_assertions_bind_complete_native_and_original_evidence(self):
        self.assertEqual(len(self.assertions), 16)
        for assertion in self.assertions:
            VERIFY(assertion, self.report, self.files)

    def test_changed_inputs_predicates_quantities_outcomes_and_compiler_sites_fail(self):
        missing = next(i for i, row in enumerate(self.report["fixtures"])
                       if row["case"] == "input-missing:" + FACADE)
        invalid = next(i for i, row in enumerate(self.report["fixtures"])
                       if row["case"] == "input-utf8:" + FACADE)
        decoy = next(i for i, row in enumerate(self.report["fixtures"])
                     if row["case"] == "owner-wrong-file")
        mutations = [
            (["schema"], 2), (["snapshot", "commit"], "0" * 40),
            (["full_repository_qualified"], True), (["rustc_version"], "rustc other"),
            (["cargo_lock_sha256"], "0" * 64), (["source_test_sha256"], "0" * 64),
            (["harness_sha256"], "0" * 64), (["fixture_origins_sha256"], "0" * 64),
            (["policy_sha256"], "0" * 64), (["input_hashes", FACADE, "sha256"], "0" * 64),
            (["fixtures", 1, "case"], "valid"), (["fixtures", 1, "legacy_accepted"], True),
            (["fixtures", 1, "native_accepted"], True),
            (["fixtures", 1, "findings"], [["repository:file:kd-route-forbid-ConnectionSet", "LOCK-028"]]),
            (["fixtures", decoy, "extra_inputs"], {}), (["fixtures", 1, "inputs"], {}),
            (["fixtures", 0, "observations", 0, "claim"], "semantic-identity"),
            (["fixtures", 0, "observations", 0, "analysis"], "unresolved"),
            (["fixtures", 0, "observations", 0, "policy", "include"], ["unrelated.rs"]),
            (["fixtures", 0, "observations", 0, "policy", "predicate", "mode"], "contains"),
            (["fixtures", 0, "observations", 0, "entries", 0, "literal_count"], 1),
            (["fixtures", 0, "observations", 0, "entries", 0, "literal_count"], False),
            (["fixtures", 0, "observations", 0, "entries", 0, "literal_offsets"], [99]),
            (["fixtures", 0, "observations", 0, "entries", 0, "omitted_literal_offsets"], 1),
            (["fixtures", 0, "observations", 0, "entries"], []),
            (["fixtures", 0, "observations"], []), (["fixtures", invalid, "error"], "LOCK-028"),
            (["fixtures", missing, "compiler", "exit_code"], 0),
            (["fixtures", invalid, "compiler", "errors", 0, "spans", 0, "file_name"], "unrelated.rs"),
            (["fixtures", missing, "compiler", "errors", 0, "spans", 0, "byte_start"], 0),
            (["fixtures", missing, "compiler", "executable_sha256"], "0" * 64),
            (["fixtures", missing, "compiler", "arguments"], ["--version"]),
            (["fixtures", missing, "compiler", "execution"], {"passed": 1}),
            (["fixtures", 0, "compiler", "execution", "ignored"], 1),
            (["fixtures", 0, "compiler", "execution", "filtered"], 1),
            (["fixtures", 0, "compiler", "execution", "passed"], 2),
            (["fixtures", 0, "compiler", "execution", "passed"], True),
            (["fixtures", 0, "compiler", "execution", "test"], "unrelated"),
            (["fixtures", 0, "compiler", "execution", "list_sha256"], "0" * 64),
        ]
        self.assertEqual(len(mutations), 39)
        for path, value in mutations:
            with self.subTest(path=path, value=value):
                report = copy.deepcopy(self.report)
                parent = report
                for key in path[:-1]:
                    parent = parent[key]
                parent[path[-1]] = value
                with self.assertRaises(ValueError):
                    VERIFY(self.assertions[0], report, self.files)
        for mutation in ["remove-case", "duplicate-case", "remove-compiler"]:
            with self.subTest(mutation=mutation):
                report = copy.deepcopy(self.report)
                if mutation == "remove-case":
                    report["fixtures"].pop()
                elif mutation == "duplicate-case":
                    report["fixtures"].append(report["fixtures"][-1])
                else:
                    del report["fixtures"][missing]["compiler"]
                with self.assertRaises(ValueError):
                    VERIFY(self.assertions[0], report, self.files)


if __name__ == "__main__":
    unittest.main()
