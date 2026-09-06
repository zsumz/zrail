"""Adversarial checks for exact frozen syntax evidence and ledger linkage."""

import copy
import gzip
import json
from pathlib import Path
import runpy
import unittest


ROOT = Path(__file__).resolve().parent.parent
MODULE = runpy.run_path(ROOT / "scripts/rc9_method_evidence.py")
VERIFY = MODULE["verify"]


class TransportEvidence(unittest.TestCase):
    REPORT_FILE = "transport-methods-parity.json.gz"
    IDS = MODULE["IDS"]
    EXPECTED_IDS = 4
    LIVE_ID = "KD-TRANSPORT-METHODS"
    SUBJECT_KEY = "names"
    SUBJECT_VALUE = "poll_io"

    @classmethod
    def setUpClass(cls):
        data = ROOT / "crates/zrail-testkit/tests/fixtures/rc9"
        ledger = json.loads((data / "assertions.json").read_bytes())
        cls.assertions = {row["id"]: row for row in ledger["reviewed_assertions"]}
        with gzip.open(data / "census.json.gz", "rb") as stream:
            census = json.loads(stream.read(64 * 1024 * 1024 + 1))
        cls.files = {(row["repository"], row["path"]): row for row in census["files"]}
        with gzip.open(ROOT / "docs/rc9/evidence" / cls.REPORT_FILE, "rb") as stream:
            payload = stream.read(64 * 1024 * 1024 + 1)
        if len(payload) > 64 * 1024 * 1024:
            raise ValueError("oversized test report")
        cls.report = json.loads(payload)

    def test_assertions_bind_original_quantities_and_parsers(self):
        self.assertEqual(len(self.IDS), self.EXPECTED_IDS)
        for identity in self.IDS:
            VERIFY(self.assertions[identity], self.report, self.assertions, self.files)

    def test_changed_policy_inputs_quantities_parser_and_outcomes_fail(self):
        assertion = self.assertions[self.LIVE_ID]
        mutations = ["selector", "world", "subject", "quantity", "missing-case", "duplicate-case",
                     "frozen-input", "unrelated-input", "missing-input", "parser-outcome",
                     "wrong-count-map", "legacy-map", "diagnostic", "claim", "omitted-quantity",
                     "sample-outside-scope", "detector-input", "partial-parse", "registry", "policy-bytes"]
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                report = copy.deepcopy(self.report)
                observed = report["observation"]
                fixture = next(row for row in report["fixtures"] if row["case"] == "duplicate-00")
                if mutation == "selector":
                    observed["policy"]["exclude"].append("**/fresh.rs")
                elif mutation == "world":
                    observed["policy"]["world"] = "production"
                elif mutation == "subject":
                    observed["policy"]["subject"][self.SUBJECT_KEY].remove(self.SUBJECT_VALUE)
                elif mutation == "quantity":
                    observed["policy"]["assertion"]["counts"][0]["count"] += 1
                elif mutation == "missing-case":
                    report["fixtures"].remove(fixture)
                elif mutation == "duplicate-case":
                    fixture["case"] = "valid"
                elif mutation == "frozen-input":
                    report["all_source_inputs"][0]["sha256"] = "0" * 64
                elif mutation == "unrelated-input":
                    fixture["native"]["inputs"][0]["sha256"] = "0" * 64
                elif mutation == "missing-input":
                    fixture["native"]["inputs"].pop()
                elif mutation == "parser-outcome":
                    fixture["legacy_fixture_parses"] = {}
                elif mutation == "wrong-count-map":
                    fixture["native"]["counts"][0]["name"] = "another_method"
                elif mutation == "legacy-map":
                    fixture["legacy_counts"] = report["legacy_counts"]
                elif mutation == "diagnostic":
                    fixture["diagnostic"] = "LOCK-028"
                elif mutation == "claim":
                    observed["claim"] = "semantic-receiver-identity"
                elif mutation == "omitted-quantity":
                    observed["occurrences_omitted"] += 1
                elif mutation == "sample-outside-scope":
                    observed["occurrence_sample"][0]["path"] = "elsewhere.rs"
                elif mutation == "detector-input":
                    report["detector_source_sha256"] = "0" * 64
                elif mutation == "partial-parse":
                    next(r for r in report["fixtures"] if r["case"] == "parse-malformed")["native"] = observed
                elif mutation == "registry":
                    report["registry_sha256"] = "0" * 64
                else:
                    report["policy_sha256"] = "0" * 64
                with self.assertRaises(ValueError):
                    VERIFY(assertion, report, self.assertions, self.files)


class ExpressionEvidence(TransportEvidence):
    REPORT_FILE = "transport-expression-paths-parity.json.gz"
    IDS = MODULE["EXPRESSION_IDS"]
    EXPECTED_IDS = 2
    LIVE_ID = "KD-TRANSPORT-ASSOCIATED"
    SUBJECT_KEY = "suffixes"
    SUBJECT_VALUE = "ConnectionSet::new"


if __name__ == "__main__":
    unittest.main()
