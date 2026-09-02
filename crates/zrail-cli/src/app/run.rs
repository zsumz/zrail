//! Top-level command dispatch and process I/O.

use std::{
    ffi::OsString,
    io::{self, Write},
};

use super::{
    args::{Command, parse},
    commands,
    error::CliError,
    output::{OutputFormat, render_error},
};

const HELP: &str = concat!(
    "zrail — executable guardrails for human- and agent-written code\n\n",
    "USAGE\n",
    "  zrail init [ROOT] [--preset zsumz|rust] [--exclude PATTERN] [--exclude-from FILE] [--baseline]\n",
    "  zrail check [--root ROOT] [--format human|json] [--max-findings N|all]\n",
    "  zrail coverage [--root ROOT] [--config PATH] [--format human|json]\n",
    "  zrail mirrors plan [--root ROOT] [--config PATH] [--format human|json]\n",
    "  zrail mirrors receipts --plan PATH --results PATH [--root ROOT] [--config PATH] [--format human|json]\n",
    "  zrail mirrors verify --plan PATH [--root ROOT] [--config PATH] [--format human|json]\n",
    "  zrail baseline [--root ROOT] [--rule RULE] [--dry-run] [--format human|json] [--accept-grants]\n",
    "  zrail update [--base REVISION] [--root ROOT] [--format human|json] [--accept-migration sha256:DIGEST] [--migration-report PATH] [--accept-grants]\n",
    "  zrail doctor [--root ROOT] [--format human|json]\n",
    "  zrail explain (--path PATH | --hypothetical-path PATH) [--root ROOT] [--format human|json]\n",
    "  zrail review [--base REVISION] [--authority-root ROOT] --root PROPOSAL [--allow-grants] [--max-findings N|all]\n",
    "  zrail diff --base REVISION [--root ROOT] [--deny-grants]\n",
    "  zrail diff --before ROOT --after ROOT [--deny-grants]\n\n",
    "  zrail migrate-config [--root ROOT] [--config PATH] [--write]\n",
    "  zrail migrate-lock [--base REVISION] [--target REVISION] --output PATH [--root ROOT]\n",
    "  zrail migrate-lock --discover-base [--root ROOT] [--config PATH] [--lock PATH]\n",
    "  zrail fmt [--root ROOT] [--config PATH] [--check]\n\n",
    "MODEL\n",
    "  zrail.toml   human architectural intent\n",
    "  zrail.lock   resolved exact state and ratchets\n",
    "  zrail        check, explain, and review architecture\n",
);

pub(crate) fn run(arguments: impl IntoIterator<Item = OsString>) -> i32 {
    match parse(arguments) {
        Ok(command) => run_command(&command),
        Err(error) => write_error(&error, OutputFormat::Human),
    }
}

fn run_command(command: &Command) -> i32 {
    let result = match command {
        Command::Check(options) => commands::check(options),
        Command::Coverage(options) => commands::coverage(options),
        Command::Doctor(options) => commands::doctor(options),
        Command::Baseline(options) => commands::baseline(options),
        Command::Update(options) => commands::update(options),
        Command::Explain {
            common,
            path,
            hypothetical,
        } => commands::explain(common, path, *hypothetical),
        Command::Diff(options) => commands::diff(options),
        Command::Review(options) => commands::review(options),
        Command::Init(options) => commands::init(options),
        Command::MigrateConfig(options) => commands::migrate_config(options),
        Command::MigrateLock(options) => commands::migrate_lock(options),
        Command::Mirrors(options) => commands::mirrors(options),
        Command::Fmt(options) => commands::format_config(options),
        Command::Help => return write_stdout(HELP, 0),
        Command::Version => {
            return write_stdout(concat!("zrail ", env!("CARGO_PKG_VERSION"), "\n"), 0);
        }
    };
    match result {
        Ok(output) => write_stdout(&output.text, output.exit_code),
        Err(error) => write_error(&error, command_format(command)),
    }
}

fn command_format(command: &Command) -> OutputFormat {
    match command {
        Command::Check(options) | Command::Doctor(options) => options.format,
        Command::Coverage(options) => options.format,
        Command::Mirrors(options) => options.format,
        Command::Update(options) => options.common.format,
        Command::Baseline(options) => options.common.format,
        Command::Explain { common, .. } => common.format,
        Command::Diff(options) => options.format,
        Command::Review(options) => options.common.format,
        Command::Init(_)
        | Command::MigrateConfig(_)
        | Command::MigrateLock(_)
        | Command::Fmt(_)
        | Command::Help
        | Command::Version => OutputFormat::Human,
    }
}

fn write_stdout(text: &str, exit_code: i32) -> i32 {
    let mut output = io::stdout().lock();
    if output.write_all(text.as_bytes()).is_err() {
        return 2;
    }
    exit_code
}

fn write_error(error: &CliError, format: OutputFormat) -> i32 {
    let text = render_error(error, format);
    let mut output = io::stderr().lock();
    if output.write_all(text.as_bytes()).is_err() {
        return 2;
    }
    2
}
