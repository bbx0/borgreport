// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use clap::{
    builder::{NonEmptyStringValueParser, Styles},
    command, value_parser, ArgMatches, Command, CommandFactory, FromArgMatches, Parser, ValueHint,
};
use constcat::concat;

/// Clap Argument IDs are used as environment variable names.
/// Some IDs MUST NOT have a clap env mapping.
/// This allows a soft override of defaults via the Environment and to force an override via cli option.
pub(crate) mod args {
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
    pub const BORG_BINARY: &str = "BORGREPORT_BORG_BINARY";
    pub const MAX_AGE_HOURS: &str = "BORGREPORT_MAX_AGE_HOURS";

    // Not used as env var
    pub const HELP2MAN: &str = "__HELP2MAN";
}

pub(crate) mod long_help {
    //Clap processes the ENV
    pub const ENV_DIR: &str =
        "Directory to look for *.env files containing BORG_* env vars for a repository. Each file name represents a repository name in the report.";
    pub const ENV_INHERIT: &str = "Inherit BORG_* env vars for a single <REPOSITORY>. This allows to run `borgreport` after `borg` while reusing the environment.";
    pub const MAILTOADDR: &str =
        "Send the report to <ADDR> using a 'sendmail' compatible mail transfer agent.";
    pub const MAILFROMADDR: &str =
        "The mail sender <ADDR>. By default this is the current user@host";
    pub const NOPROGRESS: &str =
        "Suppress all status updates during processing. By default this is auto-detected.";
    pub const TEXTFILE: &str = "Write the text report to <FILE> instead of stdout.";
    pub const HTMLFILE: &str = "Write the HTML report to <FILE>.";
    pub const METRICSFILE: &str = "Write metrics to <FILE>.";

    // Clap ignores the ENV
    pub const GLOB_ARCHIVES: &str =
        "A list of space separated archive globs e.g. \"etc-* srv-*\" for archive names starting with etc- or srv-. (Default: \"\")";
    pub const CHECK: &str = "Enables the execution of `borg check`. (Default: false)";
    pub const BORG_BINARY: &str = "Path to a local 'borg' binary. (Default: borg)";
    pub const MAX_AGE_HOURS: &str =
        "Threshold to warn, when the last backup is older than <HOURS>. (Default: 24)";
}

/// Additional --help-man output for generating a manpage with help2man
pub const HELP2MAN: &str = concat!("Environment:
Environment variables are overwritten by the respective command line option.
  ",args::ENV_DIR," <DIR>  ", long_help::ENV_DIR,"
  ",args::ENV_INHERIT," <REPOSITORY>  ", long_help::ENV_INHERIT,"
  ",args::MAILTOADDR," <ADDR>  ", long_help::MAILTOADDR,"
  ",args::MAILFROMADDR," <ADDR>  ", long_help::MAILFROMADDR,"
  ",args::NOPROGRESS," <ADDR>  ", long_help::NOPROGRESS,"
  ",args::TEXTFILE," <FILE>  ", long_help::TEXTFILE,"
  ",args::HTMLFILE," <FORMAT>  ", long_help::HTMLFILE,"
  ",args::METRICSFILE," <FILE>  ", long_help::METRICSFILE,"

Repository Environment:
  !  You probably want to configure the following variables at repository level. Setting them globally will alter the default behavior for all repositories.
  ",args::GLOB_ARCHIVES," <GLOB>  ", long_help::GLOB_ARCHIVES,"
  ",args::CHECK," <true|false>  ", long_help::CHECK,"
  ",args::BORG_BINARY," <FILE>  ", long_help::BORG_BINARY,"
  ",args::MAX_AGE_HOURS," <HOURS>  ", long_help::MAX_AGE_HOURS,"

Report bugs to <https://github.com/bbx0/borgreport/issues>."
);

/// Print help message with more details for help2man
pub(crate) fn print_help2man() -> Result<()> {
    command()
        .after_long_help(HELP2MAN)
        .styles(Styles::plain())
        .print_long_help()?;
    Ok(())
}

/// Extended --version output for generating a manpage with help2man
const LONG_VERSION: &str = concat!(
    clap::crate_version!(),
    "

Copyright (C) 2024 Philipp Micheel
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>
This is free software; you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.

Written by ",
    clap::crate_authors!()
);

/// Raw access to the `ArgMatches` for Key/Value access.
static MATCHES: std::sync::OnceLock<ArgMatches> = std::sync::OnceLock::new();
/// Accessor function to command line arguments.
pub(crate) fn matches() -> &'static ArgMatches {
    MATCHES.get_or_init(|| command().get_matches())
}

/// Structured access to the `ArgMatches`
static ARGS: std::sync::OnceLock<Args> = std::sync::OnceLock::new();
/// Accessor function to command line arguments.
pub(crate) fn args() -> &'static Args {
    //ARGS.get_or_init(|| Args::parse())
    ARGS.get_or_init(|| {
        Args::from_arg_matches(matches()).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1)
        })
    })
}

/// Command Builder
pub(crate) fn command() -> Command {
    Args::command()
}

/// Command line interface
#[derive(Parser, Debug, Clone)]
#[command(
    about,
    after_long_help = "See `man 1 borgreport` for more help.",
    arg_required_else_help = true,
    author,
    long_about = None,
    long_version = LONG_VERSION,
    version,
    )]
pub(crate) struct Args {
    #[arg(
        action = clap::ArgAction::Append,
        env = args::ENV_DIR,
        help = "Directory to look for *.env files containing BORG_* variables for a repository.",
        hide_env = true,
        id = args::ENV_DIR,
        long = "env-dir",
        long_help = long_help::ENV_DIR,
        required_unless_present_any = [args::ENV_INHERIT, args::HELP2MAN],
        value_hint = ValueHint::DirPath,
        value_name = "DIR",
        value_parser = value_parser!(std::path::PathBuf),
    )]
    pub(crate) env_dirs: Vec<std::path::PathBuf>,

    #[arg(
        action = clap::ArgAction::Set,
        env = args::ENV_INHERIT,
        hide_env = true,
        help = "Inherit BORG_* variables for a single <REPOSITORY> name from the current environment.",
        long_help = long_help::ENV_INHERIT,
        id = args::ENV_INHERIT,
        long = "env-inherit",
        value_hint = ValueHint::Other,
        value_name = "REPOSITORY",
        value_parser = NonEmptyStringValueParser::new(),
    )]
    pub(crate) env_inherit: Option<String>,

    #[arg(
        action = clap::ArgAction::Set,
        env = args::TEXTFILE,
        help = long_help::TEXTFILE,
        hide_env = true,
        id = args::TEXTFILE,
        long = "text-to",
        long_help = long_help::TEXTFILE,
        value_hint = ValueHint::FilePath,
        value_name = "FILE",
        value_parser = value_parser!(std::path::PathBuf),
    )]
    pub(crate) text_file: Option<std::path::PathBuf>,

    #[arg(
        action = clap::ArgAction::Set,
        env = args::HTMLFILE,
        help = long_help::HTMLFILE,
        hide_env = true,
        id = args::HTMLFILE,
        long = "html-to",
        long_help = long_help::HTMLFILE,
        value_hint = ValueHint::FilePath,
        value_name = "FILE",
        value_parser = value_parser!(std::path::PathBuf),
    )]
    pub(crate) html_file: Option<std::path::PathBuf>,

    #[arg(
        action = clap::ArgAction::Set,
        env = args::METRICSFILE,
        help = long_help::METRICSFILE,
        hide_env = true,
        id = args::METRICSFILE,
        long = "metrics-to",
        long_help = long_help::METRICSFILE,
        value_hint = ValueHint::FilePath,
        value_name = "FILE",
        value_parser = value_parser!(std::path::PathBuf),
    )]
    pub(crate) metrics_file: Option<std::path::PathBuf>,

    #[arg(
        action = clap::ArgAction::Set,
        env = args::MAILTOADDR,
        help = "Send the report to <ADDR> via `sendmail`",
        hide_env = true,
        id = args::MAILTOADDR,
        long = "mail-to",
        long_help = long_help::MAILTOADDR,
        value_hint = ValueHint::EmailAddress,
        value_name = "ADDR",
        value_parser = value_parser!(lettre::Address),
    )]
    pub(crate) mail_to: Option<lettre::Address>,

    #[arg(
        action = clap::ArgAction::Set,
        env = args::MAILFROMADDR,
        help = "Send the report from <ADDR> instead of a default",
        hide_env = true,
        id = args::MAILFROMADDR,
        long = "mail-from",
        long_help = long_help::MAILFROMADDR,
        requires = args::MAILTOADDR,
        value_hint = ValueHint::EmailAddress,
        value_name = "ADDR",
        value_parser = value_parser!(lettre::Address),
    )]
    pub(crate) mail_from: Option<lettre::Address>,

    #[arg(
        action = clap::ArgAction::SetTrue,
        env = args::NOPROGRESS,
        hide_env = true,
        help = "Suppress all status updates during processing.",
        long_help = long_help::NOPROGRESS,
        id = args::NOPROGRESS,
        long = "no-progress",
    )]
    pub(crate) no_progress: bool,

    #[arg(
        action = clap::ArgAction::Set,
        help = "Enforce a glob archives filter for all repositories.",
        help_heading = "Override repository options",
        id = args::GLOB_ARCHIVES,
        long = "glob-archives",
        long_help = long_help::GLOB_ARCHIVES,
        value_hint = ValueHint::Other,
        value_name = "GLOB",
        value_parser = value_parser!(String),
    )]
    pub(crate) glob_archives: Option<String>,

    // Note: `ArgAction::SetTrue` will cause `Arg::default_value` = `false` but we need `None` when the flag is not present. -> use default_missing_value
    #[arg(
        action = clap::ArgAction::Set,
        default_missing_value = "true",
        help = "Enforce to run (or not run) `borg check`",
        help_heading = "Override repository options",
        id = args::CHECK,
        long = "check",
        long_help = long_help::CHECK,
        num_args = 0..=1,
        require_equals = true,
        hide_possible_values = true,
        value_hint = ValueHint::Other,
        value_name = "true|false",
        value_parser = value_parser!(bool),
    )]
    pub(crate) no_check: Option<bool>,

    #[arg(
        action = clap::ArgAction::Set,
        help = "Local path to a specific 'borg' binary",
        help_heading = "Override repository options",
        id = args::BORG_BINARY,
        long = "borg-binary",
        long_help = long_help::BORG_BINARY,
        value_hint = ValueHint::FilePath,
        value_name = "FILE",
        value_parser = value_parser!(std::path::PathBuf),
        )]
    pub(crate) borg_binary: Option<std::path::PathBuf>,

    #[arg(
        action = clap::ArgAction::Set,
        help = "Threshold to warn when the last archive is older than <HOURS>",
        help_heading = "Override repository options",
        id = args::MAX_AGE_HOURS,
        long = "max-age-hours",
        long_help = long_help::MAX_AGE_HOURS,
        value_hint = ValueHint::Other,
        value_name = "HOURS",
        value_parser = value_parser!(f64),
    )]
    pub(crate) max_age_hours: Option<f64>,

    #[arg(
        action = clap::ArgAction::SetTrue,
        exclusive = true,
        hide = true,
        id = args::HELP2MAN,
        long = "help-man",
    )]
    /// Print an extended help message as input for `help2man`
    pub(crate) print_help2man: bool,
}
