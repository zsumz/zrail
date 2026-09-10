"""Keep typed conversion distinct from changed authority and native schema validation."""

import copy
import gzip
import json
import sys
import subprocess
import tempfile
import tomllib
import unittest

sys.dont_write_bytecode = True

from rc9_registry import ROOT, REGISTRY, FIELDS, Rejected, typed, parse_bytes, frozen, convert
from rc9_registry_cases import CASES, matrix


class RegistryConversion(unittest.TestCase):
    def test_fixtures_reproduce_real_converter_results(self):
        self.assertEqual(json.loads(gzip.decompress((ROOT / CASES).read_bytes())), matrix())

    def test_complete_schema_carrier_preserves_existing_policy(self):
        policy, report = convert(frozen())
        self.assertFalse(report["full_repository_qualified"])
        self.assertEqual(len(report["inputs"]), 9)
        self.assertEqual(len(policy["repository"]["files"]), 119)
        self.assertEqual(len(policy["dependencies"]["lock_package"]), 5)
        self.assertEqual(len(policy["source"]["rust"]["budgets"]["overrides"]), 3)
        self.assertEqual(report["excluded_from_schema_carrier"], [])
        self.assertEqual(report["qualification_pending"], ["traversal-inspection", "registry"])
        traversal, = [row for row in policy["repository"]["files"] if row["name"] == "kd-source-traversal"]
        self.assertEqual(traversal["predicate"], {"kind": "inspect"})
        self.assertEqual(traversal["entry"], "any")

    def test_every_interpreted_field_is_required(self):
        for key in FIELDS:
            value = copy.deepcopy(frozen()); del value[key]
            with self.subTest(key=key), self.assertRaises(Rejected):
                typed(value, FIELDS)

    def test_unknown_legacy_keys_do_not_change_authority(self):
        value = frozen(); value["unknown"] = "ignored"
        self.assertEqual(convert(value), convert(frozen()))
        self.assertEqual(parse_bytes((ROOT / REGISTRY).read_bytes()), frozen())

    def test_changed_interpreted_values_do_not_become_authority(self):
        for section, key, replacement in [("budgets", "production", 0),
                                           ("paths", "rust_roots", []),
                                           ("dependencies", "banned", [])]:
            value = frozen(); value[section][key] = replacement
            with self.assertRaises(Rejected) as caught:
                convert(value)
            self.assertEqual(caught.exception.stage, "authority")

    def test_explicit_size_bound_does_not_claim_unbounded_original_parity(self):
        with self.assertRaises(Rejected) as caught:
            parse_bytes(b"#" * (4 * 1024 * 1024 + 1))
        self.assertEqual(caught.exception.stage, "limit")

    def test_real_cli_emits_only_a_fresh_review_carrier(self):
        with tempfile.TemporaryDirectory(prefix="zrail-registry-cli-") as directory:
            from pathlib import Path
            output = Path(directory) / "review.toml"
            command = [sys.executable, str(ROOT / "scripts/rc9_registry.py"), str(ROOT / REGISTRY), str(output)]
            result = subprocess.run(command, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            source = output.read_bytes()
            self.assertIn(b"Includes corrected inspection; repeated qualification remains pending.", source.splitlines()[1])
            self.assertEqual(tomllib.loads(source.decode()), convert(frozen())[0])
            self.assertNotEqual(subprocess.run(command, capture_output=True).returncode, 0)
            self.assertEqual(output.read_bytes(), source)


if __name__ == "__main__":
    unittest.main()
