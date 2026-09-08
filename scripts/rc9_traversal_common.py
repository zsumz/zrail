"""Closed source-traversal qualification identities; shared parsers never execute consumer code."""

from rc9_retired_boundary_common import ROOT, canonical, execution, git, parse, require, run, sha

POLICY = "docs/rc9/policies/kafka-driver.traversal.fragment.toml"
PREFIX = "repository_files::repository_files_test::traversal::"
ROOTS = ["src", "tests", "crates/kafka-driver-core/src", "crates/kafka-driver-probe/src",
         "crates/kafka-driver-sim/src", "crates/kafka-driver-transport/src"]
SUITES = {
    "traversal_original_function_is_frozen": (False, "source-binding", 0),
    "traversal_roots_empty_trees_and_selection_are_explicit": (False, "portable-physical", 42),
    "unix::traversal_links_and_special_entries_preserve_explicit_boundaries": (False, "unix-physical", 66),
    "unix::traversal_unread_directories_fail_and_unread_files_need_no_content_read": (True, "real-permissions", 19),
    "injected::traversal_injected_failure_clauses_are_exact_frozen_excerpts": (False, "source-binding", 0),
    "injected::traversal_injected_errors_preserve_each_original_failure_stage": (False, "injected-expressions-and-native-scan", 96),
    "qualify_frozen_source_traversal": (True, "frozen-selection-and-complete-matrix", 223),
}
