"""Bind every interpreted registry field to existing policy in a schema-test carrier."""

import copy
import hashlib
import runpy
import tomllib

from rc9_registry import ROOT, PACKAGES, Rejected

BASE = "crates/zrail-testkit/tests/fixtures/good/zrail.toml"
FRAGMENTS = ["sizes", "files", "key-sets", "raw-dependency", "dependency-fields", "provenance", "lock-packages", "traversal"]


def merge(target, incoming):
    for key, value in incoming.items():
        if key not in target:
            target[key] = copy.deepcopy(value)
        elif isinstance(value, dict) and isinstance(target[key], dict):
            merge(target[key], value)
        elif isinstance(value, list) and isinstance(target[key], list):
            target[key].extend(copy.deepcopy(value))
        elif target[key] != value:
            raise Rejected("authority", "conflicting carrier section " + key)


def carrier(config):
    result = tomllib.loads((ROOT / BASE).read_text())
    inputs = {BASE: hashlib.sha256((ROOT / BASE).read_bytes()).hexdigest()}
    fragments = {}
    for name in FRAGMENTS:
        path = "docs/rc9/policies/kafka-driver." + name + ".fragment.toml"
        source = (ROOT / path).read_bytes()
        inputs[path] = hashlib.sha256(source).hexdigest()
        fragments[name] = tomllib.loads(source.decode())
        selected = copy.deepcopy(fragments[name])
        if name == "traversal":
            # The unchanged count(minimum=0) rule fails current contract loading.
            # Retain its regression separately; never call this a full bundle.
            blocked, = [row for row in selected["repository"]["files"] if row["name"] == "kd-source-traversal"]
            if blocked["entry"] != "any" or blocked["predicate"] != {"kind": "count", "minimum": 0}:
                raise Rejected("authority", "tracked traversal blocker changed; review the carrier exclusion")
            selected["repository"]["files"] = [row for row in selected["repository"]["files"]
                                                if row["name"] != "kd-source-traversal"]
        merge(result, selected)
    bindings = {}

    def bind(key, actual, expected):
        if actual != expected:
            raise Rejected("authority", "registry/policy binding differs: " + key)
        bindings[key] = copy.deepcopy(actual)

    bind("schema", result["schema"], config["schema"])
    roots = config["paths"]["rust_roots"]
    traversal = fragments["traversal"]["repository"]["files"]
    bind("paths.rust_roots", traversal[0]["predicate"]["paths"], roots)
    bind("paths.rust_roots.include", traversal[0]["include"], roots)
    bind("paths.rust_roots.traversal", traversal[1]["include"], [root + "/**" for root in roots])
    size = runpy.run_path(ROOT / "scripts/rc9-size-policies")["kafka_driver"](config)
    expected = tomllib.loads("\n".join(size))["source"]["rust"]["budgets"]["overrides"]
    bind("budgets.and_source_selection", fragments["sizes"]["source"]["rust"]["budgets"]["overrides"], expected)
    files = runpy.run_path(ROOT / "scripts/rc9-file-policies")["translate"](config)
    bind("capabilities.and_source_selection", fragments["files"]["repository"]["files"],
         tomllib.loads("\n".join(files))["repository"]["files"])
    keys = {row["name"]: row for row in fragments["key-sets"]["repository"]["files"]}
    for name in ["core", "driver", "probe", "transport"]:
        bind("dependencies." + name + "_allowed", keys["kd-dep-" + name + "-keys"]["predicate"]["assertion"]["keys"],
             sorted(config["dependencies"][name + "_allowed"]))
    bans = [row["predicate"]["text"] for row in fragments["raw-dependency"]["repository"]["files"] if row["name"].startswith("kd-lock-ban-")]
    bind("dependencies.banned", bans, ['name = "' + name + '"' for name in config["dependencies"]["banned"]])
    locks = {row["package"]: row for row in fragments["lock-packages"]["dependencies"]["lock_package"]}
    for package in PACKAGES:
        identity, = locks[package.replace("_", "-")]["assertion"]["identities"]
        version = config["dependencies"][package + "_version"]
        if not version.startswith("="):
            raise Rejected("authority", "exact prefix required")
        bind("dependencies." + package + "_version", "=" + identity["version"], version)
        bind("dependencies." + package + "_checksum", identity["checksum"], config["dependencies"][package + "_checksum"])
    # Other authored version predicates remain in the carrier unchanged. Their
    # existing independent evidence remains responsible for their full semantics.
    names = [row["name"] for row in result["repository"]["files"]]
    if len(names) != len(set(names)):
        raise Rejected("authority", "duplicate carrier file policy")
    return result, {"inputs": inputs, "bindings": bindings, "full_repository_qualified": False,
                    "excluded_from_schema_carrier": ["kd-source-traversal"],
                    "open_blocker": "RC9-NATIVE-TRAVERSAL-CONTRACT"}
