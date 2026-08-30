//! Zrail's own projection workload is a tightening qualification ratchet.

use std::{
    io::{self, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use zrail_rust::build_lock;

// Tightened to the committed self-analysis workload after source changes settle.
const MAX_PROJECTION_QUERIES: usize = 1_675_464;

#[test]
fn self_hosted_projection_work_does_not_regress() {
    let lock = build_lock(&repository_root(), "zrail.toml".as_ref())
        .expect("build zrail's self-hosted lock candidate");
    let analysis = lock.analysis.expect("self-hosted analysis certificate");

    assert!(
        analysis.projection_queries <= MAX_PROJECTION_QUERIES,
        "self-hosted projection work grew from the reviewed ceiling of \
         {MAX_PROJECTION_QUERIES} to {}; cache equivalent resolution work instead of \
         weakening analysis",
        analysis.projection_queries,
    );
}

#[test]
#[ignore = "manual wall-clock smoke; run through scripts/perf-smoke"]
fn warm_process_wall_clock_smoke_is_advisory() {
    let root = repository_root();
    let cold_started = Instant::now();
    let cold = build_lock(&root, "zrail.toml".as_ref()).expect("build cold lock candidate");
    let cold_elapsed = cold_started.elapsed();

    let warm_started = Instant::now();
    let warm = build_lock(&root, "zrail.toml".as_ref()).expect("build warm lock candidate");
    let warm_elapsed = warm_started.elapsed();
    let analysis = warm.analysis.as_ref().expect("analysis certificate");

    assert_eq!(cold, warm, "warm analysis changed architectural truth");
    write_smoke_result(
        cold_elapsed,
        warm_elapsed,
        analysis.projection_queries,
        analysis.projected_facts,
    );
}

fn write_smoke_result(cold: Duration, warm: Duration, queries: usize, facts: usize) {
    let mut result = String::from("zrail perf smoke: cold_seconds=");
    result.push_str(&cold.as_secs_f64().to_string());
    result.push_str(" warm_seconds=");
    result.push_str(&warm.as_secs_f64().to_string());
    result.push_str(" projection_queries=");
    result.push_str(&queries.to_string());
    result.push_str(" projected_facts=");
    result.push_str(&facts.to_string());
    result.push('\n');
    io::stderr()
        .write_all(result.as_bytes())
        .expect("write advisory performance result");
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve zrail repository root")
}
