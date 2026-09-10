"""Bind manually reviewed non-Rust assertion spans; never evaluate their contents."""

import hashlib
from pathlib import Path, PurePosixPath


ROOT = Path(__file__).resolve().parent.parent
PREFIX = "crates/zrail-testkit/tests/fixtures/rc9/audited-text/"
FIELDS = {"id", "repository", "commit", "path", "file_sha256", "copy", "kind", "language",
          "line", "column", "end_line", "end_column", "start_byte", "end_byte", "function",
          "span_sha256"}
MAX_FILE_BYTES = 2 * 1024 * 1024
MAX_INPUT_BYTES = 64 * 1024 * 1024


def require(condition, message):
    if not condition:
        raise ValueError("audited text source: " + message)


def location(payload, offset):
    before = payload[:offset]
    before.decode("utf-8")
    return before.count(b"\n") + 1, offset - before.rfind(b"\n")


def load(ledger, files, root=ROOT):
    rows = ledger.get("audited_text_sources", [])
    require(isinstance(rows, list) and len(rows) <= 4096, "expected at most 4096 spans")
    result, cache = {}, {}
    total = 0
    for row in rows:
        require(isinstance(row, dict) and set(row) == FIELDS, "unsupported span fields")
        strings = ["id", "repository", "commit", "path", "file_sha256", "copy", "kind",
                   "language", "span_sha256"]
        require(all(isinstance(row[key], str) for key in strings), "source identities require strings")
        require(row["kind"] == "audited-text"
                and row["language"] in {"shell", "python", "markdown", "yaml", "toml", "json"},
                "unsupported manual source kind or language")
        identity = f"{row['repository']}:{row['path']}:{row['line']}:{row['column']}:audited-text"
        require(row["id"] == identity and identity not in result, "noncanonical or duplicate identity")
        frozen = files.get((row["repository"], row["path"]))
        require(frozen is not None and frozen["commit"] == row["commit"]
                and frozen["sha256"] == row["file_sha256"], "source differs from frozen census")
        copy = row["copy"]
        require(isinstance(copy, str) and copy.startswith(PREFIX) and "\\" not in copy
                and PurePosixPath(copy).as_posix() == copy and ".." not in PurePosixPath(copy).parts,
                "copy is outside the canonical audited fixture directory")
        path = root / copy
        require(not path.is_symlink() and path.resolve().is_relative_to((root / PREFIX).resolve()),
                "copy escapes the fixture directory")
        if copy not in cache:
            with path.open("rb") as stream:
                payload = stream.read(MAX_FILE_BYTES + 1)
            require(len(payload) <= MAX_FILE_BYTES, "source copy exceeds 2 MiB")
            total += len(payload)
            require(total <= MAX_INPUT_BYTES, "source copies exceed 64 MiB")
            payload.decode("utf-8")
            cache[copy] = payload
        payload = cache[copy]
        require(hashlib.sha256(payload).hexdigest() == row["file_sha256"], "copy bytes are not frozen")
        numeric = ["line", "column", "end_line", "end_column", "start_byte", "end_byte"]
        require(all(type(row[key]) is int for key in numeric), "positions require integers")
        start, end = row["start_byte"], row["end_byte"]
        require(0 <= start < end <= len(payload) and end - start <= 16384, "invalid or oversized span")
        require(location(payload, start) == (row["line"], row["column"])
                and location(payload, end) == (row["end_line"], row["end_column"]),
                "byte offsets and physical positions differ")
        require(hashlib.sha256(payload[start:end]).hexdigest() == row["span_sha256"],
                "selected assertion bytes changed")
        require(isinstance(row["function"], list) and 1 <= len(row["function"]) <= 32
                and all(isinstance(name, str) and name.strip() == name and 0 < len(name) <= 128
                        for name in row["function"]), "invalid manual context label")
        result[identity] = (frozen, row)
    return result
