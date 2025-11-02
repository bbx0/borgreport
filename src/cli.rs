// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::{
    ArgMatches, Command, CommandFactory, FromArgMatches, Parser, ValueHint,
    builder::NonEmptyStringValueParser, value_parser,
};
use email_address::EmailAddress;

/// Clap Argument IDs are used as environment variable names.
/// Some IDs MUST NOT have a clap env mapping.
/// This allows a soft override of defaults via the Environment and to force an override via cli option.
pub mod args {
    //Clap processes option and ENV
    pub const ENV_DIR: &str = "BORGREPORT_ENV_DIR";
    pub const ENV_INHERIT: &str = "BORGREPORT_ENV_INHERIT";
    pub const MAILTOADDR: &str = "BORGREPORT_MAIL_TO";
    pub const MAILFROMADDR: &str = "BORGREPORT_MAIL_FROM";
    pub const NOPROGRESS: &str = "BORGREPORT_NO_PROGRESS";
    pub const TEXTFILE: &str = "BORGREPORT_TEXT_TO";
    pub const HTMLFILE: &str = "BORGREPORT_HTML_TO";
    pub const METRICSFILE: &str = "BORGREPORT_METRICS_TO";

    // Clap ignores the ENV (soft override at repository level allowed)
    pub const GLOB_ARCHIVES: &str = "BORGREPORT_GLOB_ARCHIVES";
    pub const CHECK: &str = "BORGREPORT_CHECK";
    pub const CHECK_OPTIONS: &str = "BORGREPORT_CHECK_OPTIONS";
    pub const COMPACT: &str = "BORGREPORT_COMPACT";
    pub const COMPACT_OPTIONS: &str = "BORGREPORT_COMPACT_OPTIONS";
    pub const BORG_BINARY: &str = "BORGREPORT_BORG_BINARY";
    pub const MAX_AGE_HOURS: &str = "BORGREPORT_MAX_AGE_HOURS";
}

/// Extended --version output for generating a manpage with help2man
const LONG_VERSION: &str = concat!(
    clap::crate_version!(),
    "

Copyright (C) 2024 Philipp Micheel.
License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>.
There is ABSOLUTELY NO WARRANTY, to the extent permitted by law.

Written by ",
    clap::crate_authors!()
);

/// Raw access to the `ArgMatches` for Key/Value access.
static MATCHES: std::sync::OnceLock<ArgMatches> = std::sync::OnceLock::new();
/// Accessor function to command line arguments.
pub fn matches() -> &'static ArgMatches {
    MATCHES.get_or_init(|| command().get_matches())
}

/// Structured access to the `ArgMatches`
static ARGS: std::sync::OnceLock<Args> = std::sync::OnceLock::new();
/// Accessor function to command line arguments.
pub fn args() -> &'static Args {
    //ARGS.get_or_init(|| Args::parse())
    ARGS.get_or_init(|| {
        Args::from_arg_matches(matches()).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1)
        })
    })
}

/// Command Builder
pub fn command() -> Command {
    Args::command()
}

/// Command line interface
#[derive(Parser, Debug, Clone)]
#[command(
    about,
    after_long_help = "See `man 1 borgreport` for more help.",
    author,
    long_about = "A wrapper for BorgBackup to query the latest backup archives and perform health checks on repositories. It summarize the status of BorgBackup repositories with statistics, warnings and error messages. You can save the report as file or send it per mail and export OpenMetrics (Prometheus Metrics) for the last archives.",
    long_version = LONG_VERSION,
    version,
    )]
pub struct Args {
    #[arg(
        action = clap::ArgAction::Append,
        env = args::ENV_DIR,
        help = "Directory to look for *.env files containing BORG_* variables for a repository.",
        hide_env = true,
        id = args::ENV_DIR,
        long = "env-dir",
        long_help = "Directory to look for *.env files containing BORG_* env vars for a repository. Each file name represents a repository name in the report.",
        value_hint = ValueHint::DirPath,
        value_name = "PATH",
        value_parser = value_parser!(std::path::PathBuf),
    )]
    pub env_dirs: Vec<std::path::PathBuf>,

    #[arg(
        action = clap::ArgAction::Set,
        env = args::ENV_INHERIT,
        hide_env = true,
        help = "Inherit BORG_* variables for a single <REPOSITORY> name from the current environment.",
        long_help = "Inherit BORG_* env vars for a single <REPOSITORY>. This allows to run `borgreport` after `borg` while reusing the environment.",
        id = args::ENV_INHERIT,
        long = "env-inherit",
        value_hint = ValueHint::Other,
        value_name = "REPOSITORY",
        value_parser = NonEmptyStringValueParser::new(),
    )]
    pub env_inherit: Option<String>,

    #[arg(
        action = clap::ArgAction::Set,
        env = args::TEXTFILE,
        help = "Write the text report to <FILE> instead of stdout.",
        hide_env = true,
        id = args::TEXTFILE,
        long = "text-to",
        value_hint = ValueHint::FilePath,
        value_name = "FILE",
        value_parser = value_parser!(std::path::PathBuf),
    )]
    pub text_file: Option<std::path::PathBuf>,

    #[arg(
        action = clap::ArgAction::Set,
        env = args::HTMLFILE,
        help = "Write the HTML report to <FILE>.",
        hide_env = true,
        id = args::HTMLFILE,
        long = "html-to",
        value_hint = ValueHint::FilePath,
        value_name = "FILE",
        value_parser = value_parser!(std::path::PathBuf),
    )]
    pub html_file: Option<std::path::PathBuf>,

    #[arg(
        action = clap::ArgAction::Set,
        env = args::METRICSFILE,
        help = "Write metrics to <FILE>.",
        hide_env = true,
        id = args::METRICSFILE,
        long = "metrics-to",
        value_hint = ValueHint::FilePath,
        value_name = "FILE",
        value_parser = value_parser!(std::path::PathBuf),
    )]
    pub metrics_file: Option<std::path::PathBuf>,

    #[arg(
        action = clap::ArgAction::Set,
        env = args::MAILTOADDR,
        help = "Send the report to <ADDR> via `sendmail`",
        hide_env = true,
        id = args::MAILTOADDR,
        long = "mail-to",
        long_help = "Send the report to <ADDR> using a 'sendmail' compatible mail transfer agent.",
        value_hint = ValueHint::EmailAddress,
        value_name = "ADDR",
        value_parser = value_parser!(email_address::EmailAddress),
    )]
    pub mail_to: Option<email_address::EmailAddress>,

    #[arg(
        action = clap::ArgAction::Set,
        env = args::MAILFROMADDR,
        help = "Send the report from <ADDR> instead of a default",
        hide_env = true,
        id = args::MAILFROMADDR,
        long = "mail-from",
        long_help = "The mail sender <ADDR>. By default this is the current user@host",
        requires = args::MAILTOADDR,
        value_hint = ValueHint::EmailAddress,
        value_name = "ADDR",
        value_parser = value_parser!(EmailAddress),
    )]
    pub mail_from: Option<EmailAddress>,

    #[arg(
        action = clap::ArgAction::SetTrue,
        env = args::NOPROGRESS,
        hide_env = true,
        help = "Suppress all status updates during processing.",
        long_help = "Suppress all status updates during processing. By default this is auto-detected.",
        id = args::NOPROGRESS,
        long = "no-progress",
    )]
    pub no_progress: bool,

    #[arg(
        action = clap::ArgAction::Set,
        help = "Enforce a glob archives filter for all repositories.",
        help_heading = "Override repository options",
        id = args::GLOB_ARCHIVES,
        long = "glob-archives",
        long_help = "A list of space separated archive globs e.g. \"etc-* srv-*\" for archive names starting with etc- or srv-. (Default: \"\")",
        value_hint = ValueHint::Other,
        value_name = "GLOB",
        value_parser = value_parser!(String),
    )]
    pub glob_archives: Option<String>,

    // Note: `ArgAction::SetTrue` will cause `Arg::default_value` = `false` but we need `None` when the flag is not present. -> use default_missing_value
    #[arg(
        action = clap::ArgAction::Set,
        default_missing_value = "true",
        help = "Enforce to run (or not run) `borg check`",
        help_heading = "Override repository options",
        id = args::CHECK,
        long = "check",
        long_help = "Enables the execution of `borg check`. (Default: false)",
        num_args = 0..=1,
        require_equals = true,
        hide_possible_values = true,
        value_hint = ValueHint::Other,
        value_name = "true|false",
        value_parser = value_parser!(bool),
    )]
    pub check: Option<bool>,

    #[arg(
        action = clap::ArgAction::Set,
        help = "Enforce override of raw `borg check` options for all repositories.",
        help_heading = "Override repository options",
        id = args::CHECK_OPTIONS,
        long = "check-options",
        long_help = "A list of space separated raw borg options supplied to the `borg check` command",
        value_hint = ValueHint::Other,
        value_name = "OPTS",
        value_parser = value_parser!(String),
    )]
    pub check_opts: Option<String>,

    // Note: `ArgAction::SetTrue` will cause `Arg::default_value` = `false` but we need `None` when the flag is not present. -> use default_missing_value
    #[arg(
        action = clap::ArgAction::Set,
        default_missing_value = "true",
        help = "Enforce to run (or not run) `borg compact`",
        help_heading = "Override repository options",
        id = args::COMPACT,
        long = "compact",
        long_help = "Enables the execution of `borg compact`. (Default: false)",
        num_args = 0..=1,
        require_equals = true,
        hide_possible_values = true,
        value_hint = ValueHint::Other,
        value_name = "true|false",
        value_parser = value_parser!(bool),
    )]
    pub compact: Option<bool>,

    #[arg(
        action = clap::ArgAction::Set,
        help = "Enforce override of raw `borg compact` options for all repositories.",
        help_heading = "Override repository options",
        id = args::COMPACT_OPTIONS,
        long = "compact-options",
        long_help = "A list of space separated raw borg options supplied to the `borg compact` command",
        value_hint = ValueHint::Other,
        value_name = "OPTS",
        value_parser = value_parser!(String),
    )]
    pub compact_opts: Option<String>,

    #[arg(
        action = clap::ArgAction::Set,
        help = "Local path to a specific 'borg' binary",
        help_heading = "Override repository options",
        id = args::BORG_BINARY,
        long = "borg-binary",
        long_help = "Path to a local 'borg' binary. (Default: borg)",
        value_hint = ValueHint::FilePath,
        value_name = "FILE",
        value_parser = value_parser!(std::path::PathBuf),
        )]
    pub borg_binary: Option<std::path::PathBuf>,

    #[arg(
        action = clap::ArgAction::Set,
        help = "Threshold to warn when the last archive is older than <HOURS>",
        help_heading = "Override repository options",
        id = args::MAX_AGE_HOURS,
        long = "max-age-hours",
        long_help = "Threshold to warn, when the last backup is older than <HOURS>. (Default: 24)",
        value_hint = ValueHint::Other,
        value_name = "HOURS",
        value_parser = value_parser!(f64),
    )]
    pub max_age_hours: Option<f64>,
}
