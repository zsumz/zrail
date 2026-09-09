"""Exact suite identities for exact-prefix translation and native literal comparison."""

from rc9_retired_boundary_common import ROOT, canonical, execution, git, parse, require, sha

POLICY = "docs/rc9/policies/kafka-driver.lock-packages.fragment.toml"
CASES = "crates/zrail-testkit/tests/fixtures/rc9/lock-prefix-cases.json"
ORIGINS = "crates/zrail-testkit/tests/fixtures/rc9/lock-prefix-origins.json"
PREFIX = "lock_packages::lock_packages_test::legacy::prefix::"
SUITES = {
    "lock_prefix_original_operation_is_an_exact_frozen_excerpt": (False, "source-binding", 0),
    "lock_prefix_pairs_distinguish_source_and_native_representations": (False, "paired-prefix-and-native-representations", 120),
    "lock_prefix_native_versions_remain_literal_not_semver_ranges": (False, "complete-frozen-literal-comparisons", 20),
    "lock_prefix_complete_original_and_native_accept_frozen_inputs": (False, "original-and-native-frozen-fixture", 5),
    "qualify_frozen_lock_prefix": (True, "frozen-prefix-and-whole-lock-binding", 120),
}
