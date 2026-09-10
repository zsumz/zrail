"""Exact suite identities for whole-array name indexing and existing stricter native validation."""

from rc9_retired_boundary_common import ROOT, canonical, execution, git, parse, require, sha

POLICY = "docs/rc9/policies/kafka-driver.lock-packages.fragment.toml"
CASES = "crates/zrail-testkit/tests/fixtures/rc9/lock-names-cases.json"
MUTATIONS = "crates/zrail-testkit/tests/fixtures/rc9/lock-names-mutations.json"
ORIGINS = "crates/zrail-testkit/tests/fixtures/rc9/lock-names-origins.json"
PREFIX = "lock_packages::lock_packages_test::legacy::names::"
SUITES = {
    "lock_names_original_selection_is_an_exact_frozen_excerpt": (False, "source-binding", 0),
    "lock_names_matrix_preserves_whole_array_selection_and_stricter_errors": (False, "name-selection-and-graph-stages", 32),
    "lock_names_unrelated_mutations_execute_the_complete_original_guard": (False, "complete-original-unrelated-mutations", 16),
    "lock_names_complete_original_and_native_accept_frozen_inputs": (False, "original-and-native-frozen-fixture", 5),
    "qualify_frozen_lock_names": (True, "frozen-name-and-whole-lock-binding", 32),
}
