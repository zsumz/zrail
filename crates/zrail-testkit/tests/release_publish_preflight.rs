//! Crate publication preflight resolves unpublished workspace crates locally.

#![cfg(unix)]

use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn preflight_patches_only_preceding_unpublished_packages() {
    let fixture = temporary_directory();
    let scripts = fixture.join("scripts");
    let runner = fixture.join("runner");
    let fake_bin = fixture.join("bin");
    for directory in [
        &scripts,
        &runner,
        &fake_bin,
        &fixture.join("crates/zrail-core"),
        &fixture.join("crates/zrail-rust"),
        &fixture.join("crates/zrail-cli"),
        &fixture.join("dist"),
    ] {
        fs::create_dir_all(directory).expect("create fixture directory");
    }

    let publisher = scripts.join("publish-crates");
    fs::copy(repository_root().join("scripts/publish-crates"), &publisher).expect("copy publisher");
    make_executable(&publisher);

    let version = "1.2.3-rc.1";
    for package in ["zrail-core", "zrail-rust", "zrail"] {
        fs::write(
            fixture.join(format!("dist/{package}-{version}.crate")),
            format!("attested {package}\n"),
        )
        .expect("write attested archive");
    }

    let fake_cargo = fake_bin.join("cargo");
    fs::write(&fake_cargo, FAKE_CARGO).expect("write fake cargo");
    make_executable(&fake_cargo);

    let path = format!(
        "{}:{}",
        fake_bin.display(),
        env::var("PATH").expect("PATH is set")
    );
    let output = Command::new(&publisher)
        .arg("--preflight")
        .current_dir(&fixture)
        .env("PATH", path)
        .env("RUNNER_TEMP", &runner)
        .env("ZRAIL_VERSION", version)
        .output()
        .expect("run publication preflight");

    assert!(
        output.status.success(),
        "preflight failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(fixture.join("cargo.log")).expect("read cargo log"),
        "zrail-core\nzrail-rust\nzrail\n"
    );
    fs::remove_dir_all(fixture).expect("remove fixture");
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
    env::temp_dir().join(format!(
        "zrail-publish-preflight-{}-{nonce}",
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

const FAKE_CARGO: &str = r#"#!/usr/bin/env bash
set -euo pipefail

package=
configs=()
while (( $# > 0 )); do
    case "$1" in
        --package)
            package=$2
            shift 2
            ;;
        --config)
            configs+=("$2")
            shift 2
            ;;
        *) shift ;;
    esac
done

core_patch="patch.crates-io.zrail-core.path = '$PWD/crates/zrail-core'"
rust_patch="patch.crates-io.zrail-rust.path = '$PWD/crates/zrail-rust'"
case "$package" in
    zrail-core)
        (( ${#configs[@]} == 0 ))
        ;;
    zrail-rust)
        (( ${#configs[@]} == 1 ))
        [[ "${configs[0]}" == "$core_patch" ]]
        ;;
    zrail)
        (( ${#configs[@]} == 2 ))
        [[ "${configs[0]}" == "$core_patch" ]]
        [[ "${configs[1]}" == "$rust_patch" ]]
        ;;
    *) exit 1 ;;
esac

printf '%s\n' "$package" >> cargo.log
mkdir -p target/package/tmp-crate
cp "dist/$package-$ZRAIL_VERSION.crate" \
    "target/package/tmp-crate/$package-$ZRAIL_VERSION.crate"
"#;
