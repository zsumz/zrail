"""Reject changed physical qualification, injected outcomes and assertion authority."""

import copy
import gzip
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True

import rc9_retired_boundary_evidence as evidence
from rc9_retired_boundary_common import ROOT, canonical, parse, sha


def changed(value, path, replacement):
    value = copy.deepcopy(value)
    parent = value
    for key in path[:-1]:
        parent = parent[key]
    parent[path[-1]] = replacement
    return value


class RetiredBoundaryEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with gzip.open(ROOT / evidence.ARTIFACT, "rb") as stream:
            cls.report = parse(stream.read())
        with gzip.open(ROOT / "crates/zrail-testkit/tests/fixtures/rc9/census.json.gz", "rb") as stream:
            cls.files = {(row["repository"], row["path"]): row for row in parse(stream.read())["files"]}
        ledger = parse((ROOT / "crates/zrail-testkit/tests/fixtures/rc9/assertions.json").read_bytes())
        cls.assertions = [row for row in ledger["reviewed_assertions"] if row["id"].startswith("KD-RETIRED-TREE-")]

    def test_exact_reports_and_all_seven_assertions_are_bound(self):
        self.assertEqual(len(self.assertions), 7)
        for assertion in self.assertions:
            evidence.verify(assertion, self.report, self.files)

    def test_immutable_report_mutations_fail(self):
        changes = [
            (["schema"], True), (["full_repository_qualified"], True),
            (["implementation_commit"], "0" * 40), (["implementation_tree"], "0" * 40),
            (["snapshot", "commit"], "0" * 40), (["test_binary_sha256"], "0" * 64),
            (["rustc_version"], "other"), (["cargo_lock_sha256"], "0" * 64),
            (["binary_path"], "/unrelated/binary"), (["listing_sha256"], "0" * 64),
            (["total_tests"], 1), (["build_command"], []), (["producer_inputs"], {}),
            (["policy_path"], "old-file-only-policy"), (["policy_sha256"], "0" * 64),
            (["ordinary_sha256"], "0" * 64), (["limitations"], []),
            (["executions"], []), (["ordinary", "input_hashes"], {}),
        ]
        self.assertEqual(len(changes), 19)
        for path, value in changes:
            with self.subTest(path=path), self.assertRaises(ValueError):
                evidence.verify_report(changed(self.report, path, value), self.files)

    def test_execution_structure_rejects_skips_wrong_suites_and_fabricated_counts(self):
        changes = [
            (["total_tests"], True), (["total_tests"], 621), (["executions"], []),
            (["executions", 0, "test"], "unrelated"), (["executions", 1, "cases"], 143),
            (["executions", 1, "cases"], True), (["executions", 6, "scope"], "exact-legacy-cycle-parity"),
            (["executions", 8, "ignored_switch"], False), (["executions", 8, "ignored_switch"], 1),
            (["executions", 10, "scope"], "observed-os-error"),
            (["executions", 11, "cases"], 27), (["executions", 12, "ignored_switch"], False),
            (["executions", 0, "outcome", "passed"], 0), (["executions", 0, "outcome", "passed"], True),
            (["executions", 0, "outcome", "ignored"], 1), (["executions", 0, "outcome", "failed"], 1),
            (["executions", 0, "outcome", "filtered"], 622),
        ]
        self.assertEqual(len(changes), 17)
        for path, value in changes:
            with self.subTest(path=path), self.assertRaises(ValueError):
                evidence.validate_executions(changed(self.report, path, value))
        for rows in [self.report["executions"][:-1], self.report["executions"] * 2,
                     self.report["executions"][::-1]]:
            with self.assertRaises(ValueError):
                evidence.validate_executions({**self.report, "executions": rows})

    def test_ordinary_structure_is_checked_even_with_a_recomputed_digest(self):
        changes = [
            (["schema"], True), (["full_repository_qualified"], True), (["input_hashes"], {}),
            (["baseline"], {}), (["fixtures"], []), (["fixtures", 1, "case"], "valid"),
            (["fixtures", 1, "native_accepted"], True), (["fixtures", 1, "legacy_accepted"], True),
            (["fixtures", 1, "diagnostics"], [["unrelated", "LOCK-028"]]),
            (["fixtures", 1, "inventories", 0, "counts"], []),
            (["fixtures", 1, "files"], []), (["fixtures", 1, "inputs"], {}),
        ]
        self.assertEqual(len(changes), 12)
        for path, value in changes:
            report = changed(self.report["ordinary"], path, value)
            outer = {**self.report, "ordinary_sha256": sha(canonical(report))}
            with self.subTest(path=path), self.assertRaises(ValueError):
                evidence.validate_ordinary(report, outer, self.files)

    def test_assertion_linkage_cannot_expand_or_relabel_the_evidence(self):
        changes = [
            (["id"], "KD-TRAVERSAL-ENTRY"), (["commit"], "0" * 40),
            (["source", "instance"], "tls"), (["source", "file_sha256"], "0" * 64),
            (["selection", "extension"], "txt"), (["required_cardinality", "exact"], 1),
            (["required_cardinality", "exact"], False),
            (["replacement", "policy_ids"], ["repository:file:unrelated"]),
            (["replacement", "expected_diagnostics"], ["REP-FILE-001"]),
            (["replacement", "implemented"], False), (["replacement", "verified"], False),
            (["replacement", "full_snapshot_verified"], True),
            (["replacement", "qualification_scope"], "exact error-path parity"),
            (["replacement", "fixtures"], []), (["replacement", "retired_boundary_evidence"], {}),
            (["replacement", "retired_parity_evidence"], {}), (["blockers"], [{"id": "open"}]),
        ]
        self.assertEqual(len(changes), 17)
        with patch.object(evidence, "verify_report"):
            for path, value in changes:
                with self.subTest(path=path), self.assertRaises(ValueError):
                    evidence.verify(changed(self.assertions[0], path, value), self.report, self.files)


if __name__ == "__main__":
    unittest.main()
