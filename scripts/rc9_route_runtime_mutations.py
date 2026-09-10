"""Stock CLI adversarial qualification on one test-owned patched checkout."""

import copy
import json

from rc9_route_runtime_common import FULL_NAME, json_command, require, sha


def receipt_cases(receipt):
    cases = [("receipt-missing", None, "RECEIPT-001"),
             ("receipt-malformed", b"{", "RECEIPT-002")]
    changes = [("schema", 1, "RECEIPT-002"), ("producer", "unversioned", "RECEIPT-002"),
               ("input_sha256", "0" * 64, "RECEIPT-003"),
               ("tests", [{"id": "unrelated", "status": "passed"}], "RECEIPT-004"),
               ("tests", [], "RECEIPT-002")]
    for index, (key, value, diagnostic) in enumerate(changes):
        changed = receipt | {key: value}
        cases.append((f"receipt-field-{index}", changed, diagnostic))
    cases.append(("receipt-unknown-key", receipt | {"unreviewed": True}, "RECEIPT-002"))
    cases.append(("receipt-duplicate-test", receipt | {"tests": receipt["tests"] * 2}, "RECEIPT-002"))
    for status in ["failed", "skipped"]:
        changed = copy.deepcopy(receipt)
        changed["tests"][0]["status"] = status
        cases.append(("receipt-" + status, changed, "RECEIPT-005"))
    for key, value in [("command", "cargo test unrelated"), ("package", "other"),
                       ("default_features", False), ("features", []),
                       ("target", "x86_64-unknown-linux-gnu"), ("toolchain", "rustc other")]:
        changed = copy.deepcopy(receipt)
        changed["execution"][key] = value
        cases.append(("receipt-execution-" + key, changed, "RECEIPT-007"))
    return [(name, json.dumps(value, indent=2).encode() + b"\n" if isinstance(value, dict) else value, diagnostic)
            for name, value, diagnostic in cases]


def result_row(name, path, payload, result):
    output = result["output"]
    return {"case": name, "path": path, "input_sha256": None if payload is None else sha(payload),
            "exit_code": result["exit_code"], "error": output.get("error"),
            "diagnostics": sorted(f["id"] for f in output.get("report", {}).get("findings", []))}


def run_mutations(binary, root, mirror, receipt):
    rows = []

    def observe(name, path, payload, diagnostic=None):
        file = root / path
        original = file.read_bytes()
        try:
            if payload is None:
                file.unlink()
            else:
                file.write_bytes(payload)
            result = json_command(binary, root, "verify", "--plan", "route-plan.json")
            row = result_row(name, path, payload, result)
            if diagnostic:
                require(row["exit_code"] == 1 and row["diagnostics"] == [diagnostic]
                        and row["error"] is None, name + ": unintended receipt diagnostic: " + str(row))
            else:
                require(row["exit_code"] == 2 and row["error"] is not None
                        and not row["diagnostics"], name + ": stale plan accepted")
            rows.append(row)
        finally:
            file.write_bytes(original)
        print("qualified " + name, flush=True)

    for name, payload, diagnostic in receipt_cases(receipt):
        observe(name, mirror["receipt"], payload, diagnostic)
    for path in [mirror["production"], mirror["test"],
                 "src/reactor/direct_plaintext/cluster_runtime/route_test_support.rs", "Cargo.toml", "Cargo.lock", "README.md"]:
        suffix = b"\n// reviewed input drift\n" if path.endswith(".rs") else b"\n# reviewed input drift\n"
        observe("input-drift:" + path, path, (root / path).read_bytes() + suffix)
    original_test = (root / mirror["test"]).read_bytes()
    observe("test-renamed", mirror["test"], original_test.replace(mirror["name"].encode(), b"unrelated"))
    observe("test-ignored", mirror["test"], original_test.replace(
        b"#[test]\nfn " + mirror["name"].encode(), b"#[test]\n#[ignore]\nfn " + mirror["name"].encode()))
    config = (root / "route-mirror.toml").read_bytes()
    for key, before, after in [("command", FULL_NAME, "unrelated"),
                               ("features", 'features = ["tls-rustls"]', 'features = []'),
                               ("target", 'target = "aarch64-apple-darwin"', 'target = "x86_64-unknown-linux-gnu"'),
                               ("toolchain", 'toolchain = "rustc 1.97.1 (8bab26f4f 2026-07-14)"', 'toolchain = "rustc other"')]:
        require(config.count(before.encode()) == 1, "ambiguous execution mutation")
        observe("contract-execution-" + key, "route-mirror.toml", config.replace(before.encode(), after.encode()))
    original_plan = (root / "route-plan.json").read_bytes()
    plan = json.loads(original_plan)
    plan["plan_sha256"] = "0" * 64
    observe("plan-digest", "route-plan.json", (json.dumps(plan, indent=2) + "\n").encode())
    require(len(rows) == 30, "incomplete runtime mutation matrix")
    return rows
