//! Exact written quantities stay attributable across source worlds and ownership locations.

#[path = "rust_inventories/boundary_test.rs"]
mod boundary_test;
#[path = "rust_inventories/expression_test.rs"]
mod expression_test;
#[path = "strict_facades/fixture.rs"]
mod fixture;
#[path = "rust_inventories/support.rs"]
mod support;

use support::{SOURCE, configured, coverage, exact, violation};
use zrail_core::{AnalysisQuality, ReportStatus};

#[test]
fn exact_map_rejects_duplicates_deletions_and_same_count_location_or_name_changes() {
    let repository = configured(&exact("src/child.rs", "poll", 1), SOURCE);
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let report = coverage(&repository);
    let inventory = &report.rust_inventories[0];
    assert!(inventory.satisfied);
    assert_eq!(inventory.quality, AnalysisQuality::Exact);
    assert_eq!(inventory.claim, "authored-rust-method-call-syntax");
    assert_eq!(inventory.observed_count, 1);
    assert_eq!(inventory.counts[0].path, "src/child.rs");
    assert_eq!(inventory.occurrence_sample.len(), 1);
    for replacement in [
        SOURCE.replace("self.poll();", "self.poll(); self.poll();"),
        SOURCE.replace("self.poll();", ""),
        SOURCE.replace("self.poll();", "self.wake();"),
        SOURCE.replace("self.poll();", "Self::poll(self);"),
        SOURCE.replace("self.poll();", "let _f = Self::poll;"),
    ] {
        repository.write("src/child.rs", &replacement);
        violation(&repository);
    }
    repository.write("src/child.rs", "//! Data.\npub struct State;\n");
    repository.write(
        "src/moved.rs",
        &SOURCE.replace("pub struct State;", "use super::child::State;"),
    );
    repository.write(
        "src/lib.rs",
        "//! Wiring.\nmod child; mod moved;\npub use child::State;\n",
    );
    assert_eq!(coverage(&repository).rust_inventories[0].observed_count, 1);
    violation(&repository);
}

#[test]
fn authored_counts_include_all_cfg_branches_and_nested_code_once() {
    let source = SOURCE.replace(
        "self.poll();",
        r"
        #[cfg(any())] self.poll();
        #[cfg(test)] self.poll();
        #[cfg(not(test))] self.poll();
        { self.poll(); }
        let _closure = || self.poll();
    ",
    );
    let repository = configured(&exact("src/child.rs", "poll", 5), &source);
    let observed = coverage(&repository);
    assert_eq!(observed.rust_inventories[0].observed_count, 5);
    assert!(observed.rust_inventories[0].satisfied);
    let policy = std::fs::read_to_string(repository.0.join("zrail.toml")).expect("contract");
    repository.write("zrail.toml", &format!("{policy}\n[[source.rust.feature_worlds]]\nname='default'\nreason='Exact default.'\n[[source.rust.feature_worlds.packages]]\npackage='fixture'\ndefault_features=true\nfeatures=[]\n"));
    assert_eq!(
        coverage(&repository).rust_inventories,
        observed.rust_inventories
    );
}

#[test]
fn comments_strings_and_opaque_macro_tokens_do_not_supply_authored_method_calls() {
    let source = SOURCE.replace(
        "self.poll();",
        r#"
        // self.poll();
        let _text = "self.poll();";
        stringify!(self.poll());
        self.poll();
    "#,
    );
    let repository = configured(&exact("src/child.rs", "poll", 1), &source);
    let report = coverage(&repository);
    assert!(report.rust_inventories[0].satisfied);
    repository.write("src/child.rs", &source.replace("        self.poll();", ""));
    violation(&repository);
}

#[test]
fn repeated_includes_preserve_one_physical_occurrence() {
    let repository = configured(
        &exact("src/shared.rs", "poll", 1),
        "//! Data.\npub struct State;\n",
    );
    repository.write("src/shared.rs", SOURCE);
    repository.write("src/child.rs", "//! Mounts.\npub struct State;\nmod a { include!(\"shared.rs\"); }\nmod b { include!(\"shared.rs\"); }\n");
    let report = coverage(&repository);
    assert!(report.rust_inventories[0].satisfied);
    assert_eq!(report.rust_inventories[0].observed_count, 1);
    assert_eq!(report.rust_inventories[0].counts.len(), 1);
}

#[test]
fn binding_includes_all_source_bytes_and_explain_reports_the_effective_inventory() {
    let repository = configured(&exact("src/child.rs", "poll", 1), SOURCE);
    let lock = zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref()).expect("complete lock");
    let report = coverage(&repository);
    assert_eq!(
        report.json().expect("JSON"),
        coverage(&repository).json().expect("repeat JSON")
    );
    assert_eq!(
        repository.explain("src/child.rs").rust_inventories,
        report.rust_inventories
    );
    assert!(report.human().contains("rust:inventory:methods"));
    assert!(
        repository
            .explain("src/child.rs")
            .human()
            .contains("authored-rust-method-call-syntax")
    );
    assert!(
        report
            .enabled_rails
            .contains(&"rust:inventory:methods".into())
    );
    repository.write(
        "src/child.rs",
        &format!("{SOURCE}\n// Bound comment outside the call.\n"),
    );
    let changed = zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref())
        .expect("changed complete lock");
    assert_ne!(lock.analysis, changed.analysis);
    let observed = coverage(&repository);
    assert!(observed.rust_inventories[0].satisfied);
    assert_ne!(
        observed.rust_inventories[0].inputs,
        report.rust_inventories[0].inputs
    );
}
