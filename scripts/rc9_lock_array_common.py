"""Exact suite identities for the original package-array precondition and native failure stages."""

from rc9_retired_boundary_common import ROOT, canonical, execution, git, parse, require, sha

POLICY = "docs/rc9/policies/kafka-driver.lock-packages.fragment.toml"
CASES = "crates/zrail-testkit/tests/fixtures/rc9/lock-array-cases.json"
ORIGINS = "crates/zrail-testkit/tests/fixtures/rc9/lock-array-origins.json"
PREFIX = "lock_packages::lock_packages_test::legacy::array::"
SUITES = {
    "lock_array_original_precondition_is_an_exact_frozen_excerpt": (False, "source-binding", 0),
    "lock_array_type_matrix_preserves_failure_stages": (False, "array-and-graph-stages", 24),
    "lock_array_empty_type_passes_but_all_five_required_counts_fail": (False, "empty-array-required-counts", 5),
    "lock_array_complete_original_and_native_accept_frozen_inputs": (False, "original-and-native-frozen-fixture", 5),
    "qualify_frozen_lock_array": (True, "frozen-array-and-whole-lock-binding", 24),
}
