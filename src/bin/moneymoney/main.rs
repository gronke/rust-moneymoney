//! Command-line interface to MoneyMoney (macOS).
//!
//! The binary is split into one module per verb (`export`, `add`, `set`,
//! `create`) plus three shared concerns (`output`, `batch`, `filters`).
//! `main.rs` itself only wires the top-level [`Cli`] / [`Cmd`] tree to
//! each module's `run` entry point.
//!
//! Build with `--features experimental` to enable `create bank-transfer`
//! and `create direct-debit`.

use clap::{CommandFactory, Parser, Subcommand};

mod add;
mod batch;
#[cfg(feature = "experimental")]
mod create;
mod export;
mod filters;
mod output;
mod set;

#[cfg(test)]
mod tests;

/// Crate-wide boxed error alias for fallible CLI helpers.
pub(crate) type BoxedErr = Box<dyn std::error::Error + Send + Sync>;

#[derive(Parser)]
#[clap(
    name = "moneymoney",
    version,
    about = "Talk to the MoneyMoney app from the terminal",
    long_about = "Talk to the MoneyMoney app from the terminal.

Subcommands group by AppleScript verb: `export` (read-only), `add` and `set` \
(stable mutations), and `create` (experimental SEPA payments). Every write \
verb accepts the same `[FILE...] [--dry-run] [--skip N] [--skip-error] \
[--skip-duplicates]` tail for consistent batch ergonomics."
)]
pub(crate) struct Cli {
    /// Output serialization format (`json` by default; more formats may follow)
    #[clap(
        long,
        value_enum,
        default_value_t = output::OutputFormat::Json,
        global = true,
        long_help = "Serialization format for any stdout output. Currently only `json`; a \
                     human-readable `table` variant lands in a follow-up PR."
    )]
    pub format: output::OutputFormat,

    /// Disable ANSI colors in CLI output (also honors the `NO_COLOR` env var)
    #[clap(long, global = true)]
    pub no_color: bool,

    #[clap(subcommand)]
    pub command: Cmd,
}

#[derive(Subcommand)]
pub(crate) enum Cmd {
    /// Read data from MoneyMoney
    Export {
        #[clap(subcommand)]
        target: export::ExportTarget,
    },
    /// Add new records (e.g. offline-account transactions)
    Add {
        #[clap(subcommand)]
        target: add::AddTarget,
    },
    /// Modify existing records (e.g. transaction checkmark / category / comment)
    Set {
        #[clap(subcommand)]
        target: set::SetTarget,
    },
    /// Create payments (requires building with `--features experimental`)
    #[cfg(feature = "experimental")]
    Create {
        #[clap(subcommand)]
        target: create::CreateTarget,
    },
}

/// Print the rendered `--help` text for `Cli` walked down `path` (e.g.
/// `&["add", "transaction"]`) to stdout, then exit non-zero. Used by the
/// write verbs when invoked with no field flags and no positional file —
/// matches the user expectation that `moneymoney add transaction` (bare)
/// shows help instead of a generic error.
pub(crate) fn print_help_and_exit(path: &[&str]) -> ! {
    let mut cmd = Cli::command();
    for name in path {
        cmd = cmd
            .find_subcommand(name)
            .expect("subcommand should be registered")
            .clone();
    }
    let full = std::iter::once("moneymoney")
        .chain(path.iter().copied())
        .collect::<Vec<_>>()
        .join(" ");
    let _ = cmd.bin_name(full).print_help();
    std::process::exit(1);
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        let color = output::use_color(false);
        eprintln!("{}: {e}", output::red("error", color));
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), BoxedErr> {
    let format = cli.format;
    let color = output::use_color(cli.no_color);
    match cli.command {
        Cmd::Export { target } => export::run(target, format),
        Cmd::Add {
            target: add::AddTarget::Transaction(args),
        } => add::run(args, format, color),
        Cmd::Set {
            target: set::SetTarget::Transaction(args),
        } => set::run(args, format, color),
        #[cfg(feature = "experimental")]
        Cmd::Create { target } => create::run(target, format, color),
    }
}
