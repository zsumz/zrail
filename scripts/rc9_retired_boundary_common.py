"""Closed suite identities and strict execution parsing for retired-tree qualification."""

import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parent.parent
POLICY = "docs/rc9/policies/kafka-driver.retired-boundary.fragment.toml"
PREFIX = "rust_inventories::rust_inventories_test::retired_test::"
QUALIFIER = PREFIX + "qualify_frozen_retired_boundary_bundle"
SUITES = {
    "retired_physical_oracles_retain_the_complete_original_function_bodies": (False, "source-binding", 0),
    "boundary::retired_non_directory_proposal_preserves_all_ordinary_predicates": (False, "ordinary-synthetic", 144),
    "boundary::retired_missing_empty_and_pruned_trees_have_explicit_outcomes": (False, "physical", 35),
    "boundary::unix::retired_contained_file_links_preserve_extension_selection": (False, "physical", 21),
    "boundary::unix::retired_directory_links_never_claim_complete_descendant_selection": (False, "physical", 28),
    "boundary::unix::retired_broken_links_are_incomplete_even_when_the_legacy_extension_passes": (False, "physical", 14),
    "boundary::unix::retired_directory_cycles_fail_closed_without_running_the_unbounded_oracle": (False, "native-only-cycle", 7),
    "boundary::unix::retired_special_entries_need_non_directory_selection_without_content_reads": (False, "physical", 21),
    "boundary::unix::retired_unreadable_directories_fail_closed_and_unreadable_files_still_count": (True, "real-permissions", 21),
    "entry_test::retired_entry_expression_is_an_exact_frozen_excerpt": (False, "source-binding", 0),
    "entry_test::retired_injected_entries_are_ignored_by_the_original_expression_but_rejected_by_inventory": (False, "injected-iterator-expression", 56),
    "entry_test::retired_injected_directory_and_metadata_failures_abort_the_entire_scan": (False, "injected-native-scan", 28),
}


def require(condition, message):
    if not condition:
        raise ValueError("retired boundaries: " + message)


def sha(payload):
    return hashlib.sha256(payload).hexdigest()


def canonical(value):
    return (json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n").encode()


def parse(payload):
    require(len(payload) <= 64 * 1024 * 1024, "oversized JSON")

    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "duplicate JSON key")
            result[key] = value
        return result

    return json.loads(payload, object_pairs_hook=unique)


def run(arguments, env=None):
    return subprocess.run([str(arg) for arg in arguments], cwd=ROOT, env=env,
                          capture_output=True, timeout=1800)


def git(*args):
    result = run(["git", *args])
    require(result.returncode == 0, "Git observation failed")
    return result.stdout.decode().strip()


def execution(stdout, code, name, total):
    require(code == 0, "failed execution")
    text = stdout.decode()
    require(re.findall(r"^test (\S+) \.\.\. (\S+)$", text, re.M) == [(name, "ok")],
            "missing, ignored, extra or unrelated executed test")
    counts = re.findall(r"^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; "
                        r"(\d+) measured; (\d+) filtered out; finished in [0-9.]+s$", text, re.M)
    require(counts == [("1", "0", "0", "0", str(total - 1))], "wrong execution counts")
    return {"test": name, "passed": 1, "failed": 0, "ignored": 0, "measured": 0, "filtered": total - 1}
