//! Crate publication preflight uses canonical offline registry identities.

#![cfg(unix)]

use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

#[test]
fn preflight_stages_all_predecessors_before_any_publication_traffic() {
    let fixture = Fixture::new("valid");
    let output = fixture.run();

    assert_success(&output);
    assert_eq!(
        fixture.log(),
        "vendor\npublish zrail-core\npublish zrail-rust\npublish zrail\n",
        "trace:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.root.join("registry-traffic").exists());
    for package in ["zrail-core", "zrail-rust", "zrail"] {
        let archive = format!("{package}-{}.crate", fixture.version);
        assert_eq!(
            fs::read(fixture.root.join("dist").join(&archive)).expect("read attested crate"),
            fs::read(fixture.root.join("target/publish-preflight").join(&archive))
                .expect("read byte proof")
        );
    }
}

#[test]
fn preflight_rejects_non_registry_sources_and_wrong_archive_checksums() {
    for fault in ["bad-source", "bad-checksum"] {
        let fixture = Fixture::new(fault);
        let output = fixture.run();

        assert!(!output.status.success(), "{fault} unexpectedly passed");
        assert_eq!(
            fixture.log(),
            "vendor\npublish zrail-core\npublish zrail-rust\n"
        );
        assert!(!fixture.root.join("registry-traffic").exists());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("non-crates.io source") || stderr.contains("wrong archive checksum"),
            "unexpected failure for {fault}: {stderr}"
        );
    }
}

struct Fixture {
    root: PathBuf,
    publisher: PathBuf,
    path: String,
    version: &'static str,
}

impl Fixture {
    fn new(lock_mode: &str) -> Self {
        let root = temporary_directory();
        for directory in ["scripts", "runner", "bin", "dist"] {
            fs::create_dir_all(root.join(directory)).expect("create fixture directory");
        }
        let publisher = root.join("scripts/publish-crates");
        fs::copy(repository_root().join("scripts/publish-crates"), &publisher)
            .expect("copy publisher");
        fs::copy(
            repository_root().join("scripts/stage-crate-source.py"),
            root.join("scripts/stage-crate-source.py"),
        )
        .expect("copy staging helper");
        make_executable(&publisher);
        for (name, body) in [("cargo", FAKE_CARGO), ("curl", FAKE_CURL)] {
            let path = root.join("bin").join(name);
            fs::write(&path, body).expect("write fake executable");
            make_executable(&path);
        }
        let version = "1.2.3-rc.1";
        let generated = Command::new("python3")
            .args(["-c", CREATE_CRATES])
            .arg(root.join("dist"))
            .arg(version)
            .arg(lock_mode)
            .output()
            .expect("create crate fixtures");
        assert_success(&generated);
        let path = format!(
            "{}:{}",
            root.join("bin").display(),
            env::var("PATH").expect("PATH is set")
        );
        Self {
            root,
            publisher,
            path,
            version,
        }
    }

    fn run(&self) -> Output {
        Command::new(&self.publisher)
            .arg("--preflight")
            .current_dir(&self.root)
            .env("PATH", &self.path)
            .env("RUNNER_TEMP", self.root.join("runner"))
            .env("ZRAIL_VERSION", self.version)
            .output()
            .expect("run publication preflight")
    }

    fn log(&self) -> String {
        fs::read_to_string(self.root.join("cargo.log")).expect("read cargo log")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn make_executable(path: &Path) {
    let mut permissions = fs::metadata(path).expect("read permissions").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("set executable permissions");
}

fn temporary_directory() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
    env::temp_dir().join(format!(
        "zrail-publish-preflight-{}-{nonce}-{sequence}",
        std::process::id()
    ))
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("testkit lives below repository root")
        .to_path_buf()
}

const CREATE_CRATES: &str = r#"
import hashlib, io, pathlib, sys, tarfile

dist, version, mode = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]
source = "registry+https://github.com/rust-lang/crates.io-index"

def write_archive(name, dependencies):
    root = f"{name}-{version}"
    manifest = f'[package]\nname = "{name}"\nversion = "{version}"\nedition = "2021"\n'
    lock = f'version = 4\n\n[[package]]\nname = "{name}"\nversion = "{version}"\n'
    for dependency, checksum in dependencies:
        dependency_source = "path+file:///forbidden" if mode == "bad-source" and name == "zrail-rust" else source
        dependency_checksum = "0" * 64 if mode == "bad-checksum" and name == "zrail-rust" else checksum
        lock += f'\n[[package]]\nname = "{dependency}"\nversion = "{version}"\nsource = "{dependency_source}"\nchecksum = "{dependency_checksum}"\n'
    files = {"Cargo.toml": manifest, "Cargo.lock": lock, "LICENSE": "license\n", "README.md": "readme\n"}
    archive = dist / f"{name}-{version}.crate"
    with tarfile.open(archive, "w:gz") as bundle:
        for relative, text in files.items():
            data = text.encode()
            member = tarfile.TarInfo(f"{root}/{relative}")
            member.size, member.mode, member.mtime = len(data), 0o644, 0
            bundle.addfile(member, io.BytesIO(data))
    return hashlib.sha256(archive.read_bytes()).hexdigest()

core = write_archive("zrail-core", [])
rust = write_archive("zrail-rust", [("zrail-core", core)])
write_archive("zrail", [("zrail-core", core), ("zrail-rust", rust)])
"#;

const FAKE_CARGO: &str = r#"#!/usr/bin/env bash
set -euo pipefail

command=$1
shift
if [[ "$command" == vendor ]]; then
    [[ " $* " == *" --locked "* ]]
    [[ " $* " == *" --versioned-dirs "* ]]
    source_dir=${!#}
    mkdir -p "$source_dir"
    printf 'vendor\n' >> cargo.log
    exit 0
fi
[[ "$command" == publish ]]

package= registry= source_dir=
locked=false no_verify=false dry_run=false
configs=()
while (( $# > 0 )); do
    case "$1" in
        --package) package=$2; shift 2 ;;
        --registry) registry=$2; shift 2 ;;
        --config) configs+=("$2"); shift 2 ;;
        --locked) locked=true; shift ;;
        --no-verify) no_verify=true; shift ;;
        --dry-run) dry_run=true; shift ;;
        *) exit 1 ;;
    esac
done
[[ "$registry" == crates-io && "$locked" == true ]]
[[ "$no_verify" == true && "$dry_run" == true && ${#configs[@]} == 2 ]]
[[ "${configs[0]}" == "source.crates-io.replace-with = 'zrail-publish-source'" ]]
case "${configs[1]}" in
    "source.zrail-publish-source.directory = '"*"'") ;;
    *) exit 1 ;;
esac
source_dir=${configs[1]#*\'}
source_dir=${source_dir%\'}
case "$package" in
    zrail-core) [[ ! -e "$source_dir/zrail-core-$ZRAIL_VERSION" ]] ;;
    zrail-rust) test -f "$source_dir/zrail-core-$ZRAIL_VERSION/.cargo-checksum.json" ;;
    zrail)
        test -f "$source_dir/zrail-core-$ZRAIL_VERSION/.cargo-checksum.json"
        test -f "$source_dir/zrail-rust-$ZRAIL_VERSION/.cargo-checksum.json"
        ;;
    *) exit 1 ;;
esac
printf 'publish %s\n' "$package" >> cargo.log
mkdir -p target/package/tmp-crate
cp "dist/$package-$ZRAIL_VERSION.crate" \
    "target/package/tmp-crate/$package-$ZRAIL_VERSION.crate"
"#;

const FAKE_CURL: &str = r"#!/usr/bin/env bash
touch registry-traffic
exit 99
";
