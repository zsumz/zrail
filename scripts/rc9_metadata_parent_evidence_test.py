"""Reject partial parent proofs, changed license mappings and manufactured execution."""

import copy
import gzip
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True

import rc9_metadata_parent_evidence as evidence
from rc9_metadata_parent_common import ROOT, parse


def changed(value, path, replacement):
    value = copy.deepcopy(value)
    parent = value
    for key in path[:-1]:
        parent = parent[key]
    parent[path[-1]] = replacement
    return value


class MetadataParentEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = parse(gzip.open(ROOT / evidence.ARTIFACT, "rb").read())
        census = parse(gzip.open(ROOT / "crates/zrail-testkit/tests/fixtures/rc9/census.json.gz", "rb").read())
        cls.files = {(row["repository"], row["path"]): row for row in census["files"]}
        ledger = parse((ROOT / "crates/zrail-testkit/tests/fixtures/rc9/assertions.json").read_bytes())
        cls.assertion, = [row for row in ledger["reviewed_assertions"] if row["id"] == "KD-METADATA-PARENT"]

    def rejects(self, value, changes, validate):
        for path, replacement in changes:
            with self.subTest(path=path, replacement=replacement), self.assertRaises(ValueError):
                validate(changed(value, path, replacement))

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
            "original_metadata_body_executed": False, "original_parent_instances": 0, "path_proof": [],
            "observations": [], "inputs": {}, "executions": [],
        }.items()]
        self.rejects(self.report, changes, lambda report: evidence.verify_report(report, self.files))

    def test_path_and_native_structure_independent_of_payload_digest(self):
        changes = [(["schema"], True), (["original_metadata_body_executed"], 1),
                   (["original_parent_instances"], True), (["full_repository_qualified"], True),
                   (["path_proof"], self.report["path_proof"][:-1]),
                   (["path_proof"], self.report["path_proof"][::-1]),
                   (["path_proof"], self.report["path_proof"] * 2), (["inputs"], {}),
                   (["source_sha256"], "0" * 64), (["observations"], []),
                   (["observations"], self.report["observations"] * 2)]
        changes += [(["path_proof", 0, key], value) for key, value in {
            "root_shape": "filesystem-observation", "name": "other", "manifest": "", "parent": "..",
            "license": "crates/kafka-driver-core/LICENSE", "policy_id": "repository:file:other",
        }.items()]
        changes += [(["observations", 0, key], value) for key, value in {
            "policy_id": "repository:file:other", "policy": {}, "analysis": "partial", "satisfied": 1,
            "claim": "cargo-package-license", "checkout_path": "/other", "entries": [],
        }.items()]
        changes += [(["observations", 0, "entries", 0, key], value) for key, value in {
            "path": "LICENSE", "kind": "directory", "resolved_path": "other", "sha256": "0" * 64,
            "bytes": True, "valid_utf8": False, "satisfied": False,
        }.items()]
        changes += [(["observations", 0, "reference", key], value) for key, value in {
            "path": "other", "sha256": "0" * 64, "valid_utf8": 1, "bytes": 0,
        }.items()]
        self.rejects(self.report, changes, lambda report: evidence.validate_semantics(report, self.files))

    def test_execution_requires_exact_tests_and_explicit_qualifier(self):
        self.rejects(self.report, [(["total_tests"], True), (["executions"], []),
            (["executions"], self.report["executions"][::-1]), (["executions", 0, "test"], "other"),
            (["executions", 0, "cases"], True), (["executions", 0, "outcome", "passed"], True),
            (["executions", 0, "outcome", "passed"], 0), (["executions", 0, "outcome", "ignored"], 1),
            (["executions", 0, "outcome", "filtered"], 639), (["executions", 0, "outcome", "failed"], 1),
            (["executions", 4, "arguments"], self.report["executions"][4]["arguments"][:-1]),
            (["executions", 1, "scope"], "filesystem-permissions"), (["executions", 4, "cases"], 0)], evidence.validate_executions)

    def test_raw_logs_bind_both_complete_runs(self):
        self.rejects(self.report, [(["listing_sha256"], "0" * 64), (["native_payload_sha256"], "0" * 64),
                                  (["executions", 0, "outcome", "passed"], 0)], evidence.validate_logs)

    def test_linkage_cannot_expand_the_fixed_parent_precondition(self):
        changes = [(["id"], "KD-METADATA-OTHER"), (["commit"], "0" * 40), (["disposition"], "approved retirement"),
                   (["source", "instance"], []), (["source", "file_sha256"], "0" * 64),
                   (["source", "function"], ["parse"]), (["selection", "paths"], ["**/Cargo.toml"]),
                   (["matching_semantics", "kind"], "Cargo manifest discovery"),
                   (["matching_semantics", "proof"], "all possible paths"), (["required_cardinality", "exact"], 2),
                   (["replacement", "policy_ids"], []), (["replacement", "expected_diagnostics"], ["REP-FILE-005"]),
                   (["replacement", "implemented"], False), (["replacement", "verified"], False),
                   (["replacement", "full_snapshot_verified"], True), (["replacement", "qualification_scope"], "full metadata"),
                   (["replacement", "fixtures"], []), (["replacement", "parent_evidence"], {}), (["blockers"], [{"id": "open"}])]
        with patch.object(evidence, "verify_report"):
            self.rejects(self.assertion, changes, lambda assertion: evidence.verify(assertion, self.report, self.files))


if __name__ == "__main__":
    unittest.main()
