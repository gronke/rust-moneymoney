//! Output formatting and color helpers shared by every subcommand.

use std::io::{self, IsTerminal, Write};

use clap::ValueEnum;
use serde::Serialize;

use crate::BoxedErr;

/// Output encoding for any stdout output produced by the CLI.
///
/// JSON is the only variant today; a `Table` variant (rendered via the
/// [`tabled`] crate) is planned for a follow-up PR. The enum lives here so
/// every subcommand routes its output through one consistent layer.
///
/// [`tabled`]: https://docs.rs/tabled
#[derive(ValueEnum, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum OutputFormat {
    /// Pretty-printed JSON (default)
    Json,
}

pub(crate) fn write_json_pretty_stdout<T: Serialize>(value: &T) -> Result<(), BoxedErr> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, value)?;
    stdout.write_all(b"\n")?;
    Ok(())
}

const ANSI_RED: &str = "\x1b[31m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_RESET: &str = "\x1b[0m";

/// Whether ANSI color should be emitted. Respects `--no-color`,
/// the `NO_COLOR` env var, and whether stderr is a terminal.
pub(crate) fn use_color(no_color: bool) -> bool {
    !no_color && std::env::var_os("NO_COLOR").is_none() && io::stderr().is_terminal()
}

fn paint(s: &str, code: &str, on: bool) -> String {
    if on {
        format!("{code}{s}{ANSI_RESET}")
    } else {
        s.to_string()
    }
}

pub(crate) fn red(s: &str, on: bool) -> String {
    paint(s, ANSI_RED, on)
}

pub(crate) fn green(s: &str, on: bool) -> String {
    paint(s, ANSI_GREEN, on)
}

pub(crate) fn yellow(s: &str, on: bool) -> String {
    paint(s, ANSI_YELLOW, on)
}
