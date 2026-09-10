"""Check route parity artifact bindings; no consumer evaluator or compiler executes here."""

import hashlib
from pathlib import Path
import tomllib


ROOT = Path(__file__).resolve().parent.parent
DATA = "crates/zrail-testkit/tests/fixtures/rc9"
FACADE = "src/reactor/direct_plaintext/cluster_runtime.rs"
SOURCE = "src/reactor/direct_plaintext/cluster_runtime/route_state_test.rs"
TEST = "route_modules_cannot_own_a_selector_or_legacy_routing_capability"
TOKENS = ["ConnectionSet", "Poller", "SingleBroker", "BrokerSet"]
MARKER = b"connections: DirectSetOwner<T>"
PREFIX = "repository:file:kd-route-"


def require(condition, message):
    if not condition:
        raise ValueError("route evidence: " + message)


def sha(payload):
    return hashlib.sha256(payload).hexdigest()


def identity(payload):
    return None if payload is None else {"sha256": sha(payload), "bytes": len(payload)}


def cases(baseline):
    result = [("valid", baseline, {}, [], None)]
    for path, original in sorted(baseline.items()):
        if path == FACADE:
            continue
        for token in TOKENS:
            for context, addition in [
                ("comment", f"\n// {token}\n"),
                ("string", f'\nconst TEXT: &str = "{token}";\n'),
                ("rename", f"\nuse crate::{token} as Renamed;\n"),
                ("lowercase", f"\n// {token.lower()}\n"),
            ]:
                findings = [] if context == "lowercase" else [[PREFIX + "forbid-" + token, "REP-FILE-004"]]
                result.append((f"{context}:{path}:{token}", baseline | {path: original + addition.encode()},
                               {}, findings, None))
    result.append(("unrelated-tokens", baseline, {"unrelated.rs": " ".join(TOKENS).encode()}, [], None))
    original = baseline[FACADE]
    removed = original.replace(MARKER, b"connections: Other<T>")
    for case, payload in [
        ("owner-missing", removed), ("owner-duplicate", original + b"\n// " + MARKER + b"\n"),
        ("owner-spelling", original.replace(MARKER, b"connections : DirectSetOwner<T>")),
        ("owner-wrong-file", removed), ("owner-comment", removed + b"\n// " + MARKER + b"\n"),
    ]:
        findings = [] if case == "owner-comment" else [[PREFIX + "owner-field", "REP-FILE-004"]]
        result.append((case, baseline | {FACADE: payload}, {"unrelated.rs": MARKER}, findings, None))
    for path in sorted(baseline):
        findings = [[PREFIX + "inputs", "REP-FILE-002"]]
        if path == FACADE:
            findings.append([PREFIX + "owner-field", "REP-FILE-004"])
        result.append(("input-missing:" + path, baseline | {path: None}, {}, findings, None))
        result.append(("input-utf8:" + path, baseline | {path: b"\xff"}, {}, [], path))
    require(len(result) == 189, "incomplete closed route mutation matrix")
    return result


def policies():
    path = ROOT / "docs/rc9/policies/kafka-driver.route-sources.fragment.toml"
    rows = tomllib.loads(path.read_text())["repository"]["files"]
    result = {}
    for row in rows:
        row.setdefault("entry", "file")
        row.setdefault("exclude", [])
        if row["predicate"]["kind"] == "literal":
            for key, value in {"count": None, "normalization": "none", "case": "sensitive"}.items():
                row["predicate"].setdefault(key, value)
        result[PREFIX.removesuffix("kd-route-") + row["name"]] = row
    require(len(result) == 6, "changed route policy inventory")
    return result, sha(path.read_bytes())


def offsets(payload, literal):
    result, start = [], 0
    while (found := payload.find(literal, start)) >= 0:
        result.append(found)
        start = found + len(literal)
    return result


def policy_contract(selected, baseline):
    required = {PREFIX + "inputs", PREFIX + "owner-field"} | {PREFIX + "forbid-" + token for token in TOKENS}
    require(set(selected) == required, "missing or extra route policy")
    for id_, policy in selected.items():
        require(policy["exclude"] == [] and policy["entry"] == "file" and policy["reason"].strip(),
                "weakened input scope or missing rationale")
        if id_ == PREFIX + "inputs":
            paths = sorted(baseline)
            predicate = {"kind": "exact-paths", "paths": paths}
        else:
            owner = id_ == PREFIX + "owner-field"
            paths = [FACADE] if owner else sorted(set(baseline) - {FACADE})
            predicate = {"kind": "literal", "text": MARKER.decode() if owner else id_.removeprefix(PREFIX + "forbid-"),
                         "mode": "exact-count" if owner else "absent", "count": 1 if owner else None,
                         "case": "sensitive", "normalization": "none"}
        require(policy["include"] == paths and policy["predicate"] == predicate,
                "translation differs from the exact frozen assertion")


def observation(observed, policy_id, policy, payloads):
    require(observed["policy_id"] == policy_id and observed["policy"] == policy,
            "policy identity, selector, or predicate changed")
    raw = policy["predicate"]["kind"] == "literal"
    require(observed["analysis"] == "exact" and observed["claim"] == ("raw-utf8-text" if raw else "physical-paths")
            and observed["reference"] is None and observed["checkout_path"] is None,
            "overstated claim or foreign input")
    paths = sorted(path for path in policy["include"] if payloads[path] is not None)
    require([entry["path"] for entry in observed["entries"]] == paths,
            "missing, duplicate, extra, or unordered physical occurrence")
    all_satisfied = True
    for entry in observed["entries"]:
        payload = payloads[entry["path"]]
        require(entry["kind"] == "file" and entry["resolved_path"] is None
                and entry["forbidden_names"] == [], "incorrect physical entry")
        if raw:
            hits = offsets(payload, policy["predicate"]["text"].encode())
            satisfied = len(hits) == (policy["predicate"]["count"] or 0)
            require(all(type(entry[key]) is int for key in ["bytes", "literal_count", "omitted_literal_offsets"])
                    and all(type(offset) is int for offset in entry["literal_offsets"])
                    and entry["sha256"] == sha(payload) and entry["bytes"] == len(payload)
                    and entry["literal_count"] == len(hits) and entry["literal_offsets"] == hits[:16]
                    and entry["omitted_literal_offsets"] == max(0, len(hits) - 16),
                    "unbound file bytes or wrong complete quantity")
        else:
            satisfied = True
            require(entry["sha256"] is None and entry["bytes"] is None and entry["literal_count"] is None
                    and entry["literal_offsets"] == [] and entry["omitted_literal_offsets"] == 0,
                    "path-only policy fabricated a content observation")
        require(entry["satisfied"] is satisfied, "wrong per-file predicate result")
        all_satisfied &= satisfied
    if raw:
        satisfied = all_satisfied and (policy["predicate"]["mode"] == "absent" or bool(paths))
    else:
        satisfied = paths == sorted(policy["predicate"]["paths"])
    require(observed["satisfied"] is satisfied, "wrong complete predicate result")


def compiler(record, case, baseline_compiler, harness):
    require(set(record) == {"executable", "executable_sha256", "arguments", "exit_code", "stderr_sha256", "errors", "execution"},
            "changed compiler receipt schema")
    require(type(record["exit_code"]) is int, "compiler status must be an integer")
    require(record["executable"] == baseline_compiler["executable"]
            and record["executable_sha256"] == baseline_compiler["executable_sha256"]
            and record["arguments"] == ["--edition=2024", "--test", "--error-format=json", "--color=never",
                                        SOURCE, "-o", "route-legacy-tests"], "changed compiler or invocation")
    for key in ["executable_sha256", "stderr_sha256"]:
        require(isinstance(record[key], str) and len(record[key]) == 64
                and set(record[key]) <= set("0123456789abcdef"), "invalid compiler hash")
    if case == "valid":
        require(record["exit_code"] == 0 and record["errors"] == [] and record["stderr_sha256"] == sha(b""),
                "original baseline did not compile cleanly")
        run = record["execution"]
        require(set(run) == {"test", "passed", "failed", "ignored", "filtered", "binary_sha256", "list_sha256"}
                and run["test"] == TEST and type(run["passed"]) is int and run["passed"] == 1
                and all(type(run[k]) is int and run[k] == 0 for k in ["failed", "ignored", "filtered"])
                and run["list_sha256"] == sha(f"{TEST}: test\n".encode()),
                "missing, extra, duplicate, ignored or filtered original outcome")
        require(len(run["binary_sha256"]) == 64, "unbound compiled guard")
        return
    require(record["exit_code"] == 1 and record["execution"] is None and len(record["errors"]) == 1,
            "unrelated compilation or fabricated execution")
    kind, path = case.split(":", 1)
    literal = "../cluster_runtime.rs" if path == FACADE else Path(path).name
    error, = record["errors"]
    require(error["level"] == "error" and error["code"] is None and not error["children"],
            "unrelated compiler diagnostic")
    span, = [span for span in error["spans"] if span["is_primary"] is True]
    if kind == "input-utf8":
        require(error["message"] == f"`{literal}` wasn't a utf-8 file" and span["file_name"] == literal
                and span["byte_start"] == span["byte_end"] == 0
                and span["label"] == "byte `255` is not valid utf-8", "unrelated UTF-8 compile failure")
    else:
        source = f'include_str!("{literal}")'.encode()
        start = harness.index(source)
        require(kind == "input-missing" and "couldn't read" in error["message"] and literal in error["message"]
                and span["file_name"] == SOURCE and span["byte_start"] == start
                and span["byte_end"] == start + len(source), "unrelated missing-input compile failure")


def verify(assertion, report, files):
    import json
    require(report["schema"] == 1 and not report["full_repository_qualified"], "overstated report")
    require(report["snapshot"]["repository"] == assertion["repository"]
            and report["snapshot"]["commit"] == assertion["commit"], "mixed source snapshots")
    toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    require(report["rustc_version"].startswith(f"rustc {toolchain} ")
            and report["cargo_lock_sha256"] == sha((ROOT / "Cargo.lock").read_bytes()),
            "changed compiler or build inputs")
    origins_path = ROOT / DATA / "route-sources-origins.json"
    origins = json.loads(origins_path.read_bytes())
    copied = {}
    for row in origins["fixtures"]:
        payload = (ROOT / row["copy"]).read_bytes()
        frozen = files[(assertion["repository"], row["path"])]
        require(sha(payload) == row["sha256"] == frozen["sha256"] and frozen["commit"] == assertion["commit"],
                "mixed snapshots or unbound source bytes")
        copied[row["path"]] = payload
    source = copied.pop(SOURCE)
    harness = source[source.index(f"#[test]\nfn {TEST}()".encode()):]
    require(len(copied) == 11 and report["input_hashes"] == {p: identity(v) for p, v in copied.items()}
            and report["source_test_sha256"] == sha(source) and report["harness_sha256"] == sha(harness)
            and report["fixture_origins_sha256"] == sha(origins_path.read_bytes()), "unbound compiler inputs")
    selected, digest = policies()
    policy_contract(selected, copied)
    require(report["policy_sha256"] == digest, "unbound policy bytes")
    id_ = assertion["id"]
    if id_.startswith("KD-ROUTE-INPUT-"):
        policy = PREFIX + "inputs"
        diagnostics = ["REP-FILE-002", "REP-FILE-006"]
        require(assertion["selection"]["paths"][0] in copied, "unqualified include input")
    elif id_.startswith("KD-ROUTE-FORBID-"):
        token = id_.removeprefix("KD-ROUTE-FORBID-")
        require(token in TOKENS, "unknown forbidden subject")
        policy, diagnostics = PREFIX + "forbid-" + token, ["REP-FILE-004"]
    else:
        require(id_ == "KD-ROUTE-OWNER-FIELD", "runtime assertions require their own evidence")
        policy, diagnostics = PREFIX + "owner-field", ["REP-FILE-004"]
    require(assertion["replacement"]["policy_ids"] == [policy]
            and assertion["replacement"]["expected_diagnostics"] == diagnostics, "wrong assertion linkage")
    matrix = cases(copied)
    rows = report["fixtures"]
    require([row["case"] for row in rows] == [case[0] for case in matrix], "missing, extra or duplicate route case")
    baseline_compiler = rows[0]["compiler"]
    require(Path(baseline_compiler["executable"]).is_absolute()
            and Path(baseline_compiler["executable"]).name == "rustc"
            and Path(baseline_compiler["executable"]).parent.parent.name.startswith(toolchain + "-"),
            "compiler invocation did not select the project toolchain")
    for row, (case, payloads, extra, findings, bad_utf8) in zip(rows, matrix):
        require(row["inputs"] == {path: identity(payload) for path, payload in payloads.items()}
                and row["extra_inputs"] == {path: identity(payload) for path, payload in extra.items()},
                "unbound fixture mutation or decoy")
        accepted = not findings and bad_utf8 is None
        require(row["legacy_accepted"] is accepted and row["native_accepted"] is accepted
                and row["findings"] == findings, "protection difference or unrelated native diagnostic")
        if bad_utf8:
            require(row["observations"] == [] and row["error"].startswith("REP-FILE-006:")
                    and bad_utf8 in row["error"] and "not UTF-8" in row["error"], "unrelated incomplete analysis")
        else:
            require(row["error"] is None and [r["policy_id"] for r in row["observations"]] == sorted(selected),
                    "incomplete policy coverage")
            for observed in row["observations"]:
                observation(observed, observed["policy_id"], selected[observed["policy_id"]], payloads)
            require([[r["policy_id"], "REP-FILE-002" if r["policy_id"] == PREFIX + "inputs" else "REP-FILE-004"]
                     for r in row["observations"] if not r["satisfied"]] == findings,
                    "findings disagree with evaluated policies")
        has_compiler = case == "valid" or case.startswith("input-")
        require(("compiler" in row) == has_compiler, "missing or unrelated original compilation receipt")
        if has_compiler:
            compiler(row["compiler"], case, baseline_compiler, harness)
