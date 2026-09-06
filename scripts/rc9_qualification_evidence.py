"""Validate raw qualification ledger-to-report bindings, never consumer architecture."""


def require(condition, message):
    if not condition:
        raise ValueError("qualification evidence: " + message)


def expected(assertion):
    semantics = assertion["matching_semantics"]
    kind = semantics["kind"]
    if kind == "Rust fs::read_to_string precondition":
        return dict(kind="bytes-equal", other=assertion["selection"]["paths"][0], utf8=True)
    if kind in ["whole UTF-8 literal match", "whole UTF-8 literal presence"]:
        mode = semantics["mode"]
        cardinality = assertion["required_cardinality"]
        require(mode != "absent" or cardinality == {"exact": 0}, "invalid absence quantity")
        require(mode != "contains" or cardinality == {"minimum": 1}, "invalid presence quantity")
        return dict(kind="literal", text=semantics["literal"], mode=semantics["mode"],
                    normalization=semantics["normalization"], case=semantics["case"],
                    count=cardinality["exact"] if mode == "exact-count" else None)
    if kind == "raw first-occurrence byte order":
        require(semantics["relation"] == "strictly-before", "unsupported marker relation")
        return dict(kind="literal-order", before=semantics["before"], after=semantics["after"])
    if kind == "literal within raw first-marker interval":
        require(semantics["bounds"] == "start-inclusive/end-exclusive", "unsupported interval bounds")
        return dict(kind="literal-between", start=semantics["start"], end=semantics["end"],
                    contains=semantics["literal"])
    require(kind == "trimmed line prefix value allowlist"
            and semantics["normalization"] == "Rust str::trim", "unsupported raw predicate")
    return dict(kind="line-values-allowed", prefix=semantics["prefix"],
                values=semantics["allowed"], normalization="trim")


def cases(predicate):
    kind = predicate["kind"]
    if kind == "bytes-equal":
        positive = ["valid", "empty-readable-input"]
        negative = ["invalid-utf8", "missing-input", "wrong-entry-kind"]
        diagnostic, claim = "REP-FILE-005", "utf8-file-bytes"
    elif kind == "literal":
        mode = predicate["mode"]
        positive = ["valid", "comment-wrapped-input"]
        if mode == "contains":
            positive += ["duplicate-marker"]
            negative = ["omitted-marker", "changed-case", "split-marker"]
        elif mode == "absent":
            positive += ["changed-case", "split-marker"]
            negative = ["forbidden-marker", "commented-forbidden-marker", "quoted-forbidden-marker"]
        else:
            require(mode == "exact-count", "unsupported literal mode")
            negative = ["omitted-occurrence", "extra-occurrence", "changed-case", "duplicated-input"]
        diagnostic, claim = "REP-FILE-004", "raw-utf8-text"
    elif kind == "literal-order":
        positive = ["valid", "comment-wrapped-input", "unicode-prefix", "later-duplicates"]
        negative = ["missing-before", "missing-after", "reversed-first-markers", "earlier-comment-after",
                    "later-ordered-pair-cannot-repair"]
        diagnostic, claim = "REP-FILE-008", "raw-utf8-byte-interval"
    elif kind == "literal-between":
        positive = ["valid", "comment-wrapped-input", "unicode-prefix", "repeated-interval-markers"]
        negative = ["missing-interval-marker", "marker-only-before-interval", "marker-only-after-interval",
                    "later-valid-interval-cannot-repair", "missing-start", "missing-end", "earlier-comment-end"]
        diagnostic, claim = "REP-FILE-008", "raw-utf8-byte-interval"
    else:
        require(kind == "line-values-allowed", "unsupported evidence predicate")
        positive = ["valid", "empty-unselected-suffix", "nonmatching-spacing", "commented-image",
                    "irrelevant-prefix", "unicode-padding", "duplicate-reviewed-image",
                    "alternate-reviewed-image", "crlf", "zero-selected-lines"]
        negative = ["forbidden-value", "indented-forbidden-value", "raw-block-literal", "quoted-pin",
                    "trailing-comment", "changed-case", "omitted-unauthorized-samples"]
        diagnostic, claim = "REP-FILE-008", "raw-utf8-line-values"
    outcomes = dict.fromkeys(positive, None) | dict.fromkeys(negative, diagnostic)
    if kind != "bytes-equal":
        outcomes["invalid-utf8"] = "REP-FILE-006"
    return outcomes, claim


def verify(assertion, report):
    predicate = expected(assertion)
    expected_cases, claim = cases(predicate)
    path, = assertion["selection"]["paths"]
    policy_id, = assertion["replacement"]["policy_ids"]
    require(report["predicates"] == 125 and len(report["inputs"]) == 24
            and len(report["fixtures"]) == 923, "incomplete qualification scope")
    observed, = [row for row in report["observations"] if row["policy_id"] == policy_id]
    policy = observed["policy"]
    require(policy["predicate"] == predicate and policy["include"] == [path]
            and policy["exclude"] == [] and policy["entry"] == "file", "predicate or selector mismatch")
    require(policy_id == "repository:file:" + policy["name"] == "repository:file:" + assertion["id"].lower(),
            "noncanonical assertion identity")
    require(observed["analysis"] == "exact" and observed["claim"] == claim, "overstated raw analysis claim")
    entry, = observed["entries"]
    require(entry["path"] == path and entry["sha256"] == report["inputs"][path]
            and entry["satisfied"], "unbound frozen input")
    rows = [row for row in report["fixtures"] if row["policy_id"] == policy_id]
    require(len(rows) == len(expected_cases) and {row["case"] for row in rows} == set(expected_cases),
            "missing, extra, or duplicate assertion fixtures")
    for row in rows:
        diagnostic = expected_cases[row["case"]]
        require(row["path"] == path and row["diagnostic"] == diagnostic
                and row["native_accepted"] == row["legacy_accepted"] == (diagnostic is None),
                "wrong fixture outcome or intended diagnostic")
        removed = row["case"] in ["missing-input", "wrong-entry-kind"]
        require(set(row["inputs"]) == set(report["inputs"]) - ({path} if removed else set())
                and all(p == path or digest == report["inputs"][p] for p, digest in row["inputs"].items()),
                "unbound or changed unrelated input")
        require(row["source_sha256"] == row["inputs"].get(path), "fixture byte identity mismatch")
        if row["case"] == "valid":
            require(row["inputs"] == report["inputs"], "baseline fixture differs from frozen inputs")
