"""Exact execution identities for the fixed metadata-parent precondition."""

from rc9_retired_boundary_common import ROOT, canonical, execution, git, parse, require, sha

POLICY = "docs/rc9/policies/kafka-driver.metadata.fragment.toml"
ORIGINS = "crates/zrail-testkit/tests/fixtures/rc9/metadata-origins.json"
PREFIX = "repository_files::repository_files_test::metadata::legacy::parent::"
SUITES = {
    "metadata_parent_original_body_and_registry_are_unchanged": (False, "source-binding", 0),
    "metadata_parent_fixed_paths_preserve_all_twelve_parent_mappings": (False, "lexical-path-mapping", 12),
    "metadata_parent_mapping_rejects_changed_license_authority": (False, "mapping-mutations", 7),
    "metadata_parent_original_and_native_accept_exact_frozen_fixture_inputs": (False, "original-and-native-fixture", 3),
    "qualify_frozen_metadata_parent": (True, "frozen-path-and-native-binding", 12),
}
