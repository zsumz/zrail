//! `zrail` command-line entry point.

mod app;

fn main() {
    std::process::exit(app::run(std::env::args_os()));
}
