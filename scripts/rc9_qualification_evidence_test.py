"""Adversarial checks for bound audit artifacts; no consumer programs execute."""

import copy
import gzip
import json
from pathlib import Path
import runpy
import unittest


ROOT = Path(__file__).resolve().parent.parent
VERIFY = runpy.run_path(ROOT / "scripts/rc9_qualification_evidence.py")["verify"]


class QualificationEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        ledger = json.loads((ROOT / "crates/zrail-testkit/tests/fixtures/rc9/assertions.json").read_bytes())
        cls.assertions = [row for row in ledger["reviewed_assertions"] if row["id"].startswith("KD-QUAL-")]
        with gzip.open(ROOT / "docs/rc9/evidence/qualification-parity.json.gz", "rb") as stream:
            payload = stream.read(64 * 1024 * 1024 + 1)
        if len(payload) > 64 * 1024 * 1024:
            raise ValueError("oversized test report")
        cls.report = json.loads(payload)

    def test_all_frozen_assertion_bindings(self):
        self.assertEqual(len(self.assertions), 125)
        for assertion in self.assertions:
            VERIFY(assertion, self.report)

    def test_changed_policy_fixtures_inputs_and_claims_fail(self):
        assertion = next(row for row in self.assertions if row["id"] == "KD-QUAL-097")
        policy_id, = assertion["replacement"]["policy_ids"]
        for mutation in ["predicate", "selector", "missing-case", "duplicate-case",
                         "unrelated-input", "diagnostic", "claim"]:
            with self.subTest(mutation=mutation):
                report = copy.deepcopy(self.report)
                observed = next(row for row in report["observations"] if row["policy_id"] == policy_id)
                fixture = next(row for row in report["fixtures"]
                               if row["policy_id"] == policy_id and not row["native_accepted"])
                if mutation == "predicate":
                    observed["policy"]["predicate"]["contains"] = "weakened"
                elif mutation == "selector":
                    observed["policy"]["include"] = ["elsewhere"]
                elif mutation == "missing-case":
                    report["fixtures"].remove(fixture)
                elif mutation == "duplicate-case":
                    fixture["case"] = "valid"
                elif mutation == "unrelated-input":
                    fixture["inputs"]["package.json"] = "0" * 64
                elif mutation == "diagnostic":
                    fixture["diagnostic"] = "LOCK-028"
                else:
                    observed["claim"] = "execution-evidence"
                with self.assertRaises(ValueError):
                    VERIFY(assertion, report)


if __name__ == "__main__":
    unittest.main()
