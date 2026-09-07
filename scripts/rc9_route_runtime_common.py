"""Closed inputs and trusted execution parsing for the retained routing scenario."""

import hashlib
import json
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parent.parent
POLICY = "docs/rc9/policies/kafka-driver.route-runtime.fragment.toml"
PROFILE = "docs/rc9/policies/kafka-driver.route-runtime.profile.toml"
PATCH = "docs/rc9/patches/kafka-driver-explicit-array-conversion.patch"
PATCHED = "src/request/bornera_test.rs"
BEFORE = b"<[u8; 4]>::try_from(&frame[8..12])"
AFTER = b"<[u8; 4] as core::convert::TryFrom<&[u8]>>::try_from(&frame[8..12])"
NAME = "same_endpoint_generation_reuses_all_owners_then_opens_only_demanded_sibling"
FULL_NAME = "reactor::direct_plaintext::cluster_runtime::route_state::test::" + NAME
PRODUCER = "zrail-rc9-route-runner 1.0.0"
PIN = {"name": "kafka-driver", "repository": "kafkars/kafka-driver",
       "commit": "a45be8071e6a663cd4ae3142cc5304ece9fdf45e",
       "tree": "5a6dbfe18b7f8a22c6cf902803f0f1f222480b2b"}


def require(condition, message):
    if not condition:
        raise ValueError("route runtime: " + message)


def sha(payload):
    return hashlib.sha256(payload).hexdigest()


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON key")
        result[key] = value
    return result


def parse(payload):
    require(len(payload) <= 64 * 1024 * 1024, "oversized JSON")
    return json.loads(payload, object_pairs_hook=unique)


def policy():
    result, = tomllib.loads((ROOT / POLICY).read_text())["source"]["rust"]["test_mirrors"]
    require(result["name"] == NAME and len(result["inputs"]) == 824
            and result["inputs"] == sorted(set(result["inputs"])), "changed exact mirror inputs")
    return result


def input_paths(mirror):
    paths = sorted([mirror["production"], mirror["test"], *mirror["inputs"]])
    require(len(paths) == len(set(paths)) == 826, "incomplete frozen tree binding")
    return paths


def hashes(root, paths):
    result = {}
    for path in paths:
        file = root / path
        require(file.is_file() and not file.is_symlink(), "unread or linked input: " + path)
        result[path] = sha(file.read_bytes())
    return result


def run(arguments, cwd, env=None):
    return subprocess.run([str(arg) for arg in arguments], cwd=cwd, env=env,
                          capture_output=True, timeout=900)


def git(root, *args):
    result = run(["git", "-C", root, *args], ROOT)
    require(result.returncode == 0, "Git observation failed")
    return result.stdout.decode().strip()


def json_command(binary, root, action, *args):
    result = run([binary, "mirrors", action, "--root", root, "--config", "route-mirror.toml",
                  "--format", "json", *args], ROOT)
    value = parse(result.stdout or result.stderr)
    return {"exit_code": result.returncode, "output": value}


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def parse_execution(result, name=FULL_NAME):
    require(result.returncode == 0, "execution failed")
    artifacts, text = [], []
    for line in result.stdout.decode().splitlines():
        if line.startswith("{"):
            message = parse(line)
            if message.get("reason") == "compiler-artifact" and message.get("executable"):
                artifacts.append(message)
            if message.get("reason") == "build-finished":
                require(message["success"] is True, "compiler did not succeed")
        else:
            text.append(line)
    binary, = artifacts
    require(binary["target"]["name"] == "kafka_driver" and binary["target"]["kind"] == ["lib"]
            and binary["profile"]["test"] is True, "wrong compiled Cargo target")
    outcomes = re.findall(r"^test (\S+) \.\.\. (\S+)$", "\n".join(text), re.M)
    require(outcomes == [(name, "ok")], "missing, extra, ignored or unrelated executed test")
    summaries = re.findall(r"^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; "
                           r"(\d+) measured; (\d+) filtered out; finished in [0-9.]+s$",
                           "\n".join(text), re.M)
    require(len(summaries) == 1 and summaries[0][:4] == ("1", "0", "0", "0"), "wrong test counts")
    return binary, {"test": name, "passed": 1, "failed": 0, "ignored": 0, "measured": 0,
                    "filtered": int(summaries[0][4])}


def execute(arguments, root, env, log, name=FULL_NAME):
    result = run(arguments, root, env)
    log.write_bytes(result.stdout + b"\n--- compiler stderr ---\n" + result.stderr)
    binary, outcome = parse_execution(result, name)
    executable = Path(binary["executable"]).resolve(strict=True)
    require(executable.is_relative_to(Path(env["CARGO_TARGET_DIR"]).resolve()), "foreign binary")
    listing = run([executable, name, "--exact", "--list", "--format=terse"], root, env)
    require(listing.returncode == 0 and listing.stdout == f"{name}: test\n".encode(),
            "test listing is not exactly one named runnable test")
    return {"arguments": arguments, "outcome": outcome,
            "binary_sha256": sha(executable.read_bytes()), "listing_sha256": sha(listing.stdout)}
