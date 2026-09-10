"""Exact suite identities for the two frozen dependency detector assertions."""

from rc9_retired_boundary_common import ROOT, canonical, execution, git, parse, require, sha

POLICY = "docs/rc9/policies/kafka-driver.dependency-detector.fixture.toml"
CASES = "crates/zrail-testkit/tests/fixtures/rc9/dependency-detector-cases.json"
ORIGINS = "crates/zrail-testkit/tests/fixtures/rc9/dependency-detector-origins.json"
PREFIX = "repository_files::repository_files_test::raw_dependency::legacy::detector::"
SUITES = {
    "original::package_extraction_matches_exact_names_only": (False, "original-inline-detector", 2),
    "dependency_detector_original_excerpts_are_bound": (False, "source-binding", 0),
    "dependency_detector_lines_match_complete_original_name_sets": (False, "ordinary-line-matrix", 32),
    "qualify_frozen_dependency_detector": (True, "frozen-source-bound-line-matrix", 32),
}
