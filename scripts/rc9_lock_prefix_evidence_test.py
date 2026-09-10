"""Reject incomplete prefix proofs and hidden range authority and manufactured execution."""

import copy
import gzip
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True

import rc9_lock_prefix_evidence as evidence
from rc9_lock_prefix_common import ROOT, parse


def changed(value, path, replacement):
    value = copy.deepcopy(value)
    parent = value
    for key in path[:-1]:
        parent = parent[key]
    parent[path[-1]] = replacement
    return value


class LockPrefixEvidence(unittest.TestCase):
    rejected = 0

    @classmethod
    def setUpClass(cls):
        cls.report = parse(gzip.open(ROOT / evidence.ARTIFACT, "rb").read())
        census = parse(gzip.open(ROOT / "crates/zrail-testkit/tests/fixtures/rc9/census.json.gz", "rb").read())
        cls.files = {(row["repository"], row["path"]): row for row in census["files"]}
        ledger = parse((ROOT / "crates/zrail-testkit/tests/fixtures/rc9/assertions.json").read_bytes())
        cls.assertions = [row for row in ledger["reviewed_assertions"] if row["id"].startswith("KD-PROVENANCE-") and row["id"].endswith("-VERSION-PREFIX")]
        assert len(cls.assertions) == 5

    @classmethod
    def tearDownClass(cls):
        print(f"{cls.rejected} adversarial mutations rejected")

    def rejects(self, value, changes, validate):
        for path, replacement in changes:
            with self.subTest(path=path, replacement=replacement), self.assertRaises(ValueError):
                validate(changed(value, path, replacement))
            type(self).rejected += 1

    def test_bound_report_and_assertion(self):
        for assertion in self.assertions:
            evidence.verify(assertion, self.report, self.files)

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
            "literals": [], "conversion": {}, "translated_sha256": "0" * 64, "observations": [], "inputs": {}, "executions": [],
        }.items()]
        self.rejects(self.report, changes, lambda report: evidence.verify_report(report, self.files))

    def test_prefix_and_native_stages_independent_of_payload_digest(self):
        changes = [(["schema"], True), (["original_helper_invocations"], True), (["full_repository_qualified"], True),
                   (["inputs"], {}), (["workspace_packages"], {}), (["source_sha256"], "0" * 64),
                   (["original_origins_sha256"], "0" * 64), (["cases_sha256"], "0" * 64),
                   (["fixtures"], self.report["fixtures"][:-1]), (["fixtures"], self.report["fixtures"][::-1])]
        changes += [(["fixtures", 0, key], value) for key, value in {
            "policy_id": "other", "case": "bare", "input_version": "other", "suffix": None,
            "prefix_error": "failed", "original_helper_error": "failed", "native_version": "^1.0.0",
        }.items()]
        changes += [(["fixtures", 0, "native", key], value) for key, value in {
            "contract_source": "", "lock_source": "version = 4", "contract_sha256": "0" * 64,
            "lock_sha256": "0" * 64, "contract_error": "generic error", "graph_error": "generic error", "observation": None,
        }.items()]
        changes += [(["fixtures", 0, "native", "observation", key], value) for key, value in {
            "scope": "reachable-only", "observed_count": True, "lock_node_count": 83,
            "satisfied": False, "policy": {}, "observed_sample": [], "exact_difference": {},
        }.items()]
        changes += [(["fixtures", 1, "prefix_error"], None), (["fixtures", 1, "original_helper_error"], None),
                    (["fixtures", 1, "native", "observation"], None),
                    (["fixtures", 9, "original_helper_error"], "invalid semver"),
                    (["fixtures", 9, "native", "graph_error"], "generic error"),
                    (["fixtures", 10, "native", "contract_error"], None),
                    (["fixtures", 10, "native", "graph_error"], None),
                    (["fixtures", 11, "native", "observation"], self.report["observations"][0])]
        self.rejects(self.report, changes, lambda report: evidence.validate_semantics(report, self.files))

    def test_exact_literal_and_full_frozen_observations(self):
        changes = [(["observations"], []), (["observations"], self.report["observations"] * 2),
                   (["literals"], []), (["literals"], self.report["literals"][::-1])]
        changes += [(["observations", 0, key], value) for key, value in {
            "policy_id": "other", "policy": {}, "scope": "reachable-only", "quality": "partial",
            "lock_sha256": "0" * 64, "lock_node_count": 5, "observed_count": True,
            "observed_sample": [], "observed_omitted": 1, "exact_difference": {}, "satisfied": 1,
        }.items()]
        changes += [(["literals", 0, key], value) for key, value in {
            "policy_id": "other", "case": "wildcard", "expected_version": "1.0.0",
            "original_helper_error": "bornera guardrail version must be exact",
            "native_diagnostic": "CARGO-001", "observation": {},
        }.items()]
        changes += [(["literals", 0, "observation", key], value) for key, value in {
            "satisfied": True, "observed_count": 0, "scope": "reachable-only", "exact_difference": {},
        }.items()]
        changes += [(["literals", 3, "observation", "satisfied"], True)]
        self.rejects(self.report, changes, lambda report: evidence.validate_semantics(report, self.files))

    def test_converter_cannot_accept_new_or_unprefixed_authority(self):
        changes = [(["translated_sha256"], "0" * 64), (["conversion", "section"], "repository.files"),
                   (["conversion", "rules"], []), (["conversion", "cases"], []),
                   (["conversion", "cases", 0, "error"], "failed"),
                   (["conversion", "cases", 1, "error"], None),
                   (["conversion", "cases", 20, "error"], None),
                   (["conversion", "linkage_rejections"], []),
                   (["conversion", "linkage_rejections", 0, "error"], None),
                   (["conversion", "linkage_rejections", 0, "package"], "other")]
        self.rejects(self.report, changes, lambda report: evidence.validate_semantics(report, self.files))


    def test_execution_requires_exact_tests_and_explicit_qualifier(self):
        self.rejects(self.report, [(["total_tests"], True), (["executions"], []),
            (["executions"], self.report["executions"][::-1]), (["executions", 0, "test"], "other"),
            (["executions", 0, "cases"], True), (["executions", 0, "outcome", "passed"], True),
            (["executions", 0, "outcome", "passed"], 0), (["executions", 0, "outcome", "ignored"], 1),
            (["executions", 0, "outcome", "filtered"], 654), (["executions", 0, "outcome", "failed"], 1),
            (["executions", 4, "arguments"], self.report["executions"][4]["arguments"][:-1]),
            (["executions", 1, "scope"], "full-repository"), (["executions", 4, "cases"], 0)], evidence.validate_executions)

    def test_raw_logs_bind_both_complete_runs(self):
        self.rejects(self.report, [(["listing_sha256"], "0" * 64), (["native_payload_sha256"], "0" * 64),
                                  (["executions", 0, "outcome", "passed"], 0)], evidence.validate_logs)

    def test_linkage_preserves_each_of_the_five_prefix_registry_instances(self):
        changes = [(["id"], "KD-PROVENANCE-LOCK-PACKAGE-NAMES"), (["commit"], "0" * 40),
                   (["disposition"], "approved retirement"), (["source", "instance"], "other"),
                   (["source", "file_sha256"], "0" * 64), (["source", "function"], ["parse"]),
                   (["source", "line"], 150), (["source", "syntax_sha256"], "0" * 64),
                   (["invocations"], []), (["selection", "paths"], ["Cargo.lock"]),
                   (["selection", "keys"], ["dependencies", "other_version"]),
                   (["policy_registry", "file_sha256"], "0" * 64), (["policy_registry", "entry"], "1.0.0"),
                   (["policy_registry", "selector"], ["other"]),
                   (["matching_semantics", "kind"], "semver requirement"), (["matching_semantics", "prefix"], "^"),
                   (["matching_semantics", "remaining_text_used_as_exact_lock_version"], 1),
                   (["required_cardinality", "scope"], "reachable packages only"),
                   (["replacement", "policy_ids"], []), (["replacement", "expected_diagnostics"], ["DEP-LOCK-001"]),
                   (["replacement", "implemented"], False), (["replacement", "verified"], False),
                   (["replacement", "full_snapshot_verified"], True), (["replacement", "qualification_scope"], "full registry"),
                   (["replacement", "fixtures"], []), (["replacement", "prefix_evidence"], {}), (["blockers"], [{"id": "open"}])]
        with patch.object(evidence, "verify_report"):
            for assertion in self.assertions:
                self.rejects(assertion, changes, lambda row: evidence.verify(row, self.report, self.files))


if __name__ == "__main__":
    unittest.main()
