"""Reject incomplete array proofs, conflated failure stages and manufactured execution."""

import copy
import gzip
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True

import rc9_lock_array_evidence as evidence
from rc9_lock_array_common import ROOT, parse


def changed(value, path, replacement):
    value = copy.deepcopy(value)
    parent = value
    for key in path[:-1]:
        parent = parent[key]
    parent[path[-1]] = replacement
    return value


class LockArrayEvidence(unittest.TestCase):
    rejected = 0

    @classmethod
    def setUpClass(cls):
        cls.report = parse(gzip.open(ROOT / evidence.ARTIFACT, "rb").read())
        census = parse(gzip.open(ROOT / "crates/zrail-testkit/tests/fixtures/rc9/census.json.gz", "rb").read())
        cls.files = {(row["repository"], row["path"]): row for row in census["files"]}
        ledger = parse((ROOT / "crates/zrail-testkit/tests/fixtures/rc9/assertions.json").read_bytes())
        cls.assertion, = [row for row in ledger["reviewed_assertions"] if row["id"] == "KD-PROVENANCE-LOCK-PACKAGE-ARRAY"]

    @classmethod
    def tearDownClass(cls):
        print(f"{cls.rejected} adversarial mutations rejected")

    def rejects(self, value, changes, validate):
        for path, replacement in changes:
            with self.subTest(path=path, replacement=replacement), self.assertRaises(ValueError):
                validate(changed(value, path, replacement))
            type(self).rejected += 1

    def test_bound_report_and_assertion(self):
        evidence.verify(self.assertion, self.report, self.files)

    def test_immutable_producer_source_and_scope(self):
        changes = [([key], value) for key, value in {
            "schema": True, "full_repository_qualified": True, "implementation_commit": "0" * 40,
            "implementation_tree": "0" * 40, "snapshot": {}, "test_binary_sha256": "0" * 64,
            "rustc_version": "other", "cargo_lock_sha256": "0" * 64, "binary_path": "/other",
            "listing_sha256": "0" * 64, "total_tests": 1, "build_command": [], "producer_inputs": {},
            "policy_path": "other", "policy_sha256": "0" * 64, "native_payload_sha256": "0" * 64,
            "limitations": [], "source_sha256": "0" * 64, "origins_sha256": "0" * 64,
            "cases_sha256": "0" * 64, "original_origins_sha256": "0" * 64,
            "original_helper_invocations": 0, "workspace_packages": {}, "fixtures": [],
            "empty_counts": [], "observations": [], "inputs": {}, "executions": [],
        }.items()]
        self.rejects(self.report, changes, lambda report: evidence.verify_report(report, self.files))

    def test_array_stages_independent_of_payload_digest(self):
        changes = [(["schema"], True), (["original_helper_invocations"], True),
                   (["full_repository_qualified"], True), (["inputs"], {}), (["workspace_packages"], {}),
                   (["source_sha256"], "0" * 64), (["original_origins_sha256"], "0" * 64),
                   (["cases_sha256"], "0" * 64), (["fixtures"], self.report["fixtures"][:-1]),
                   (["fixtures"], self.report["fixtures"][::-1]),
                   (["fixtures"], self.report["fixtures"] * 2)]
        changes += [(["fixtures", 0, key], value) for key, value in {
            "case": "string", "source_sha256": "0" * 64, "array_count": 0,
            "original_error": "unrelated failure", "native_count": 0,
            "native_error": "generic nonzero exit", "stage": "later-package-validation",
        }.items()]
        changes += [(["fixtures", 10, "array_count"], False), (["fixtures", 10, "native_count"], False),
                    (["fixtures", 10, "original_error"], "must contain package entries"),
                    (["fixtures", 15, "native_error"], "Cargo.lock requires [[package]] entries"),
                    (["fixtures", 20, "stage"], "array-rejected"),
                    (["fixtures", 23, "array_count"], 1)]
        self.rejects(self.report, changes, lambda report: evidence.validate_semantics(report, self.files))

    def test_whole_lock_and_empty_counts_are_distinct(self):
        changes = [(["observations"], []), (["observations"], self.report["observations"] * 2),
                   (["empty_counts"], []), (["empty_counts"], self.report["empty_counts"][::-1])]
        changes += [(["observations", 0, key], value) for key, value in {
            "policy_id": "other", "policy": {}, "scope": "reachable-only", "quality": "partial",
            "lock_sha256": "0" * 64, "lock_node_count": 5, "observed_count": True,
            "observed_sample": [], "observed_omitted": 1, "exact_difference": {}, "satisfied": 1,
        }.items()]
        changes += [(["empty_counts", 0, key], value) for key, value in {
            "policy_id": "other", "original_error": "Cargo.lock must contain package entries",
            "native_diagnostic": "CARGO-001", "observed": {},
        }.items()]
        changes += [(["empty_counts", 0, "observed", key], value) for key, value in {
            "lock_node_count": False, "observed_count": 1, "satisfied": True,
            "scope": "reachable-only", "observed_sample": [{}], "exact_difference": {},
        }.items()]
        self.rejects(self.report, changes, lambda report: evidence.validate_semantics(report, self.files))

    def test_execution_requires_exact_tests_and_explicit_qualifier(self):
        self.rejects(self.report, [(["total_tests"], True), (["executions"], []),
            (["executions"], self.report["executions"][::-1]), (["executions", 0, "test"], "other"),
            (["executions", 0, "cases"], True), (["executions", 0, "outcome", "passed"], True),
            (["executions", 0, "outcome", "passed"], 0), (["executions", 0, "outcome", "ignored"], 1),
            (["executions", 0, "outcome", "filtered"], 644), (["executions", 0, "outcome", "failed"], 1),
            (["executions", 4, "arguments"], self.report["executions"][4]["arguments"][:-1]),
            (["executions", 1, "scope"], "full-repository"), (["executions", 4, "cases"], 0)], evidence.validate_executions)

    def test_raw_logs_bind_both_complete_runs(self):
        self.rejects(self.report, [(["listing_sha256"], "0" * 64), (["native_payload_sha256"], "0" * 64),
                                  (["executions", 0, "outcome", "passed"], 0)], evidence.validate_logs)

    def test_linkage_cannot_expand_the_array_precondition(self):
        changes = [(["id"], "KD-PROVENANCE-LOCK-PACKAGE-NAMES"), (["commit"], "0" * 40),
                   (["disposition"], "approved retirement"), (["source", "instance"], []),
                   (["source", "file_sha256"], "0" * 64), (["source", "function"], ["parse"]),
                   (["source", "line"], 149), (["source", "syntax_sha256"], "0" * 64),
                   (["invocations"], self.assertion["invocations"][:-1]),
                   (["selection", "paths"], ["**/Cargo.lock"]), (["selection", "keys"], ["name"]),
                   (["matching_semantics", "kind"], "nonempty array"),
                   (["matching_semantics", "empty_array"], "rejected"),
                   (["required_cardinality", "scope"], "reachable packages only"),
                   (["replacement", "policy_ids"], []), (["replacement", "expected_diagnostics"], ["DEP-LOCK-001"]),
                   (["replacement", "expected_diagnostics"], ["CARGO-001"]),
                   (["replacement", "implemented"], False), (["replacement", "verified"], False),
                   (["replacement", "full_snapshot_verified"], True), (["replacement", "qualification_scope"], "full provenance"),
                   (["replacement", "fixtures"], []), (["replacement", "array_evidence"], {}), (["blockers"], [{"id": "open"}])]
        with patch.object(evidence, "verify_report"):
            self.rejects(self.assertion, changes, lambda assertion: evidence.verify(assertion, self.report, self.files))


if __name__ == "__main__":
    unittest.main()
