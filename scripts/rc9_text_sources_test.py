"""Adversarial source-span binding tests, separate from source policy evaluation."""

import copy
import hashlib
from pathlib import Path
import runpy
import tempfile
import unittest


MODULE = runpy.run_path(Path(__file__).with_name("rc9_text_sources.py"))
LOAD, PREFIX = MODULE["load"], MODULE["PREFIX"]


class AuditedTextSources(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.payload = b"#!/bin/sh\n# UTF-8: \xc3\xa9\nif [ -z \"$input\" ]; then\n  exit 2\nfi\n"
        self.path = self.root / PREFIX / "fixture/check"
        self.path.parent.mkdir(parents=True)
        self.path.write_bytes(self.payload)
        start = self.payload.index(b"if ")
        end = len(self.payload)
        self.row = {
            "id": "example/repo:scripts/check:3:1:audited-text", "repository": "example/repo",
            "commit": "a" * 40, "path": "scripts/check", "copy": PREFIX + "fixture/check",
            "file_sha256": hashlib.sha256(self.payload).hexdigest(), "kind": "audited-text",
            "language": "shell", "line": 3, "column": 1, "end_line": 6, "end_column": 1,
            "start_byte": start, "end_byte": end, "function": ["top-level"],
            "span_sha256": hashlib.sha256(self.payload[start:end]).hexdigest(),
        }
        self.files = {("example/repo", "scripts/check"):
                      {"commit": "a" * 40, "sha256": self.row["file_sha256"]}}

    def verify(self, rows):
        return LOAD({"audited_text_sources": rows}, self.files, self.root)

    def test_exact_physical_span_and_default_empty_family(self):
        self.assertEqual(list(self.verify([self.row])), [self.row["id"]])
        self.assertEqual(LOAD({}, self.files, self.root), {})

    def test_unbound_identity_scope_bytes_and_positions_fail(self):
        mutations = {
            "kind": "rust-census", "language": "executable-plugin", "repository": "other/repo",
            "commit": "b" * 40, "path": "scripts/other", "file_sha256": "0" * 64,
            "copy": PREFIX + "../outside", "id": "forged", "line": 4, "column": 2,
            "end_line": 5, "end_column": 2, "start_byte": -1, "end_byte": 99999,
            "span_sha256": "0" * 64, "function": [],
        }
        for key, value in mutations.items():
            row = copy.deepcopy(self.row)
            row[key] = value
            with self.subTest(field=key), self.assertRaises(ValueError):
                self.verify([row])
        for rows in [[self.row, self.row], [dict(self.row, evaluator="shell")], [dict(self.row, start_byte=True)]]:
            with self.assertRaises(ValueError):
                self.verify(rows)
        for key in ["repository", "copy", "language", "span_sha256"]:
            with self.subTest(field=key), self.assertRaises(ValueError):
                self.verify([dict(self.row, **{key: []})])

    def test_mutated_or_escaping_copy_cannot_supply_frozen_authority(self):
        self.path.write_bytes(self.payload + b"# extra\n")
        with self.assertRaises(ValueError):
            self.verify([self.row])
        self.path.unlink()
        outside = self.root / "outside"
        outside.write_bytes(self.payload)
        self.path.symlink_to(outside)
        with self.assertRaises(ValueError):
            self.verify([self.row])

    def test_offsets_must_end_on_utf8_boundaries(self):
        row = copy.deepcopy(self.row)
        row["start_byte"] = self.payload.index(b"\xa9")
        row["line"], row["column"] = 2, 11
        row["id"] = "example/repo:scripts/check:2:11:audited-text"
        with self.assertRaises(ValueError):
            self.verify([row])

    def test_span_file_and_total_work_limits_fail_closed(self):
        with self.assertRaises(ValueError):
            self.verify([self.row] * 4097)
        for size in [16385, MODULE["MAX_FILE_BYTES"] + 1]:
            payload = b"x" * size
            self.path.write_bytes(payload)
            digest = hashlib.sha256(payload).hexdigest()
            row = dict(self.row, file_sha256=digest, start_byte=0, end_byte=size,
                       span_sha256=digest, line=1, column=1, end_line=1, end_column=size + 1,
                       id="example/repo:scripts/check:1:1:audited-text")
            self.files[("example/repo", "scripts/check")]["sha256"] = digest
            with self.assertRaises(ValueError):
                self.verify([row])
        self.path.write_bytes(self.payload)
        self.files[("example/repo", "scripts/check")]["sha256"] = self.row["file_sha256"]
        budget = LOAD.__globals__["MAX_INPUT_BYTES"]
        try:
            LOAD.__globals__["MAX_INPUT_BYTES"] = len(self.payload) - 1
            with self.assertRaises(ValueError):
                self.verify([self.row])
        finally:
            LOAD.__globals__["MAX_INPUT_BYTES"] = budget


if __name__ == "__main__":
    unittest.main()
