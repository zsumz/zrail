"""Conversion rejects missing prefix authority without changing the frozen fragment."""

import sys
import unittest
import tomllib

sys.dont_write_bytecode = True

from rc9_lock_prefix_conversion import ROOT, qualify


class PrefixConversion(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = qualify()

    def test_frozen_conversion_preserves_every_existing_rule(self):
        rules = tomllib.loads((ROOT / "docs/rc9/policies/kafka-driver.lock-packages.fragment.toml").read_text())["dependencies"]["lock_package"]
        self.assertEqual(self.report["rules"], rules)

    def test_only_the_five_unchanged_prefixed_inputs_translate(self):
        rows = self.report["cases"]
        self.assertEqual(len(rows), 120)
        self.assertEqual(sum(row["error"] is None for row in rows), 5)
        self.assertTrue(all(row["case"] == "frozen" for row in rows if row["error"] is None))

    def test_all_ninety_five_authority_linkage_mutations_are_rejected(self):
        rows = self.report["linkage_rejections"]
        self.assertEqual(len(rows), 95)
        self.assertEqual(len({(row["package"], row["case"]) for row in rows}), 95)
        self.assertTrue(all(row["error"] for row in rows))


if __name__ == "__main__":
    unittest.main()
