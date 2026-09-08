"""Expected traversal case identities and outcomes, independently checked from the artifact."""

from pathlib import PurePosixPath
import tomllib

from rc9_traversal_common import ROOT, ROOTS, POLICY, require


def policies():
    values = tomllib.loads((ROOT / POLICY).read_text())["repository"]["files"]
    result = {}
    for row in values:
        row = {"exclude": [], **row}
        if row["predicate"]["kind"] == "count":
            row["predicate"] = {"maximum": None, **row["predicate"]}
        result["repository:file:" + row["name"]] = row
    require(set(result) == {"repository:file:kd-source-roots", "repository:file:kd-source-traversal"}, "wrong traversal policy bundle")
    return result


def matrix():
    result = {}
    portable = ["missing-root", "file-root", "empty", "nested-empty", "ordered-source", "component-order", "git-empty", "git-source"]
    unix = ["file-link", "directory-link", "directory-rs-link", "escaping-file-link", "broken-link", "cycle", "root-empty-link", "root-source-link",
            "fifo-pipe.rs", "fifo-pipe.RS", "fifo-.rs"]
    for cases in [portable, unix, ["unread-empty", "unread-nested", "unread-file"]]:
        for root in ROOTS:
            for case in cases:
                failure = case in {"missing-root", "file-root", "unread-empty", "unread-nested"}
                incomplete = case in {"git-empty", "git-source", "directory-link", "directory-rs-link", "escaping-file-link", "broken-link", "cycle",
                                      "root-empty-link", "root-source-link", "unread-empty", "unread-nested"}
                relative = {
                    "ordered-source": ["a.rs", "nested/cfg.rs", "z.rs"],
                    "component-order": ["a/nested.rs", "a.rs"], "git-source": [".git/hidden.rs"],
                    "file-link": ["link.rs"], "directory-rs-link": ["linked.rs"], "escaping-file-link": ["link.rs"],
                    "broken-link": ["link.rs"], "cycle": ["loop.rs"], "root-source-link": ["visible.rs"],
                    "fifo-pipe.rs": ["pipe.rs"], "unread-file": ["source.rs"],
                }.get(case, [])
                original = [root + "/" + path for path in relative]
                native = [] if incomplete else sorted(original)
                if case in {"missing-root", "file-root"}:
                    native = [next(other for other in ROOTS if other != root) + "/surviving.rs"]
                result[root + ":" + case] = (failure, incomplete, original, native)
    result["outside:unread-directory"] = (False, True, [], [])
    require(len(result) == 133, "wrong physical matrix size")
    return result


def validate_physical(rows):
    expected = matrix()
    require(len(rows) == len(expected) and [row["case"] for row in rows] == list(expected), "missing, duplicated or reordered physical cases")
    policy_map = policies()
    for row in rows:
        failure, incomplete, original, native = expected[row["case"]]
        root, case = row["case"].split(":", 1)
        require(row["legacy_paths"] == original and row["native_paths"] == native, "wrong complete path observation")
        require((row["legacy_error"] is not None) is failure and (row["native_error"] is not None) is incomplete, "wrong physical failure outcome")
        if failure:
            require(row["legacy_error"].startswith("read directory $ROOT/" + root), "wrong original failure stage")
            if case.startswith("unread-"):
                require("Permission denied" in row["legacy_error"], "missing actual OS permission failure")
        if incomplete:
            require(row["native_error"].startswith("REP-FILE-006:") and root in row["native_error"]
                    and not row["observations"] and not row["diagnostics"], "partial or unrelated native failure")
            continue
        diagnostics = [["repository:file:kd-source-roots", "REP-FILE-002"]] if failure else []
        require(row["diagnostics"] == diagnostics and len(row["observations"]) == 2, "wrong exact-root diagnostic")
        require([observed["policy_id"] for observed in row["observations"]] == list(policy_map), "missing or duplicate physical policy")
        for observed in row["observations"]:
            identity = observed["policy_id"]
            require(observed["policy"] == policy_map[identity] and observed["analysis"] == "exact"
                    and observed["claim"] == "physical-paths" and observed["satisfied"] is (not failure or identity.endswith("traversal")), "wrong physical policy")
            require(all(entry["sha256"] is None and entry["bytes"] is None for entry in observed["entries"]), "physical inspection fabricated content reads")
            if identity.endswith("roots"):
                roots = sorted(set(ROOTS) - ({root} if failure else set()))
                require([entry["path"] for entry in observed["entries"]] == roots
                        and all(entry["kind"] == "directory" for entry in observed["entries"]), "missing root masked by other source")
            else:
                paths = [entry["path"] for entry in observed["entries"] if entry["kind"] != "directory" and PurePosixPath(entry["path"]).suffix == ".rs"]
                require(paths == native, "observed traversal paths differ")


def validate_injected(rows):
    expected = {}
    for root in ROOTS:
        for source in ["false", "true"]:
            for kind in ["PermissionDenied", "NotFound"]:
                for fault in ["Directory", "EntryFirst", "EntryLast", "Metadata"]:
                    path = "$ROOT/" + root + "/nested"
                    before, after = {"Directory": ("read directory " + path, "read " + path),
                                     "EntryFirst": ("read directory entry", "read entry"),
                                     "EntryLast": ("read directory entry", "read entry"),
                                     "Metadata": ("inspect " + path, "inspect " + path)}[fault]
                    expected[f"{root}:{source}:{kind}:{fault}"] = (before, after)
    require(len(rows) == len(expected) == 96 and [row["case"] for row in rows] == list(expected), "incomplete injected matrix")
    for row in rows:
        before, after = expected[row["case"]]
        require(row == {"case": row["case"], "legacy_error": before + ": rc9 explicitly injected filesystem failure",
                        "native_error": after + ": rc9 explicitly injected filesystem failure", "native_operations": 1}
                and type(row["native_operations"]) is int, "wrong injected stage or partial-success claim")
