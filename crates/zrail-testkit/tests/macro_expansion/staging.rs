//! Allow mode validates reviewed entries without enforcing them yet.

use std::fs;

use zrail_rust::build_lock;

use super::{check, repository, reset};

#[test]
fn staged_allowance_is_checked_while_unreviewed_macros_remain_allowed() {
    let root = repository(
        "staged",
        "//! Staged macros.\nmod local { macro_rules! reviewed { () => { 1 }; } pub(crate) use reviewed; }\npub fn run() { let _ = local::reviewed!(); unreviewed!(); }\n",
        r#"
[[source.rust.macros.allow]]
name = "local::reviewed"
definition = "src/lib.rs"
reason = "The local transcriber expands to one integer literal."
"#,
    );
    let contract = fs::read_to_string(root.join("zrail.toml"))
        .expect("read staged contract")
        .replace("mode = \"deny-unreviewed\"", "mode = \"allow\"");
    fs::write(root.join("zrail.toml"), contract).expect("stage macro enforcement");
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build staged macro lock")
        .write(&root.join("zrail.lock"))
        .expect("write staged macro lock");

    let report = check(&root);
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| { matches!(finding.id.as_str(), "RUST-MACRO-001" | "RUST-MACRO-002") })
    );

    fs::write(
        root.join("src/lib.rs"),
        "//! Only an unreviewed macro remains.\npub fn run() { unreviewed!(); }\n",
    )
    .expect("remove staged invocation");
    let stale = check(&root);
    assert!(stale.findings.iter().any(|finding| {
        finding.id == "RUST-MACRO-002" && finding.message.contains("local::reviewed")
    }));
    reset(&root);
}
