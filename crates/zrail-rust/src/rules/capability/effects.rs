//! Effect vocabulary maps normalized source facts to one stable diagnostic.

use zrail_core::{Effect, Finding};

use crate::source::{ObservedFact, RustFileFacts};

pub(super) fn finding(
    file: &RustFileFacts,
    path: &ObservedFact,
    profile: &str,
    effect: Effect,
) -> Finding {
    Finding::error(
        "EFFECT-001",
        format!("profile.{profile}"),
        "effect",
        format!(
            "profile {profile:?} denies {effect:?}, provided here by {}",
            path.name
        ),
    )
    .at(&file.relative, path.span)
    .with_analysis(path.quality)
    .with_help("acquire the effect in an outer adapter and inject an explicit capability")
}

pub(crate) const fn tokens(effect: Effect) -> &'static [&'static str] {
    match effect {
        Effect::Filesystem => &["std::fs"],
        Effect::CompileFilesystem | Effect::CompileEnvironment => &[],
        Effect::Network => &["std::net", "std::os::unix::net"],
        Effect::Process => &["std::process", "tokio::process"],
        Effect::Synchronization => &["std::sync", "tokio::sync"],
        Effect::Thread => &["std::thread"],
        Effect::WallClock => &["std::time::Instant", "std::time::SystemTime"],
        Effect::AsyncRuntime => &["tokio", "async_std", "smol"],
        Effect::Database => &["sqlx", "diesel", "rusqlite"],
        Effect::ContainerRuntime => &["bollard", "containerd_client", "docker_api"],
        Effect::Environment => &["std::env"],
        Effect::Randomness => &["rand", "getrandom"],
    }
}
