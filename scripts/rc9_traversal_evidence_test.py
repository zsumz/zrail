"""Reject incomplete traversal, skipped execution, changed provenance and inflated authority."""

import copy
import gzip
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True

import rc9_traversal_evidence as evidence
from rc9_traversal_common import ROOT, parse


def changed(value, path, replacement):
    value = copy.deepcopy(value)
    parent = value
    for key in path[:-1]:
        parent = parent[key]
    parent[path[-1]] = replacement
    return value


class TraversalEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = parse(gzip.open(ROOT / evidence.ARTIFACT, "rb").read())
        census = parse(gzip.open(ROOT / "crates/zrail-testkit/tests/fixtures/rc9/census.json.gz", "rb").read())
        cls.files = {(row["repository"], row["path"]): row for row in census["files"]}
        ledger = parse((ROOT / "crates/zrail-testkit/tests/fixtures/rc9/assertions.json").read_bytes())
        cls.assertions = [row for row in ledger["reviewed_assertions"] if row["id"].startswith("KD-TRAVERSAL-")]

    def rejects(self, value, changes, validate):
        for path, replacement in changes:
            with self.subTest(path=path, replacement=replacement), self.assertRaises(ValueError):
                validate(changed(value, path, replacement))

    def test_exact_reports_and_three_assertions(self):
        self.assertEqual(len(self.assertions), 3)
        for assertion in self.assertions:
            evidence.verify(assertion, self.report, self.files)

    def test_immutable_report_provenance_and_scope(self):
        changes = [([key], replacement) for key, replacement in {
            "schema": True, "full_repository_qualified": True, "implementation_commit": "0" * 40,
            "implementation_tree": "0" * 40, "snapshot": {}, "test_binary_sha256": "0" * 64,
            "rustc_version": "other", "cargo_lock_sha256": "0" * 64, "binary_path": "/other",
            "listing_sha256": "0" * 64, "total_tests": 1, "build_command": [], "producer_inputs": {},
            "policy_path": "other", "policy_sha256": "0" * 64, "native_payload_sha256": "0" * 64,
            "limitations": [], "physical": [], "injected": [], "executions": [], "frozen_inputs": {},
        }.items()]
        self.rejects(self.report, changes, lambda report: evidence.verify_report(report, self.files))

    def test_frozen_structure_independent_of_payload_digest(self):
        changes = [(["roots"], []), (["guardrails_sha256"], "0" * 64), (["support_sha256"], "0" * 64),
                   (["frozen_inputs"], {}), (["frozen_paths"], self.report["native_frozen_paths"]),
                   (["native_frozen_paths"], self.report["frozen_paths"]), (["frozen_paths", 0], "outside.rs"),
                   (["frozen_paths"], self.report["frozen_paths"] * 2), (["frozen_observations"], []),
                   (["frozen_observations", 1, "entries"], []), (["frozen_observations", 0, "satisfied"], 1),
                   (["frozen_observations", 1, "analysis"], "partial"), (["frozen_observations", 1, "entries", 0, "sha256"], "0" * 64)]
        self.rejects(self.report, changes, lambda report: evidence.validate_frozen(report, self.files))

    def test_physical_boundaries_independent_of_payload_digest(self):
        rows = self.report["physical"]
        for altered in [rows[:-1], rows[::-1], rows * 2]:
            with self.assertRaises(ValueError):
                evidence.validate_physical(altered)
        changes = [([0, "legacy_error"], None), ([0, "legacy_error"], "read directory entry: unrelated"),
                   ([0, "native_paths"], []), ([0, "diagnostics"], []),
                   ([0, "observations", 0, "satisfied"], True), ([0, "observations", 1, "entries"], []),
                   ([0, "observations", 1, "policy_id"], "repository:file:kd-source-roots"),
                   ([0, "observations", 0, "entries", 0, "sha256"], "0" * 64)]
        cases = {row["case"]: index for index, row in enumerate(rows)}
        changes += [([cases["src:component-order"], "legacy_paths"], ["src/a.rs", "src/a/nested.rs"]),
                    ([cases["src:component-order"], "native_paths"], ["src/a/nested.rs", "src/a.rs"]),
                    ([cases["src:directory-link"], "native_error"], "REP-FILE-006: unrelated"),
                    ([cases["src:cycle"], "legacy_error"], "skipped"),
                    ([cases["src:unread-empty"], "legacy_error"], "read directory $ROOT/src: synthetic"),
                    ([cases["outside:unread-directory"], "native_error"], None)]
        self.rejects(rows, changes, evidence.validate_physical)

    def test_injected_failures_are_not_relabelled_os_observations(self):
        rows = self.report["injected"]
        for altered in [rows[:-1], rows[::-1], rows * 2]:
            with self.assertRaises(ValueError):
                evidence.validate_injected(altered)
        self.rejects(rows, [([0, "native_operations"], True), ([0, "native_operations"], 0),
                           ([0, "legacy_error"], "read entry: observed OS error"),
                           ([0, "native_error"], "partial success"), ([1, "legacy_error"], rows[0]["legacy_error"]),
                           ([3, "case"], "src:false:PermissionDenied:DirEntry::file_type")], evidence.validate_injected)

    def test_exact_executions_reject_skips_and_fabricated_counts(self):
        self.rejects(self.report, [(["total_tests"], True), (["executions"], []),
            (["executions"], self.report["executions"][::-1]), (["executions", 0, "test"], "other"),
            (["executions", 0, "cases"], True), (["executions", 0, "outcome", "passed"], True),
            (["executions", 0, "outcome", "passed"], 0), (["executions", 0, "outcome", "ignored"], 1),
            (["executions", 0, "outcome", "filtered"], 629), (["executions", 0, "outcome", "failed"], 1),
            (["executions", 3, "arguments"], self.report["executions"][3]["arguments"][:-1]),
            (["executions", 5, "scope"], "observed-os-errors"), (["executions", 6, "cases"], 0)], evidence.validate_executions)

    def test_raw_execution_logs_remain_bound(self):
        self.rejects(self.report, [(["listing_sha256"], "0" * 64), (["native_payload_sha256"], "0" * 64),
                                  (["executions", 0, "outcome", "passed"], 0)], evidence.validate_logs)

    def test_assertion_linkage_cannot_expand_scope(self):
        changes = [(["id"], "KD-RETIRED-TREE-tls"), (["commit"], "0" * 40), (["disposition"], "approved retirement"),
                   (["source", "instance"], "other"), (["source", "file_sha256"], "0" * 64),
                   (["selection", "roots"], []), (["selection", "extension"], ".txt"),
                   (["matching_semantics", "operation"], "another operation"),
                   (["replacement", "policy_ids"], ["repository:file:kd-source-traversal"]),
                   (["replacement", "expected_diagnostics"], []), (["replacement", "implemented"], False),
                   (["replacement", "verified"], False), (["replacement", "full_snapshot_verified"], True),
                   (["replacement", "qualification_scope"], "exact-order parity"), (["replacement", "fixtures"], []),
                   (["replacement", "traversal_evidence"], {}), (["blockers"], [{"id": "open"}])]
        with patch.object(evidence, "verify_report"):
            self.rejects(self.assertions[0], changes, lambda assertion: evidence.verify(assertion, self.report, self.files))


if __name__ == "__main__":
    unittest.main()
