//! Reusable clap argument groups that several subcommands compose via
//! `#[clap(flatten)]` to keep the CLI surface consistent.

use clap::Args;

/// `--include-groups` and `--include-icon-data` toggles shared by every
/// "list of records with optional groupings and icons" export
/// (currently `accounts` and `categories`).
#[derive(Args, Clone, Debug)]
pub(crate) struct IconGroupOptions {
    /// Include per-item icon bytes (omitted by default)
    #[clap(
        long = "include-icon-data",
        long_help = "Include the `icon` field (raw image bytes) for each item. Omitted by default \
                     because payloads are large."
    )]
    pub include_icon_data: bool,

    /// Include groups / folders (omitted by default; only leaf items are exported)
    #[clap(
        long = "include-groups",
        long_help = "Include groups (`group: true`) in the output. Omitted by default; only leaf \
                     items (real accounts / real categories) are exported."
    )]
    pub include_groups: bool,
}

/// `--from-account` filter shared by `export transactions` and
/// `export portfolio`.
#[derive(Args, Clone, Debug)]
pub(crate) struct AccountFilter {
    /// Restrict to one account (UUID or IBAN)
    #[clap(
        long = "from-account",
        value_name = "UUID|IBAN",
        long_help = "Only return rows associated with this account. Accepts a MoneyMoney account \
                     UUID or IBAN. When omitted, all accounts are included."
    )]
    pub from_account: Option<String>,
}
