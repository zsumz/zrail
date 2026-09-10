"""Reject changed source, partial comparisons, skipped execution and live-policy scope inflation."""

import copy
import gzip
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True

import rc9_dependency_detector_evidence as evidence
from rc9_dependency_detector_common import ROOT, parse


def changed(value, path, replacement):
    value = copy.deepcopy(value)
    parent = value
    for key in path[:-1]:
        parent = parent[key]
    parent[path[-1]] = replacement
    return value


class DependencyDetectorEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = parse(gzip.open(ROOT / evidence.ARTIFACT, "rb").read())
        census = parse(gzip.open(ROOT / "crates/zrail-testkit/tests/fixtures/rc9/census.json.gz", "rb").read())
        cls.files = {(row["repository"], row["path"]): row for row in census["files"]}
        ledger = parse((ROOT / "crates/zrail-testkit/tests/fixtures/rc9/assertions.json").read_bytes())
        cls.assertions = [row for row in ledger["reviewed_assertions"] if row["id"] in evidence.IDENTITIES]

    def rejects(self, value, changes, validate):
        for path, replacement in changes:
            with self.subTest(path=path, replacement=replacement), self.assertRaises(ValueError):
                validate(changed(value, path, replacement))

    def test_exact_reports_and_both_assertions(self):
        self.assertEqual(len(self.assertions), 2)
        for assertion in self.assertions:
            evidence.verify(assertion, self.report, self.files)

    def test_immutable_producer_source_and_scope(self):
        changes = [([key], value) for key, value in {
            "schema": True, "full_repository_qualified": True, "implementation_commit": "0" * 40,
            "implementation_tree": "0" * 40, "snapshot": {}, "test_binary_sha256": "0" * 64,
            "rustc_version": "other", "cargo_lock_sha256": "0" * 64, "binary_path": "/other",
            "listing_sha256": "0" * 64, "total_tests": 1, "build_command": [], "producer_inputs": {},
            "policy_path": "Cargo.lock", "policy_sha256": "0" * 64, "native_payload_sha256": "0" * 64,
            "limitations": [], "source_sha256": "0" * 64, "original_detector_sha256": "0" * 64,
            "original_helper_sha256": "0" * 64, "cases_sha256": "0" * 64, "origins_sha256": "0" * 64,
            "original_assertions_executed": 0, "fixtures": [], "executions": [],
        }.items()]
        self.rejects(self.report, changes, lambda report: evidence.verify_report(report, self.files))

    def test_fixture_structure_independent_of_payload_digest(self):
        changes = [(["fixtures"], self.report["fixtures"][:-1]), (["fixtures"], self.report["fixtures"][::-1]),
                   (["fixtures"], self.report["fixtures"] * 2), (["fixtures", 0, "case"], "other"),
                   (["fixtures", 0, "source_sha256"], "0" * 64), (["fixtures", 0, "original_names"], ["bytes"]),
                   (["fixtures", 0, "original_names"], ["bytes", "bytes", "tokio-util-extra"]),
                   (["fixtures", 0, "diagnostics"], [["other", "LOCK-008"]]),
                   (["fixtures", 1, "diagnostics"], []), (["fixtures", 0, "observations"], [])]
        base = ["fixtures", 0, "observations", 0]
        changes += [(base + [key], value) for key, value in {
            "policy_id": "repository:file:kd-dep-detector-present-name", "policy": {}, "analysis": "partial",
            "claim": "cargo-lock-graph", "satisfied": 1, "checkout_path": "/other", "reference": {}, "entries": [],
        }.items()]
        entry = base + ["entries", 0]
        changes += [(entry + [key], value) for key, value in {
            "path": "Cargo.lock", "kind": "directory", "resolved_path": "other", "sha256": "0" * 64,
            "bytes": 0, "satisfied": False, "literal_count": False, "literal_offsets": [0], "omitted_literal_offsets": True,
        }.items()]
        changes += [(["fixtures", 11, "observations", 1, "entries", 0, "literal_count"], 1),
                    (["fixtures", 12, "observations", 1, "entries", 0, "omitted_literal_offsets"], 0),
                    (["fixtures", 12, "observations", 1, "entries", 0, "literal_offsets"], list(range(20))),
                    (["fixtures", 0, "observations", 1, "entries", 0, "literal_offsets"], [True])]
        self.rejects(self.report, changes, evidence.validate_fixtures)

    def test_execution_requires_exact_tests_not_counts_alone(self):
        self.rejects(self.report, [(["total_tests"], True), (["executions"], []),
            (["executions"], self.report["executions"][::-1]), (["executions", 0, "test"], "other"),
            (["executions", 0, "cases"], True), (["executions", 0, "outcome", "passed"], True),
            (["executions", 0, "outcome", "passed"], 0), (["executions", 0, "outcome", "ignored"], 1),
            (["executions", 0, "outcome", "filtered"], 634), (["executions", 0, "outcome", "failed"], 1),
            (["executions", 3, "arguments"], self.report["executions"][3]["arguments"][:-1]),
            (["executions", 2, "scope"], "live-cargo-lock"), (["executions", 3, "cases"], 0)], evidence.validate_executions)

    def test_raw_logs_bind_repeats_and_exact_execution(self):
        self.rejects(self.report, [(["listing_sha256"], "0" * 64), (["native_payload_sha256"], "0" * 64),
                                  (["executions", 0, "outcome", "passed"], 0)], evidence.validate_logs)

    def test_linkage_cannot_turn_an_inline_detector_into_live_authority(self):
        changes = [(["id"], "KD-TRAVERSAL-ENTRY"), (["commit"], "0" * 40), (["disposition"], "approved retirement"),
                   (["source", "instance"], "other"), (["source", "file_sha256"], "0" * 64),
                   (["source", "function"], ["dependency_graph_contains_no_async_runtime"]),
                   (["selection", "paths"], ["Cargo.lock"]), (["matching_semantics", "name"], "other"),
                   (["matching_semantics", "present"], True), (["required_cardinality", "present"], 0),
                   (["matching_semantics", "fixture"], "name = \"bytes\""),
                   (["replacement", "policy_ids"], []), (["replacement", "expected_diagnostics"], ["DEP-LOCK-001"]),
                   (["replacement", "implemented"], False), (["replacement", "verified"], False),
                   (["replacement", "full_snapshot_verified"], True), (["replacement", "qualification_scope"], "live Cargo graph"),
                   (["replacement", "fixtures"], []), (["replacement", "detector_evidence"], {}), (["blockers"], [{"id": "open"}])]
        with patch.object(evidence, "verify_report"):
            self.rejects(self.assertions[0], changes, lambda assertion: evidence.verify(assertion, self.report, self.files))


if __name__ == "__main__":
    unittest.main()
