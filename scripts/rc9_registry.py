"""Trusted frozen registry conversion, never an analyzer or lock-acceptance path."""

import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys
import tomllib

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parent.parent
REGISTRY = "crates/zrail-testkit/tests/fixtures/rc9/lock-packages/valid/guardrails.toml"
REGISTRY_SHA = "099d48347a22288d024a05260c5a102a4b142a0f1ce7a4d763e8979a83e09f10"
ARRAYS = ["banned", "core_allowed", "driver_allowed", "probe_allowed", "transport_allowed"]
PACKAGES = ["kafka_wire", "kafka_wire_core", "bornera", "bornera_core", "bornera_rustls"]
STRINGS = [package + suffix for package in PACKAGES for suffix in ["_version", "_checksum"]]
FIELDS = {"schema": "u32", "paths": {"rust_roots": ["string"]},
          "budgets": {key: "usize64" for key in ["facade", "production", "test"]},
          "dependencies": {**{key: ["string"] for key in ARRAYS}, **{key: "string" for key in STRINGS}},
          "capabilities": [{"root": "string", "forbidden": ["string"]}]}


class Rejected(ValueError):
    def __init__(self, stage, reason):
        super().__init__(reason)
        self.stage = stage


def typed(value, shape, path="registry"):
    if isinstance(shape, dict):
        if type(value) is not dict or not shape.keys() <= value.keys():
            raise Rejected("parse", path + " requires every typed field")
        return {key: typed(value[key], field, path + "." + key) for key, field in shape.items()}
    if isinstance(shape, list):
        if type(value) is not list:
            raise Rejected("parse", path + " requires an array")
        return [typed(item, shape[0], path + "[]") for item in value]
    valid = type(value) is str if shape == "string" else (
        type(value) is int and 0 <= value <= (2 ** (32 if shape == "u32" else 64) - 1))
    if not valid:
        raise Rejected("parse", path + " requires " + shape)
    return value


def parse_bytes(source):
    if len(source) > 4 * 1024 * 1024:
        raise Rejected("limit", "trusted conversion input exceeds 4 MiB")
    try:
        text = source.decode("utf-8")
    except UnicodeDecodeError as error:
        raise Rejected("read", "registry must be UTF-8") from error
    try:
        value = tomllib.loads(text)
    except tomllib.TOMLDecodeError as error:
        raise Rejected("parse", "registry must be complete TOML") from error
    bounded_integers(value)
    config = typed(value, FIELDS)
    if config["schema"] != 1:
        raise Rejected("schema", "unsupported guardrails.toml schema")
    return config


def bounded_integers(value):
    # tomllib has arbitrary precision; the original toml 0.8 parser does not.
    if type(value) is int and not -(2**63) <= value < 2**63:
        raise Rejected("parse", "TOML integer exceeds the original signed 64-bit parser range")
    if isinstance(value, dict):
        for item in value.values():
            bounded_integers(item)
    elif isinstance(value, list):
        for item in value:
            bounded_integers(item)


def load(path):
    try:
        with path.open("rb") as stream:
            source = stream.read(4 * 1024 * 1024 + 1)
    except OSError as error:
        raise Rejected("read", "registry must be readable") from error
    return parse_bytes(source)


def frozen():
    source = (ROOT / REGISTRY).read_bytes()
    if hashlib.sha256(source).hexdigest() != REGISTRY_SHA:
        raise Rejected("authority", "frozen registry changed")
    return parse_bytes(source)


def literal(value):
    if isinstance(value, dict):
        return "{ " + ", ".join(json.dumps(key) + " = " + literal(item) for key, item in value.items()) + " }"
    if isinstance(value, list):
        return "[" + ", ".join(literal(item) for item in value) + "]"
    return json.dumps(value, ensure_ascii=False)


def document(value):
    return "\n".join(json.dumps(key) + " = " + literal(item) for key, item in value.items()) + "\n"


def convert(config):
    # Unknown legacy fields are ignored, but every interpreted value is fixed.
    # This is a review-only carrier, not a complete consumer contract.
    config = typed(config, FIELDS)
    if config["schema"] != 1:
        raise Rejected("schema", "unsupported guardrails.toml schema")
    if config != frozen():
        raise Rejected("authority", "changed interpreted registry values require separate review")
    from rc9_registry_policy import carrier
    return carrier(copy.deepcopy(config))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("registry", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    policy, _ = convert(load(args.registry))
    with args.output.open("x", encoding="utf-8", newline="\n") as output:
        output.write("# REVIEW-ONLY SCHEMA CARRIER. Not a qualified downstream contract.\n"
                     "# EXCLUDES kd-source-traversal: known contract-validation blocker.\n" + document(policy))
    print("registry conversion: unchanged reviewed values; no lock or downstream mutation")


if __name__ == "__main__":
    main()
