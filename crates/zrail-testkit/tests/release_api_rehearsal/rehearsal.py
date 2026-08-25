#!/usr/bin/env python3
"""Executable mocked GitHub API rehearsal for the release-state helper."""

from __future__ import annotations

from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
from urllib.parse import parse_qs, urlparse


COMMIT = "a" * 40
TAG_OBJECT = "b" * 40
OTHER_COMMIT = "c" * 40
RELEASE_ID = 41


class State:
    def __init__(self, version):
        self.version = version
        self.tag = f"v{version}"
        self.prerelease = "-" in version
        self.base_url = ""
        self.release = None
        self.assets = {}
        self.next_asset_id = 100
        self.fail_second_upload_with_starter_once = False
        self.failed_upload = False
        self.starter_asset_id = None
        self.tag_object = TAG_OBJECT
        self.tag_commit = COMMIT
        self.graphql_results = []
        self.created = 0
        self.release_get_ids = []
        self.asset_downloads = []
        self.asset_delete_attempts = []
        self.asset_deletions = []
        self.upload_attempts = []
        self.uploads = []
        self.events = []
        self.patches = 0
        self.fail_publish_response_once = False
        self.failed_publish_response = False

    def release_json(self):
        if self.release is None:
            return None
        result = dict(self.release)
        result["upload_url"] = (
            f"{self.base_url}/uploads/repos/acme/zrail/releases/{RELEASE_ID}/assets"
            "{?name,label}"
        )
        result["assets"] = [
            {
                "id": asset.get("reported_id", asset_id),
                "name": asset["name"],
                "state": asset["state"],
                "url": (
                    f"{self.base_url}/api/repos/acme/zrail/releases/assets/{asset_id}"
                ),
            }
            for asset_id, asset in sorted(self.assets.items())
        ]
        return result


class Handler(BaseHTTPRequestHandler):
    server_version = "ReleaseRehearsal/1"

    @property
    def state(self):
        return self.server.state

    def log_message(self, _format, *_args):
        pass

    def read_body(self):
        length = int(self.headers.get("Content-Length", "0"))
        return self.rfile.read(length)

    def send_json(self, status, value):
        body = json.dumps(value).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def send_bytes(self, status, value):
        self.send_response(status)
        self.send_header("Content-Type", "application/octet-stream")
        self.send_header("Content-Length", str(len(value)))
        self.end_headers()
        self.wfile.write(value)

    def do_POST(self):
        parsed = urlparse(self.path)
        if parsed.path == "/graphql":
            self.graphql(parsed)
        elif parsed.path == "/api/repos/acme/zrail/releases":
            self.create_release()
        elif parsed.path == f"/uploads/repos/acme/zrail/releases/{RELEASE_ID}/assets":
            self.upload_asset(parsed)
        else:
            self.send_error(404)

    def graphql(self, _parsed):
        payload = json.loads(self.read_body())
        assert "release(tagName:$tag){databaseId}" in payload["query"]
        assert payload["variables"] == {
            "owner": "acme",
            "name": "zrail",
            "tag": self.state.tag,
        }
        release = None if self.state.release is None else {"databaseId": RELEASE_ID}
        self.state.graphql_results.append(None if release is None else RELEASE_ID)
        self.send_json(200, {"data": {"repository": {"release": release}}})

    def create_release(self):
        payload = json.loads(self.read_body())
        assert self.state.release is None
        assert payload == {
            "tag_name": self.state.tag,
            "name": f"zrail {self.state.version}",
            "body": "Reviewed notes.\n",
            "draft": True,
            "prerelease": self.state.prerelease,
        }
        self.state.created += 1
        self.state.release = {
            "id": RELEASE_ID,
            "tag_name": payload["tag_name"],
            "name": payload["name"],
            "body": payload["body"],
            "draft": True,
            "prerelease": self.state.prerelease,
            "target_commitish": "intentionally-not-authoritative",
        }
        self.send_json(201, self.state.release_json())

    def upload_asset(self, parsed):
        name = parse_qs(parsed.query).get("name", [None])[0]
        assert name in {"a.bin", "b.bin"}
        content = self.read_body()
        self.state.events.append(f"upload:{name}")
        self.state.upload_attempts.append((name, content))
        if any(asset["name"] == name for asset in self.state.assets.values()):
            self.send_json(422, {"message": "asset name already exists"})
            return
        if (
            self.state.fail_second_upload_with_starter_once
            and name == "b.bin"
            and not self.state.failed_upload
        ):
            self.state.failed_upload = True
            asset_id = self.state.next_asset_id
            self.state.next_asset_id += 1
            self.state.starter_asset_id = asset_id
            self.state.assets[asset_id] = {
                "name": name,
                "content": b"",
                "state": "starter",
            }
            self.send_json(502, {"message": "injected ambiguous upload failure"})
            return
        asset_id = self.state.next_asset_id
        self.state.next_asset_id += 1
        self.state.assets[asset_id] = {
            "name": name,
            "content": content,
            "state": "uploaded",
        }
        self.state.uploads.append(name)
        self.send_json(201, {"id": asset_id, "name": name})

    def do_GET(self):
        path = urlparse(self.path).path
        if path == f"/api/repos/acme/zrail/git/ref/tags/{self.state.tag}":
            self.send_json(
                200,
                {
                    "ref": f"refs/tags/{self.state.tag}",
                    "object": {"type": "tag", "sha": self.state.tag_object},
                },
            )
        elif path == f"/api/repos/acme/zrail/git/tags/{TAG_OBJECT}":
            self.send_json(
                200,
                {
                    "sha": TAG_OBJECT,
                    "object": {"type": "commit", "sha": self.state.tag_commit},
                },
            )
        elif path == f"/api/repos/acme/zrail/releases/{RELEASE_ID}":
            if self.state.release is None:
                self.send_error(404)
                return
            self.state.events.append("get-release")
            self.state.release_get_ids.append(RELEASE_ID)
            self.send_json(200, self.state.release_json())
        elif path.startswith("/api/repos/acme/zrail/releases/assets/"):
            asset_id = int(path.rsplit("/", 1)[1])
            asset = self.state.assets.get(asset_id)
            if asset is None:
                self.send_error(404)
                return
            if asset["state"] != "uploaded":
                self.send_json(409, {"message": "asset is not downloadable"})
                return
            self.state.asset_downloads.append(asset["name"])
            self.send_bytes(200, asset["content"])
        else:
            self.send_error(404)

    def do_DELETE(self):
        path = urlparse(self.path).path
        prefix = "/api/repos/acme/zrail/releases/assets/"
        if not path.startswith(prefix):
            self.send_error(404)
            return
        raw_asset_id = path.removeprefix(prefix)
        try:
            asset_id = int(raw_asset_id)
        except ValueError:
            self.state.asset_delete_attempts.append(raw_asset_id)
            self.send_error(404)
            return
        self.state.asset_delete_attempts.append(asset_id)
        asset = self.state.assets.get(asset_id)
        if asset is None:
            self.send_error(404)
            return
        if asset["state"] != "starter":
            self.send_json(409, {"message": "refusing to delete uploaded asset"})
            return
        self.state.events.append(f"delete:{asset_id}")
        self.state.asset_deletions.append(asset_id)
        del self.state.assets[asset_id]
        self.send_response(204)
        self.end_headers()

    def do_PATCH(self):
        path = urlparse(self.path).path
        if path != f"/api/repos/acme/zrail/releases/{RELEASE_ID}":
            self.send_error(404)
            return
        assert json.loads(self.read_body()) == {"draft": False}
        self.state.patches += 1
        self.state.release["draft"] = False
        if (
            self.state.fail_publish_response_once
            and not self.state.failed_publish_response
        ):
            self.state.failed_publish_response = True
            self.send_json(502, {"message": "publication applied before response failed"})
            return
        self.send_json(200, self.state.release_json())


class Fixture:
    def __init__(self, helper, version="1.2.3-rc.1"):
        self.helper = helper
        self.temporary = tempfile.TemporaryDirectory(prefix="zrail-release-rehearsal-")
        self.root = Path(self.temporary.name)
        self.assets = self.root / "assets"
        self.assets.mkdir()
        (self.assets / "a.bin").write_bytes(b"first\0asset")
        (self.assets / "b.bin").write_bytes(b"second\0asset")
        self.asset_list = self.root / "assets.txt"
        self.asset_list.write_text("a.bin\nb.bin\n", encoding="utf-8")
        self.notes = self.root / "notes.md"
        self.notes.write_text("Reviewed notes.\n", encoding="utf-8")
        self.release_id = self.root / "release-id"
        self.state = State(version)
        self.server = HTTPServer(("127.0.0.1", 0), Handler)
        self.server.state = self.state
        self.state.base_url = f"http://127.0.0.1:{self.server.server_port}"
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def close(self):
        self.server.shutdown()
        self.thread.join(timeout=5)
        self.server.server_close()
        self.temporary.cleanup()

    def run(self, mode, version=None):
        environment = os.environ.copy()
        environment.update(
            {
                "GH_TOKEN": "test-token",
                "GITHUB_API_URL": f"{self.state.base_url}/api",
                "GITHUB_GRAPHQL_URL": f"{self.state.base_url}/graphql",
                "GITHUB_REPOSITORY": "acme/zrail",
                "GITHUB_REF_NAME": self.state.tag,
                "GITHUB_SHA": COMMIT,
            }
        )
        return subprocess.run(
            [
                sys.executable,
                "-B",
                str(self.helper),
                mode,
                "--assets-dir",
                str(self.assets),
                "--assets-file",
                str(self.asset_list),
                "--notes-file",
                str(self.notes),
                "--release-id-file",
                str(self.release_id),
                "--version",
                version or self.state.version,
                "--title",
                f"zrail {self.state.version}",
            ],
            env=environment,
            capture_output=True,
            text=True,
            timeout=20,
        )


def require_success(result):
    assert result.returncode == 0, result.stderr


def rehearse_stable_identity(helper):
    fixture = Fixture(helper, "1.2.3")
    try:
        require_success(fixture.run("prepare"))
        assert fixture.state.release["prerelease"] is False
        require_success(fixture.run("publish"))
        assert fixture.state.release["draft"] is False
        assert fixture.state.release["prerelease"] is False
    finally:
        fixture.close()


def rehearse_resume(helper):
    fixture = Fixture(helper)
    try:
        fixture.state.fail_second_upload_with_starter_once = True
        first = fixture.run("prepare")
        assert first.returncode != 0
        assert "injected ambiguous upload failure" in first.stderr
        assert fixture.state.created == 1
        assert fixture.state.release_get_ids == [RELEASE_ID]
        assert fixture.state.uploads == ["a.bin"]
        assert [name for name, _ in fixture.state.upload_attempts] == [
            "a.bin",
            "b.bin",
        ]
        assert fixture.state.graphql_results == [None]
        starter_id = fixture.state.starter_asset_id
        assert isinstance(starter_id, int)
        assert fixture.state.assets[starter_id] == {
            "name": "b.bin",
            "content": b"",
            "state": "starter",
        }
        a_id = next(
            asset_id
            for asset_id, asset in fixture.state.assets.items()
            if asset["name"] == "a.bin"
        )
        resume_event_offset = len(fixture.state.events)

        require_success(fixture.run("prepare"))
        assert fixture.state.created == 1
        assert fixture.state.graphql_results == [None, RELEASE_ID]
        assert fixture.state.uploads == ["a.bin", "b.bin"]
        assert [name for name, _ in fixture.state.upload_attempts] == [
            "a.bin",
            "b.bin",
            "b.bin",
        ]
        assert fixture.state.upload_attempts[-1][1] == b"second\0asset"
        assert fixture.state.asset_delete_attempts == [starter_id]
        assert fixture.state.asset_deletions == [starter_id]
        assert a_id in fixture.state.assets
        resume_events = fixture.state.events[resume_event_offset:]
        deletion_index = resume_events.index(f"delete:{starter_id}")
        assert resume_events[deletion_index + 1] == "get-release"
        assert resume_events.index("upload:b.bin", deletion_index) > deletion_index + 1
        assert {"a.bin", "b.bin"} <= set(fixture.state.asset_downloads)
        assert {asset["name"] for asset in fixture.state.assets.values()} == {
            "a.bin",
            "b.bin",
        }
        assert all(
            asset["state"] == "uploaded"
            for asset in fixture.state.assets.values()
        )
        assert {
            asset["name"]: asset["content"]
            for asset in fixture.state.assets.values()
        } == {"a.bin": b"first\0asset", "b.bin": b"second\0asset"}
        assert fixture.release_id.read_text(encoding="utf-8") == f"{RELEASE_ID}\n"

        fixture.state.fail_publish_response_once = True
        failed_publish = fixture.run("publish")
        assert failed_publish.returncode != 0
        assert "publication applied before response failed" in failed_publish.stderr
        assert fixture.state.graphql_results[-1] == RELEASE_ID
        assert fixture.state.patches == 1
        assert fixture.state.release["draft"] is False

        uploads = list(fixture.state.upload_attempts)
        delete_attempts = list(fixture.state.asset_delete_attempts)
        deletions = list(fixture.state.asset_deletions)
        download_offset = len(fixture.state.asset_downloads)
        require_success(fixture.run("prepare"))
        assert fixture.state.created == 1
        assert fixture.state.patches == 1
        assert fixture.state.upload_attempts == uploads
        assert fixture.state.asset_delete_attempts == delete_attempts
        assert fixture.state.asset_deletions == deletions
        assert set(fixture.state.asset_downloads[download_offset:]) == {
            "a.bin",
            "b.bin",
        }
        assert fixture.release_id.read_text(encoding="utf-8") == f"{RELEASE_ID}\n"

        download_offset = len(fixture.state.asset_downloads)
        require_success(fixture.run("publish"))
        assert fixture.state.created == 1
        assert fixture.state.patches == 1
        assert fixture.state.release["draft"] is False
        assert fixture.state.upload_attempts == uploads
        assert fixture.state.asset_delete_attempts == delete_attempts
        assert fixture.state.asset_deletions == deletions
        assert set(fixture.state.asset_downloads[download_offset:]) == {
            "a.bin",
            "b.bin",
        }
    finally:
        fixture.close()


def reject_edited_body(helper):
    fixture = Fixture(helper)
    try:
        require_success(fixture.run("prepare"))
        uploads = list(fixture.state.uploads)
        fixture.state.release["body"] = "Manually edited notes."
        result = fixture.run("prepare")
        assert result.returncode != 0
        assert "metadata differs" in result.stderr
        assert fixture.state.uploads == uploads
        assert fixture.state.release["draft"] is True
    finally:
        fixture.close()


def reject_edited_asset(helper):
    fixture = Fixture(helper)
    try:
        require_success(fixture.run("prepare"))
        uploads = list(fixture.state.upload_attempts)
        asset_ids = set(fixture.state.assets)
        first_asset = next(iter(fixture.state.assets.values()))
        first_asset["content"] = b"different remote bytes"
        result = fixture.run("prepare")
        assert result.returncode != 0
        assert "asset bytes differ" in result.stderr
        assert fixture.state.upload_attempts == uploads
        assert fixture.state.asset_delete_attempts == []
        assert fixture.state.asset_deletions == []
        assert set(fixture.state.assets) == asset_ids
        assert first_asset["content"] == b"different remote bytes"
        assert fixture.state.release["draft"] is True
    finally:
        fixture.close()


def reject_unknown_asset_state(helper):
    fixture = Fixture(helper)
    try:
        require_success(fixture.run("prepare"))
        uploads = list(fixture.state.upload_attempts)
        first_asset = next(iter(fixture.state.assets.values()))
        first_asset["state"] = "processing"
        result = fixture.run("prepare")
        assert result.returncode != 0
        assert fixture.state.upload_attempts == uploads
        assert fixture.state.asset_delete_attempts == []
        assert fixture.state.asset_deletions == []
        assert fixture.state.release["draft"] is True
    finally:
        fixture.close()


def reject_invalid_starter_id(helper):
    fixture = Fixture(helper)
    try:
        require_success(fixture.run("prepare"))
        uploads = list(fixture.state.upload_attempts)
        first_asset = next(iter(fixture.state.assets.values()))
        first_asset["state"] = "starter"
        first_asset["content"] = b""
        first_asset["reported_id"] = "not-a-numeric-id"
        result = fixture.run("prepare")
        assert result.returncode != 0
        assert fixture.state.upload_attempts == uploads
        assert fixture.state.asset_delete_attempts == []
        assert fixture.state.asset_deletions == []
        assert fixture.state.release["draft"] is True
    finally:
        fixture.close()


def reject_wrong_tag_commit(helper):
    fixture = Fixture(helper)
    try:
        fixture.state.tag_commit = OTHER_COMMIT
        result = fixture.run("prepare")
        assert result.returncode != 0
        assert "does not peel to GITHUB_SHA" in result.stderr
        assert fixture.state.created == 0
        assert fixture.state.release is None
    finally:
        fixture.close()


def reject_malformed_tag_object(helper):
    fixture = Fixture(helper)
    try:
        fixture.state.tag_object = "not-a-full-object-id"
        result = fixture.run("prepare")
        assert result.returncode != 0
        assert "invalid Git object" in result.stderr
        assert fixture.state.created == 0
        assert fixture.state.release is None
    finally:
        fixture.close()


def reject_mismatched_version(helper):
    fixture = Fixture(helper)
    try:
        result = fixture.run("prepare", "1.2.3-rc.2")
        assert result.returncode != 0
        assert "tag does not match" in result.stderr
        assert fixture.state.created == 0
        assert fixture.state.release is None
    finally:
        fixture.close()


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: rehearsal.py RELEASE_STATE_HELPER")
    helper = Path(sys.argv[1]).resolve()
    rehearse_stable_identity(helper)
    rehearse_resume(helper)
    reject_edited_body(helper)
    reject_edited_asset(helper)
    reject_unknown_asset_state(helper)
    reject_invalid_starter_id(helper)
    reject_wrong_tag_commit(helper)
    reject_malformed_tag_object(helper)
    reject_mismatched_version(helper)
    print(
        "release API rehearsal: starter cleanup, publication resume, "
        "body, assets, and remote-tag checks passed"
    )


if __name__ == "__main__":
    main()
