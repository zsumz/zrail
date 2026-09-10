"""Reject incomplete name proofs, hidden stricter rejection and manufactured execution."""

import copy
import gzip
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True

import rc9_lock_names_evidence as evidence
from rc9_lock_names_common import ROOT, parse


def changed(value, path, replacement):
    value = copy.deepcopy(value)
    parent = value
    for key in path[:-1]:
        parent = parent[key]
    parent[path[-1]] = replacement
    return value


class LockNamesEvidence(unittest.TestCase):
    rejected = 0

    @classmethod
    def setUpClass(cls):
        cls.report = parse(gzip.open(ROOT / evidence.ARTIFACT, "rb").read())
        census = parse(gzip.open(ROOT / "crates/zrail-testkit/tests/fixtures/rc9/census.json.gz", "rb").read())
        cls.files = {(row["repository"], row["path"]): row for row in census["files"]}
        ledger = parse((ROOT / "crates/zrail-testkit/tests/fixtures/rc9/assertions.json").read_bytes())
        cls.assertion, = [row for row in ledger["reviewed_assertions"] if row["id"] == "KD-PROVENANCE-LOCK-PACKAGE-NAMES"]

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
            "mutations": [], "mutations_sha256": "0" * 64, "observations": [], "inputs": {}, "executions": [],
        }.items()]
        self.rejects(self.report, changes, lambda report: evidence.verify_report(report, self.files))

    def test_name_stages_independent_of_payload_digest(self):
        changes = [(["schema"], True), (["original_helper_invocations"], True),
                   (["full_repository_qualified"], True), (["inputs"], {}), (["workspace_packages"], {}),
                   (["source_sha256"], "0" * 64), (["original_origins_sha256"], "0" * 64),
                   (["cases_sha256"], "0" * 64), (["mutations_sha256"], "0" * 64),
                   (["fixtures"], self.report["fixtures"][:-1]), (["fixtures"], self.report["fixtures"][::-1]),
                   (["fixtures"], self.report["fixtures"] * 2)]
        changes += [(["fixtures", 0, key], value) for key, value in {
            "case": "target", "source_sha256": "0" * 64, "original_count": False,
            "original_error": "unrelated failure", "native_count": False,
            "native_error": "generic nonzero exit", "stage": "missing-name",
        }.items()]
        changes += [(["fixtures", 10, "original_error"], None), (["fixtures", 10, "original_count"], 0),
                    (["fixtures", 17, "original_error"], "index not found"),
                    (["fixtures", 17, "native_error"], None),
                    (["fixtures", 23, "stage"], "same-selection"),
                    (["fixtures", 27, "native_error"], "Cargo.lock package requires string name"),
                    (["fixtures", 9, "original_count"], 1), (["fixtures", 9, "native_count"], 1),
                    (["fixtures", 31, "original_count"], 0)]
        self.rejects(self.report, changes, lambda report: evidence.validate_semantics(report, self.files))

    def test_frozen_mutations_bind_complete_original_and_native_scope(self):
        changes = [(["observations"], []), (["observations"], self.report["observations"] * 2),
                   (["mutations"], []), (["mutations"], self.report["mutations"][::-1]),
                   (["mutations"], self.report["mutations"] * 2)]
        changes += [(["observations", 0, key], value) for key, value in {
            "policy_id": "other", "policy": {}, "scope": "reachable-only", "quality": "partial",
            "lock_sha256": "0" * 64, "lock_node_count": 5, "observed_count": True,
            "observed_sample": [], "observed_omitted": 1, "exact_difference": {}, "satisfied": 1,
        }.items()]
        changes += [(["mutations", 0, key], value) for key, value in {
            "case": "append-unrelated", "original_error": "index not found", "native_error": "unrelated",
            "lock_source": "", "inputs": {}, "observations": [], "stage": "missing-name",
        }.items()]
        changes += [(["mutations", 3, "original_error"], None), (["mutations", 3, "native_error"], None),
                    (["mutations", 3, "observations"], self.report["observations"]),
                    (["mutations", 6, "original_error"], "index not found"),
                    (["mutations", 6, "native_error"], None),
                    (["mutations", 6, "stage"], "same-acceptance"),
                    (["mutations", 1, "observations", 0, "lock_node_count"], 83),
                    (["mutations", 1, "observations", 0, "observed_count"], 0),
                    (["mutations", 1, "observations", 0, "scope"], "reachable-only"),
                    (["mutations", 1, "inputs", "Cargo.toml"], "0" * 64)]
        self.rejects(self.report, changes, lambda report: evidence.validate_semantics(report, self.files))


    def test_execution_requires_exact_tests_and_explicit_qualifier(self):
        self.rejects(self.report, [(["total_tests"], True), (["executions"], []),
            (["executions"], self.report["executions"][::-1]), (["executions", 0, "test"], "other"),
            (["executions", 0, "cases"], True), (["executions", 0, "outcome", "passed"], True),
            (["executions", 0, "outcome", "passed"], 0), (["executions", 0, "outcome", "ignored"], 1),
            (["executions", 0, "outcome", "filtered"], 649), (["executions", 0, "outcome", "failed"], 1),
            (["executions", 4, "arguments"], self.report["executions"][4]["arguments"][:-1]),
            (["executions", 1, "scope"], "full-repository"), (["executions", 4, "cases"], 0)], evidence.validate_executions)

    def test_raw_logs_bind_both_complete_runs(self):
        self.rejects(self.report, [(["listing_sha256"], "0" * 64), (["native_payload_sha256"], "0" * 64),
                                  (["executions", 0, "outcome", "passed"], 0)], evidence.validate_logs)

    def test_linkage_cannot_expand_the_name_precondition(self):
        changes = [(["id"], "KD-PROVENANCE-kafka-wire-VERSION-PREFIX"), (["commit"], "0" * 40),
                   (["disposition"], "approved retirement"), (["source", "instance"], []),
                   (["source", "file_sha256"], "0" * 64), (["source", "function"], ["parse"]),
                   (["source", "line"], 150), (["source", "syntax_sha256"], "0" * 64),
                   (["invocations"], self.assertion["invocations"][:-1]),
                   (["selection", "paths"], ["**/Cargo.lock"]), (["selection", "keys"], ["name"]),
                   (["matching_semantics", "kind"], "nonempty string"),
                   (["matching_semantics", "source_expression"], "first matching node only"),
                   (["matching_semantics", "helper_line"], 149),
                   (["matching_semantics", "missing_name"], "ignored"),
                   (["matching_semantics", "present_non_string_name"], "rejected by original"),
                   (["required_cardinality", "scope"], "reachable packages only"),
                   (["replacement", "policy_ids"], []), (["replacement", "expected_diagnostics"], ["DEP-LOCK-001"]),
                   (["replacement", "expected_diagnostics"], ["CARGO-001"]),
                   (["replacement", "implemented"], False), (["replacement", "verified"], False),
                   (["replacement", "full_snapshot_verified"], True), (["replacement", "qualification_scope"], "full provenance"),
                   (["replacement", "fixtures"], []), (["replacement", "names_evidence"], {}), (["blockers"], [{"id": "open"}])]
        with patch.object(evidence, "verify_report"):
            self.rejects(self.assertion, changes, lambda assertion: evidence.verify(assertion, self.report, self.files))


if __name__ == "__main__":
    unittest.main()
