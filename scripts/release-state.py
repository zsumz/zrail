#!/usr/bin/env python3
"""Prepare and publish one byte-exact GitHub release through pinned APIs."""

from __future__ import annotations

import argparse
from enum import Enum
import json
import os
from pathlib import Path
import re
import sys
from urllib.error import HTTPError, URLError
from urllib.parse import quote, urlencode, urlparse
from urllib.request import HTTPRedirectHandler, Request, build_opener


VERSION = re.compile(
    r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
)


class ReleaseError(RuntimeError):
    """A fail-closed release-state error."""


class ReleaseKind(Enum):
    ABSENT = "absent"
    EXACT_DRAFT = "exact draft"
    EXACT_PUBLISHED = "exact published"
    INVALID = "invalid"


class SafeRedirects(HTTPRedirectHandler):
    """Do not forward the GitHub token to an asset storage host."""

    def redirect_request(self, request, fp, code, msg, headers, new_url):
        redirected = super().redirect_request(request, fp, code, msg, headers, new_url)
        if redirected is not None and urlparse(request.full_url).netloc != urlparse(new_url).netloc:
            redirected.remove_header("Authorization")
        return redirected


class GitHub:
    def __init__(self, api_url: str, graphql_url: str, repository: str, token: str):
        try:
            self.owner, self.name = repository.split("/", 1)
        except ValueError as error:
            raise ReleaseError("GITHUB_REPOSITORY must be owner/name") from error
        if not self.owner or not self.name or "/" in self.name:
            raise ReleaseError("GITHUB_REPOSITORY must be owner/name")
        self.api_url = api_url.rstrip("/")
        self.graphql_url = graphql_url
        self.token = token
        self.opener = build_opener(SafeRedirects())

    def repository_url(self, suffix: str) -> str:
        owner = quote(self.owner, safe="")
        name = quote(self.name, safe="")
        return f"{self.api_url}/repos/{owner}/{name}/{suffix.lstrip('/')}"

    def request_json(self, method: str, url: str, payload=None, expected=(200,)):
        data = None if payload is None else json.dumps(payload).encode()
        body = self._request(method, url, data, "application/vnd.github+json", expected)
        try:
            value = json.loads(body)
        except json.JSONDecodeError as error:
            raise ReleaseError(f"GitHub returned invalid JSON from {url}") from error
        return value

    def request_bytes(self, url: str) -> bytes:
        return self._request("GET", url, None, "application/octet-stream", (200,))

    def upload(self, url: str, name: str, content: bytes):
        endpoint = url.split("{", 1)[0]
        separator = "&" if "?" in endpoint else "?"
        endpoint = f"{endpoint}{separator}{urlencode({'name': name})}"
        return self._request(
            "POST", endpoint, content, "application/vnd.github+json", (201,),
            content_type="application/octet-stream",
        )

    def delete(self, url: str):
        return self._request(
            "DELETE", url, None, "application/vnd.github+json", (204,)
        )

    def _request(self, method, url, data, accept, expected, content_type="application/json"):
        headers = {
            "Accept": accept,
            "Authorization": f"Bearer {self.token}",
            "Content-Type": content_type,
            "User-Agent": "zrail-release-state",
            "X-GitHub-Api-Version": "2022-11-28",
        }
        request = Request(url, data=data, headers=headers, method=method)
        try:
            with self.opener.open(request) as response:
                status = response.status
                body = response.read()
        except HTTPError as error:
            detail = error.read().decode(errors="replace")
            raise ReleaseError(f"GitHub {method} {url} returned HTTP {error.code}: {detail}") from error
        except URLError as error:
            raise ReleaseError(f"GitHub {method} {url} failed: {error.reason}") from error
        if status not in expected:
            raise ReleaseError(f"GitHub {method} {url} returned HTTP {status}")
        return body


class ReleaseState:
    def __init__(self, arguments):
        required = [
            "GH_TOKEN", "GITHUB_API_URL", "GITHUB_GRAPHQL_URL",
            "GITHUB_REPOSITORY", "GITHUB_REF_NAME", "GITHUB_SHA",
        ]
        missing = [name for name in required if not os.environ.get(name)]
        if missing:
            raise ReleaseError(f"missing required environment: {', '.join(missing)}")
        self.github = GitHub(
            os.environ["GITHUB_API_URL"], os.environ["GITHUB_GRAPHQL_URL"],
            os.environ["GITHUB_REPOSITORY"], os.environ["GH_TOKEN"],
        )
        self.tag = os.environ["GITHUB_REF_NAME"]
        self.version = arguments.version
        if VERSION.fullmatch(self.version) is None:
            raise ReleaseError("release version must be stable or a SemVer prerelease")
        if self.tag != f"v{self.version}":
            raise ReleaseError("release tag does not match the exact release version")
        self.commit = os.environ["GITHUB_SHA"]
        if re.fullmatch(r"[0-9a-fA-F]{40}|[0-9a-fA-F]{64}", self.commit) is None:
            raise ReleaseError("GITHUB_SHA must be a full hexadecimal commit ID")
        self.title = arguments.title
        self.prerelease = "-" in self.version
        self.body = arguments.notes_file.read_text(encoding="utf-8")
        self.assets_dir = arguments.assets_dir
        self.asset_names = self._read_asset_names(arguments.assets_file)
        self.release_id_file = arguments.release_id_file
        actual = {path.name for path in self.assets_dir.iterdir() if path.is_file()}
        if actual != set(self.asset_names):
            raise ReleaseError("local release assets do not match the reviewed asset-name set")

    def _read_asset_names(self, path: Path) -> list[str]:
        names = path.read_text(encoding="utf-8").splitlines()
        if not names or len(names) != len(set(names)):
            raise ReleaseError("asset-name list must be non-empty and unique")
        for name in names:
            if not name or Path(name).name != name or name in {".", ".."}:
                raise ReleaseError(f"invalid release asset name: {name!r}")
        return names

    def verify_remote_tag(self):
        encoded_tag = quote(self.tag, safe="")
        value = self.github.request_json(
            "GET", self.github.repository_url(f"git/ref/tags/{encoded_tag}")
        )
        if value.get("ref") != f"refs/tags/{self.tag}":
            raise ReleaseError("GitHub returned a different remote tag reference")
        target = self._git_object(value)
        seen = set()
        for _ in range(16):
            kind, sha = target
            if kind == "commit":
                if sha.lower() != self.commit.lower():
                    raise ReleaseError("remote tag does not peel to GITHUB_SHA")
                return
            if kind != "tag" or sha in seen:
                raise ReleaseError("remote tag does not peel unambiguously to a commit")
            seen.add(sha)
            value = self.github.request_json(
                "GET", self.github.repository_url(f"git/tags/{quote(sha, safe='')}")
            )
            if value.get("sha") != sha:
                raise ReleaseError("GitHub returned a different annotated tag object")
            target = self._git_object(value)
        raise ReleaseError("remote tag annotation chain is too deep")

    @staticmethod
    def _git_object(value) -> tuple[str, str]:
        target = value.get("object")
        if not isinstance(target, dict):
            raise ReleaseError("GitHub tag response has no Git object")
        kind, sha = target.get("type"), target.get("sha")
        valid_sha = isinstance(sha, str) and re.fullmatch(
            r"[0-9a-fA-F]{40}|[0-9a-fA-F]{64}", sha
        )
        if kind not in {"commit", "tag"} or not valid_sha:
            raise ReleaseError("GitHub tag response has an invalid Git object")
        return kind, sha

    def find_release_id(self) -> int | None:
        query = """query($owner:String!,$name:String!,$tag:String!){
          repository(owner:$owner,name:$name){release(tagName:$tag){databaseId}}
        }"""
        payload = {
            "query": query,
            "variables": {"owner": self.github.owner, "name": self.github.name, "tag": self.tag},
        }
        result = self.github.request_json("POST", self.github.graphql_url, payload)
        if result.get("errors"):
            raise ReleaseError("GitHub GraphQL draft lookup returned errors")
        repository = result.get("data", {}).get("repository")
        release = repository.get("release") if isinstance(repository, dict) else None
        if release is None:
            return None
        release_id = release.get("databaseId") if isinstance(release, dict) else None
        if isinstance(release_id, bool) or not isinstance(release_id, int) or release_id <= 0:
            raise ReleaseError("GitHub GraphQL returned an invalid release database ID")
        return release_id

    def release(self, release_id: int):
        return self.github.request_json(
            "GET", self.github.repository_url(f"releases/{release_id}")
        )

    def create_draft(self) -> int:
        value = self.github.request_json(
            "POST", self.github.repository_url("releases"),
            {"tag_name": self.tag, "name": self.title, "body": self.body,
             "draft": True, "prerelease": self.prerelease},
            expected=(201,),
        )
        release_id = value.get("id")
        if isinstance(release_id, bool) or not isinstance(release_id, int) or release_id <= 0:
            raise ReleaseError("GitHub create release returned an invalid database ID")
        return release_id

    def classify(self, release_id: int | None, release=None) -> ReleaseKind:
        if release_id is None:
            return ReleaseKind.ABSENT
        expected = {
            "tag_name": self.tag, "name": self.title, "body": self.body,
        }
        valid = (
            isinstance(release, dict)
            and type(release.get("id")) is int
            and release["id"] == release_id
            and type(release.get("draft")) is bool
            and release.get("prerelease") is self.prerelease
            and all(release.get(key) == value for key, value in expected.items())
        )
        if not valid:
            return ReleaseKind.INVALID
        if release["draft"]:
            return ReleaseKind.EXACT_DRAFT
        return ReleaseKind.EXACT_PUBLISHED

    def current_release(self):
        release_id = self.find_release_id()
        release = None if release_id is None else self.release(release_id)
        return self.classify(release_id, release), release_id, release

    def require_kind(self, release, release_id: int, expected: ReleaseKind):
        if self.classify(release_id, release) is not expected:
            raise ReleaseError("release metadata differs from the reviewed release identity")

    def assets(self, release):
        assets = release.get("assets")
        if not isinstance(assets, list):
            raise ReleaseError("release assets are not an array")
        by_name = {}
        for asset in assets:
            if not isinstance(asset, dict) or not isinstance(asset.get("name"), str):
                raise ReleaseError("release contains an invalid asset")
            name = asset["name"]
            if name in by_name:
                raise ReleaseError(f"release contains duplicate asset: {name}")
            by_name[name] = asset
        unexpected = set(by_name) - set(self.asset_names)
        if unexpected:
            raise ReleaseError(f"release contains unexpected assets: {sorted(unexpected)}")
        return by_name

    def inspect_assets(self, release, complete: bool, allow_starter: bool):
        by_name = self.assets(release)
        missing = set(self.asset_names) - set(by_name)
        if complete and missing:
            raise ReleaseError(f"release is missing assets: {sorted(missing)}")
        starters = []
        starter_ids = set()
        for name, asset in by_name.items():
            state = asset.get("state")
            if state == "starter" and allow_starter:
                asset_id = asset.get("id")
                if (
                    isinstance(asset_id, bool)
                    or not isinstance(asset_id, int)
                    or asset_id <= 0
                    or asset_id in starter_ids
                ):
                    raise ReleaseError(f"release starter asset has an invalid ID: {name}")
                starter_ids.add(asset_id)
                starters.append((name, asset_id))
                continue
            if state != "uploaded":
                raise ReleaseError(f"release asset has an invalid state: {name}")
            url = asset.get("url")
            if not isinstance(url, str) or not url:
                raise ReleaseError(f"release asset has no API URL: {name}")
            actual = self.github.request_bytes(url)
            expected_bytes = (self.assets_dir / name).read_bytes()
            if actual != expected_bytes:
                raise ReleaseError(f"release asset bytes differ: {name}")
        return by_name, missing, starters

    def validate(self, release, release_id: int, kind: ReleaseKind, complete: bool,
                 allow_starter: bool = False):
        self.require_kind(release, release_id, kind)
        result = self.inspect_assets(release, complete, allow_starter)
        self.verify_remote_tag()
        return result

    def delete_release_asset(self, asset_id: int):
        self.github.delete(
            self.github.repository_url(f"releases/assets/{asset_id}")
        )

    def prepare_draft(self, release_id: int, release):
        _, missing, starters = self.validate(
            release, release_id, ReleaseKind.EXACT_DRAFT, False, True
        )
        deleted = set()
        while starters:
            name, asset_id = starters[0]
            self.delete_release_asset(asset_id)
            deleted.add(name)
            release = self.release(release_id)
            by_name, missing, starters = self.validate(
                release, release_id, ReleaseKind.EXACT_DRAFT, False, True
            )
            remaining = deleted & set(by_name)
            if remaining:
                raise ReleaseError(
                    f"release starter asset remains after deletion: {sorted(remaining)[0]}"
                )
        upload_url = release.get("upload_url")
        if not isinstance(upload_url, str) or not upload_url:
            raise ReleaseError("release has no asset upload URL")
        for name in self.asset_names:
            if name in missing:
                self.github.upload(upload_url, name, (self.assets_dir / name).read_bytes())
        self.validate(
            self.release(release_id), release_id, ReleaseKind.EXACT_DRAFT, True
        )

    def prepare(self):
        self.verify_remote_tag()
        kind, release_id, release = self.current_release()
        if kind is ReleaseKind.ABSENT:
            release_id = self.create_draft()
            release = self.release(release_id)
            kind = self.classify(release_id, release)
        if kind is ReleaseKind.INVALID:
            raise ReleaseError("release metadata differs from the reviewed release identity")
        if kind is ReleaseKind.EXACT_DRAFT:
            self.prepare_draft(release_id, release)
        elif kind is ReleaseKind.EXACT_PUBLISHED:
            self.validate(release, release_id, kind, True)
        else:
            raise ReleaseError("release lookup returned an invalid state")
        self.release_id_file.write_text(f"{release_id}\n", encoding="utf-8")

    def publish(self):
        try:
            expected_id = int(self.release_id_file.read_text(encoding="utf-8").strip())
        except (OSError, ValueError) as error:
            raise ReleaseError("prepared release database ID is missing or invalid") from error
        if expected_id <= 0:
            raise ReleaseError("prepared release database ID is missing or invalid")
        kind, release_id, release = self.current_release()
        if release_id != expected_id or kind is ReleaseKind.ABSENT:
            raise ReleaseError("prepared and remote release database IDs differ")
        if kind is ReleaseKind.INVALID:
            raise ReleaseError("release metadata differs from the reviewed release identity")
        if kind is ReleaseKind.EXACT_PUBLISHED:
            self.validate(release, release_id, kind, True)
            return
        self.validate(release, release_id, ReleaseKind.EXACT_DRAFT, True)
        published = self.github.request_json(
            "PATCH", self.github.repository_url(f"releases/{release_id}"),
            {"draft": False},
        )
        self.validate(published, release_id, ReleaseKind.EXACT_PUBLISHED, True)


def parse_arguments():
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("prepare", "publish"))
    parser.add_argument("--assets-dir", type=Path, required=True)
    parser.add_argument("--assets-file", type=Path, required=True)
    parser.add_argument("--notes-file", type=Path, required=True)
    parser.add_argument("--release-id-file", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--title", required=True)
    return parser.parse_args()


def main() -> int:
    try:
        arguments = parse_arguments()
        state = ReleaseState(arguments)
        getattr(state, arguments.mode)()
    except (OSError, ReleaseError) as error:
        print(f"release state error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
