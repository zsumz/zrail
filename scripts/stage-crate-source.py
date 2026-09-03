#!/usr/bin/env python3
"""Safely stage one .crate archive as a Cargo directory-source package."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import sys
import tarfile
import tempfile
from typing import Optional


MAX_ARCHIVE_BYTES = 64 * 1024 * 1024
MAX_MEMBERS = 100_000
MAX_FILES = 100_000
MAX_EXPANDED_BYTES = 512 * 1024 * 1024
CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"


class StageError(Exception):
    pass


def basic_string(value: str, description: str) -> str:
    try:
        parsed = json.loads(value)
    except (TypeError, json.JSONDecodeError) as error:
        raise StageError(f"invalid {description}: {error}") from error
    if not isinstance(parsed, str):
        raise StageError(f"invalid {description}: expected a basic string")
    return parsed


def package_identity(path: Path) -> tuple[str, str]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError) as error:
        raise StageError(f"cannot read packaged Cargo.toml identity: {error}") from error
    table = None
    fields: dict[str, str] = {}
    for line in lines:
        stripped = line.strip()
        if '"""' in stripped or "'''" in stripped:
            raise StageError("packaged Cargo.toml identity uses an unsupported multiline string")
        if stripped == "[package]":
            table = "package"
            continue
        if stripped.startswith("["):
            table = None
            continue
        if table != "package" or "=" not in stripped:
            continue
        key, value = (part.strip() for part in stripped.split("=", 1))
        if key not in {"name", "version"}:
            continue
        if key in fields:
            raise StageError(f"packaged Cargo.toml repeats package.{key}")
        fields[key] = basic_string(value, f"packaged Cargo.toml package.{key}")
    if set(fields) != {"name", "version"}:
        raise StageError("packaged Cargo.toml lacks an explicit package name or version")
    return fields["name"], fields["version"]


def lock_packages(path: Path) -> list[dict[str, str]]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError) as error:
        raise StageError(f"cannot read packaged Cargo.lock: {error}") from error
    packages: list[dict[str, str]] = []
    package: Optional[dict[str, str]] = None
    for line in lines:
        stripped = line.strip()
        if '"""' in stripped or "'''" in stripped:
            raise StageError("packaged Cargo.lock uses an unsupported multiline string")
        if stripped == "[[package]]":
            if package is not None:
                packages.append(package)
            package = {}
            continue
        if stripped.startswith("["):
            if package is not None:
                packages.append(package)
                package = None
            continue
        if package is None or "=" not in stripped:
            continue
        key, value = (part.strip() for part in stripped.split("=", 1))
        if key not in {"name", "version", "source", "checksum"}:
            continue
        if key in package:
            raise StageError(f"packaged Cargo.lock repeats package field {key}")
        package[key] = basic_string(value, f"packaged Cargo.lock package {key}")
    if package is not None:
        packages.append(package)
    if any("name" not in package or "version" not in package for package in packages):
        raise StageError("packaged Cargo.lock has an invalid package list")
    return packages


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require_regular_file(path: Path, description: str) -> None:
    if path.is_symlink() or not path.is_file():
        raise StageError(f"{description} is not a regular file: {path}")


def validate_identity(value: str, description: str) -> None:
    if not value or value in {".", ".."} or "/" in value or "\\" in value:
        raise StageError(f"invalid {description}: {value!r}")


def member_parts(member: tarfile.TarInfo, expected_root: str) -> tuple[str, ...]:
    name = member.name.rstrip("/")
    if not name or "\\" in name:
        raise StageError(f"invalid archive member path: {member.name!r}")
    path = PurePosixPath(name)
    parts = tuple(name.split("/"))
    if path.is_absolute() or any(part in {"", ".", ".."} for part in parts):
        raise StageError(f"unsafe archive member path: {member.name!r}")
    if parts[0] != expected_root:
        raise StageError(
            f"archive member is outside the single expected root {expected_root!r}: "
            f"{member.name!r}"
        )
    return parts


def verify_locked_dependencies(
    root: Path, dependencies: list[tuple[str, str, Path]]
) -> None:
    if not dependencies:
        return
    lock_path = root / "Cargo.lock"
    require_regular_file(lock_path, "packaged Cargo.lock")
    packages = lock_packages(lock_path)
    for name, version, archive in dependencies:
        require_regular_file(archive, f"dependency archive for {name}")
        matches = [
            package
            for package in packages
            if package.get("name") == name and package.get("version") == version
        ]
        if len(matches) != 1:
            raise StageError(
                f"packaged Cargo.lock must contain exactly one {name} {version} node"
            )
        package = matches[0]
        if package.get("source") != CRATES_IO_SOURCE:
            raise StageError(f"packaged Cargo.lock has non-crates.io source for {name}")
        expected_checksum = sha256(archive)
        if package.get("checksum") != expected_checksum:
            raise StageError(f"packaged Cargo.lock has wrong archive checksum for {name}")


def stage_archive(
    archive: Path,
    directory: Path,
    name: str,
    version: str,
    dependencies: list[tuple[str, str, Path]],
) -> None:
    validate_identity(name, "package name")
    validate_identity(version, "package version")
    require_regular_file(archive, "crate archive")
    if archive.stat().st_size > MAX_ARCHIVE_BYTES:
        raise StageError(f"crate archive exceeds {MAX_ARCHIVE_BYTES} bytes")
    if directory.is_symlink() or (directory.exists() and not directory.is_dir()):
        raise StageError(f"directory source is not a directory: {directory}")
    directory.mkdir(parents=True, exist_ok=True)

    expected_root = f"{name}-{version}"
    destination = directory / expected_root
    if destination.exists() or destination.is_symlink():
        raise StageError(f"directory-source package already exists: {destination}")

    try:
        archive_bytes = archive.read_bytes()
    except OSError as error:
        raise StageError(f"cannot read crate archive: {error}") from error
    if len(archive_bytes) > MAX_ARCHIVE_BYTES:
        raise StageError(f"crate archive exceeds {MAX_ARCHIVE_BYTES} bytes")
    archive_checksum = hashlib.sha256(archive_bytes).hexdigest()
    temporary = Path(tempfile.mkdtemp(prefix=f".{expected_root}.", dir=directory))
    staged_root = temporary / expected_root
    seen: set[tuple[str, ...]] = set()
    member_count = 0
    file_count = 0
    expanded_bytes = 0
    try:
        try:
            with tarfile.open(fileobj=io.BytesIO(archive_bytes), mode="r|gz") as bundle:
                for member in bundle:
                    member_count += 1
                    if member_count > MAX_MEMBERS:
                        raise StageError("crate archive exceeds member limit")
                    parts = member_parts(member, expected_root)
                    if parts in seen:
                        raise StageError(f"duplicate archive member: {member.name!r}")
                    seen.add(parts)
                    target = temporary.joinpath(*parts)
                    if member.isdir():
                        target.mkdir(parents=True, exist_ok=True)
                        continue
                    if not member.isreg():
                        raise StageError(
                            f"archive member is not a regular file: {member.name!r}"
                        )
                    if member.size < 0:
                        raise StageError(f"archive member has a negative size: {member.name!r}")
                    file_count += 1
                    expanded_bytes += member.size
                    if file_count > MAX_FILES or expanded_bytes > MAX_EXPANDED_BYTES:
                        raise StageError("crate archive exceeds extraction limits")
                    target.parent.mkdir(parents=True, exist_ok=True)
                    if target.exists() or target.is_symlink():
                        raise StageError(f"archive member collides on disk: {member.name!r}")
                    source = bundle.extractfile(member)
                    if source is None:
                        raise StageError(f"cannot read archive member: {member.name!r}")
                    with target.open("xb") as output:
                        shutil.copyfileobj(source, output)
                    os.chmod(target, (member.mode & 0o777) | 0o600)
        except (OSError, tarfile.TarError) as error:
            raise StageError(f"cannot extract crate archive: {error}") from error

        manifest_path = staged_root / "Cargo.toml"
        require_regular_file(manifest_path, "packaged Cargo.toml")
        checksum_path = staged_root / ".cargo-checksum.json"
        if checksum_path.exists() or checksum_path.is_symlink():
            raise StageError("crate archive already contains .cargo-checksum.json")
        manifest_name, manifest_version = package_identity(manifest_path)
        if manifest_name != name or manifest_version != version:
            raise StageError("packaged Cargo.toml identity does not match the requested package")

        verify_locked_dependencies(staged_root, dependencies)
        file_checksums = {
            path.relative_to(staged_root).as_posix(): sha256(path)
            for path in sorted(staged_root.rglob("*"))
            if path.is_file() and not path.is_symlink()
        }
        checksum_path.write_text(
            json.dumps(
                {"files": file_checksums, "package": archive_checksum},
                separators=(",", ":"),
                sort_keys=True,
            ),
            encoding="utf-8",
        )
        try:
            staged_root.rename(destination)
        except OSError as error:
            raise StageError(f"cannot install directory-source package: {error}") from error
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--archive", required=True, type=Path)
    parser.add_argument("--directory", required=True, type=Path)
    parser.add_argument("--name", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument(
        "--locked-dependency",
        action="append",
        default=[],
        nargs=3,
        metavar=("NAME", "VERSION", "ARCHIVE"),
    )
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    dependencies = [
        (name, version, Path(archive))
        for name, version, archive in arguments.locked_dependency
    ]
    try:
        stage_archive(
            arguments.archive,
            arguments.directory,
            arguments.name,
            arguments.version,
            dependencies,
        )
    except StageError as error:
        print(f"stage-crate-source: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
