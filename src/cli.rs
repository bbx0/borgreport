// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Result, anyhow};
use clap::{
    Arg, ArgAction, Command, ValueHint,
    builder::{NonEmptyStringValueParser, ValueParser},
};
use email_address::EmailAddress;
use std::{path::PathBuf, str::FromStr, sync::LazyLock};

static COMMAND: LazyLock<Command> = LazyLock::new(command_builder);
static MATCHES: LazyLock<clap::ArgMatches> = LazyLock::new(|| COMMAND.clone().get_matches());

static REPORT_ARGS: LazyLock<ReportArgs> = LazyLock::new(|| ReportArgs {
    env_dirs: MATCHES
        .get_many(args::report::ENV_DIR)
        .map(std::iter::Iterator::collect)
        .unwrap_or_default(),
    env_inherit: MATCHES.get_one(args::report::ENV_INHERIT),
    text_file: MATCHES.get_one(args::report::TEXTFILE),
    html_file: MATCHES.get_one(args::report::HTMLFILE),
    metrics_file: MATCHES.get_one(args::report::METRICSFILE),
    mail_to: MATCHES.get_one(args::report::MAILTOADDR),
    mail_from: MATCHES.get_one(args::report::MAILFROMADDR),
    no_progress: MATCHES.get_flag(args::report::NOPROGRESS),
});

static REPOSITORY_ARGS: LazyLock<RepositoryArgs> = LazyLock::new(|| RepositoryArgs {
    glob_archives: RepositoryArg::new(args::repository::GLOB_ARCHIVES),
    check: RepositoryArg::new(args::repository::CHECK),
    check_options: RepositoryArg::new(args::repository::CHECK_OPTIONS),
    compact: RepositoryArg::new(args::repository::COMPACT),
    compact_options: RepositoryArg::new(args::repository::COMPACT_OPTIONS),
    borg_binary: RepositoryArg::new(args::repository::BORG_BINARY),
    max_age_hours: RepositoryArg::new(args::repository::MAX_AGE_HOURS),
});

// Must match the command_builder() value parser types
pub struct ReportArgs {
    pub env_dirs: Vec<&'static PathBuf>,
    pub env_inherit: Option<&'static String>,
    pub text_file: Option<&'static PathBuf>,
    pub html_file: Option<&'static PathBuf>,
    pub metrics_file: Option<&'static PathBuf>,
    pub mail_to: Option<&'static EmailAddress>,
    pub mail_from: Option<&'static EmailAddress>,
    pub no_progress: bool,
}

/// Repository argument to override the Borg env
pub struct RepositoryArg<T: 'static> {
    id: &'static str,
    value: Option<&'static T>,
}
impl<T: std::any::Any + Clone + Send + Sync + 'static> RepositoryArg<T> {
    /// Panics if `T` does not match the `value_parser` for `id` in `command_builder()`.
    fn new(id: &'static str) -> Self {
        Self {
            id,
            value: MATCHES.get_one(id),
        }
    }
    pub const fn id(&self) -> &'static str {
        self.id
    }
    pub const fn value(&self) -> Option<&'static T> {
        self.value
    }

    /// Parse a `value` into the type of the Arg Id using the `clap::builder::ValueParser`
    pub fn parse(&self, value: impl Into<std::ffi::OsString> + Clone) -> Result<T> {
        parse_value(self.id, value)
    }
}

// Must match the command_builder() value parser types
pub struct RepositoryArgs {
    pub glob_archives: RepositoryArg<String>,
    pub check: RepositoryArg<bool>,
    pub check_options: RepositoryArg<String>,
    pub compact: RepositoryArg<bool>,
    pub compact_options: RepositoryArg<String>,
    pub borg_binary: RepositoryArg<PathBuf>,
    pub max_age_hours: RepositoryArg<f64>,
}

/// Args which control the report creation and override the borgreport process Environment
pub fn report_args() -> &'static ReportArgs {
    &REPORT_ARGS
}

/// Args which override the borg env repository configuration
pub fn repository_args() -> &'static RepositoryArgs {
    &REPOSITORY_ARGS
}

/// Clap Argument IDs are used as environment variable names.
/// This allows a soft override of defaults via the Environment and to force an override via cli option.
pub mod args {
    //Clap processes option and ENV
    pub(super) mod report {
        pub const ENV_DIR: &str = "BORGREPORT_ENV_DIR";
        pub const ENV_INHERIT: &str = "BORGREPORT_ENV_INHERIT";
        pub const MAILTOADDR: &str = "BORGREPORT_MAIL_TO";
        pub const MAILFROMADDR: &str = "BORGREPORT_MAIL_FROM";
        pub const NOPROGRESS: &str = "BORGREPORT_NO_PROGRESS";
        pub const TEXTFILE: &str = "BORGREPORT_TEXT_TO";
        pub const HTMLFILE: &str = "BORGREPORT_HTML_TO";
        pub const METRICSFILE: &str = "BORGREPORT_METRICS_TO";
    }

    // Clap ignores the ENV (soft override at repository level allowed)
    // These MUST NOT have a clap env binding (process environment)
    pub(super) mod repository {
        pub const GLOB_ARCHIVES: &str = "BORGREPORT_GLOB_ARCHIVES";
        pub const CHECK: &str = "BORGREPORT_CHECK";
        pub const CHECK_OPTIONS: &str = "BORGREPORT_CHECK_OPTIONS";
        pub const COMPACT: &str = "BORGREPORT_COMPACT";
        pub const COMPACT_OPTIONS: &str = "BORGREPORT_COMPACT_OPTIONS";
        pub const BORG_BINARY: &str = "BORGREPORT_BORG_BINARY";
        pub const MAX_AGE_HOURS: &str = "BORGREPORT_MAX_AGE_HOURS";
    }
}

/// Command Builder
#[allow(clippy::too_many_lines)]
pub fn command_builder() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .long_about(
            "A wrapper for BorgBackup to query the latest backup archives and perform health checks on repositories. It summarize the status of BorgBackup repositories with statistics, warnings and error messages. You can save the report as file or send it per mail and export OpenMetrics (Prometheus Metrics) for the last archives.",
        )
        .after_long_help("See `man 1 borgreport` for more help.")
        .version(env!("CARGO_PKG_VERSION"))
        .long_version(concat!(
            env!("CARGO_PKG_VERSION"),'\n',
            '\n',
            "Copyright (C) 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>\n",
            "License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>.\n",
            "There is ABSOLUTELY NO WARRANTY, to the extent permitted by law."
        ))
        .arg(
            Arg::new(args::report::ENV_DIR)
                .long("env-dir")
                .env(args::report::ENV_DIR)
                .hide_env(true)
                .action(ArgAction::Append)
                .help("Directory to look for *.env files containing BORG_* variables for a repository.")
                .long_help("Directory to look for *.env files containing BORG_* env vars for a repository. Each file name represents a repository name in the report.")
                .value_hint(ValueHint::DirPath)
                .value_name("PATH")
                .value_parser(ValueParser::path_buf()),
        )
        .arg(
            Arg::new(args::report::ENV_INHERIT)
                .long("env-inherit")
                .env(args::report::ENV_INHERIT)
                .hide_env(true)
                .action(ArgAction::Set)
                .help("Inherit BORG_* variables for a single <REPOSITORY> name from the current environment.")
                .long_help("Inherit BORG_* env vars for a single <REPOSITORY>. This allows to run `borgreport` after `borg` while reusing the environment.")
                .value_hint(ValueHint::Other)
                .value_name("REPOSITORY")
                .value_parser(NonEmptyStringValueParser::new()),
        )
        .arg(
            Arg::new(args::report::TEXTFILE)
                .long("text-to")
                .env(args::report::TEXTFILE)
                .hide_env(true)
                .action(ArgAction::Set)
                .help("Write the text report to <FILE> instead of stdout.")
                .value_hint(ValueHint::FilePath)
                .value_name("FILE")
                .value_parser(ValueParser::path_buf()),
        )
        .arg(
            Arg::new(args::report::HTMLFILE)
                .long("html-to")
                .env(args::report::HTMLFILE)
                .hide_env(true)
                .action(ArgAction::Set)
                .help("Write the HTML report to <FILE>.")
                .value_hint(ValueHint::FilePath)
                .value_name("FILE")
                .value_parser(ValueParser::path_buf()),
        )
        .arg(
            Arg::new(args::report::METRICSFILE)
                .long("metrics-to")
                .env(args::report::METRICSFILE)
                .hide_env(true)
                .action(ArgAction::Set)
                .help("Write metrics to <FILE>.")
                .value_hint(ValueHint::FilePath)
                .value_name("FILE")
                .value_parser(ValueParser::path_buf()),
        )
        .arg(
            Arg::new(args::report::MAILTOADDR)
                .long("mail-to")
                .env(args::report::MAILTOADDR)
                .hide_env(true)
                .action(ArgAction::Set)
                .help("Send the report to <ADDR> via `sendmail`")
                .long_help("Send the report to <ADDR> using a 'sendmail' compatible mail transfer agent.")
                .value_hint(ValueHint::EmailAddress)
                .value_name("ADDR")
                .value_parser(ValueParser::new(EmailAddress::from_str)),
        )
        .arg(
            Arg::new(args::report::MAILFROMADDR)
                .long("mail-from")
                .env(args::report::MAILFROMADDR)
                .hide_env(true)
                .action(ArgAction::Set)
                .help("Send the report from <ADDR> instead of a default")
                .long_help("The mail sender <ADDR>. By default this is the current user@host")
                .requires(args::report::MAILTOADDR)
                .value_hint(ValueHint::EmailAddress)
                .value_name("ADDR")
                .value_parser(ValueParser::new(EmailAddress::from_str)),
        )
        .arg(
            Arg::new(args::report::NOPROGRESS)
                .long("no-progress")
                .env(args::report::NOPROGRESS)
                .hide_env(true)
                .action(ArgAction::SetTrue)
                .help("Suppress all status updates during processing.")
                .long_help("Suppress all status updates during processing. By default this is auto-detected.")
                .value_parser(ValueParser::bool()),
        )
        .next_help_heading("Override repository options")
        .arg(
            Arg::new(args::repository::GLOB_ARCHIVES)
                .long("glob-archives")
                .action(ArgAction::Set)
                .help("Enforce a glob archives filter for all repositories.")
                .long_help("A list of space separated archive globs e.g. \"etc-* srv-*\" for archive names starting with etc- or srv-. (Default: \"\")")
                .value_hint(ValueHint::Other)
                .value_name("GLOB")
                .value_parser(ValueParser::string()),
        )
        .arg(
            // Note: `ArgAction::SetTrue` will cause `Arg::default_value` = `false` but we need `None` when the flag is not present. -> use default_missing_value
            Arg::new(args::repository::CHECK)
                .long("check")
                .action(ArgAction::Set)
                .help("Enforce to run (or not run) `borg check`")
                .long_help("Enables the execution of `borg check`. (Default: false)")
                .default_missing_value("true")
                .num_args(0..=1)
                .require_equals(true)
                .hide_possible_values(true)
                .value_hint(ValueHint::Other)
                .value_name("true|false")
                .value_parser(ValueParser::bool()),
        )
        .arg(
            Arg::new(args::repository::CHECK_OPTIONS)
                .long("check-options")
                .action(ArgAction::Set)
                .help("Enforce override of raw `borg check` options for all repositories.")
                .long_help("A list of space separated raw borg options supplied to the `borg check` command")
                .value_hint(ValueHint::Other)
                .value_name("OPTS")
                .value_parser(ValueParser::string()),
        )
        .arg(
            // Note: `ArgAction::SetTrue` will cause `Arg::default_value` = `false` but we need `None` when the flag is not present. -> use default_missing_value
            Arg::new(args::repository::COMPACT)
                .long("compact")
                .action(ArgAction::Set)
                .help("Enforce to run (or not run) `borg compact`")
                .long_help("Enables the execution of `borg compact`. (Default: false)")
                .default_missing_value("true")
                .num_args(0..=1)
                .require_equals(true)
                .hide_possible_values(true)
                .value_hint(ValueHint::Other)
                .value_name("true|false")
                .value_parser(ValueParser::bool()),
        )
        .arg(
            Arg::new(args::repository::COMPACT_OPTIONS)
                .long("compact-options")
                .action(ArgAction::Set)
                .help("Enforce override of raw `borg compact` options for all repositories.")
                .long_help("A list of space separated raw borg options supplied to the `borg compact` command")
                .value_hint(ValueHint::Other)
                .value_name("OPTS")
                .value_parser(ValueParser::string()),
        )
        .arg(
            Arg::new(args::repository::BORG_BINARY)
                .long("borg-binary")
                .action(ArgAction::Set)
                .help("Local path to a specific 'borg' binary")
                .long_help("Path to a local 'borg' binary. (Default: borg)")
                .value_hint(ValueHint::FilePath)
                .value_name("FILE")
                .value_parser(ValueParser::path_buf()),
        )
        .arg(
            Arg::new(args::repository::MAX_AGE_HOURS)
                .long("max-age-hours")
                .action(ArgAction::Set)
                .help("Threshold to warn when the last archive is older than <HOURS>")
                .long_help("Threshold to warn, when the last backup is older than <HOURS>. (Default: 24)")
                .value_hint(ValueHint::Other)
                .value_name("HOURS")
                .value_parser(ValueParser::new(f64::from_str)),
        )
}

/// Parse a given `value` into the correct type of the Arg `id` using the `clap::builder::ValueParser`
fn parse_value<T: std::any::Any + Clone + Send + Sync + 'static>(
    id: &'static str,
    value: impl Into<std::ffi::OsString> + Clone,
) -> Result<T> {
    let parser = COMMAND
        .get_arguments()
        .find(|arg| arg.get_id().as_str() == id)
        .map(|arg| arg.get_value_parser().clone())
        .ok_or_else(|| anyhow!("Unknown argument id: {id}"))?;
    let matches = Command::new("")
        .no_binary_name(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .allow_hyphen_values(true)
        .arg(Arg::new(id).value_parser(parser))
        .try_get_matches_from([value])?;
    matches
        .try_get_one::<T>(id)?
        .cloned()
        .ok_or_else(|| anyhow!("No value parsed for argument id: {id}"))
}

#[cfg(test)]
mod tests {
    /// The `value_parser` for an Arg must match the type declared for it or `parse_value()` fails.
    #[test]
    fn value_parser_types() {
        use super::args::report::{
            ENV_DIR, ENV_INHERIT, HTMLFILE, MAILFROMADDR, MAILTOADDR, METRICSFILE, NOPROGRESS,
            TEXTFILE,
        };
        use super::args::repository::{
            BORG_BINARY, CHECK, CHECK_OPTIONS, COMPACT, COMPACT_OPTIONS, GLOB_ARCHIVES,
            MAX_AGE_HOURS,
        };
        use super::parse_value;
        use email_address::EmailAddress;
        use std::path::PathBuf;

        assert!(parse_value::<PathBuf>(ENV_DIR, "/tmp").is_ok());
        assert!(parse_value::<String>(ENV_INHERIT, "repo").is_ok());
        assert!(parse_value::<PathBuf>(TEXTFILE, "report.txt").is_ok());
        assert!(parse_value::<PathBuf>(HTMLFILE, "report.html").is_ok());
        assert!(parse_value::<PathBuf>(METRICSFILE, "borg.metrics").is_ok());
        assert!(parse_value::<EmailAddress>(MAILTOADDR, "to@example.com").is_ok());
        assert!(parse_value::<EmailAddress>(MAILFROMADDR, "from@example.com").is_ok());
        assert!(parse_value::<bool>(NOPROGRESS, "true").is_ok());
        assert!(parse_value::<String>(GLOB_ARCHIVES, "etc-* srv-*").is_ok());
        assert!(parse_value::<bool>(CHECK, "true").is_ok());
        assert!(parse_value::<String>(CHECK_OPTIONS, "--verify-data").is_ok());
        assert!(parse_value::<bool>(COMPACT, "true").is_ok());
        assert!(parse_value::<String>(COMPACT_OPTIONS, "--dry-run").is_ok());
        assert!(parse_value::<PathBuf>(BORG_BINARY, "borg").is_ok());
        assert!(parse_value::<f64>(MAX_AGE_HOURS, "24").is_ok());
    }

    /// Every long option is the Arg id (and the env var) without the `BORGREPORT_` prefix in lowercase
    #[test]
    fn long_names_derive_from_ids() {
        use super::COMMAND;

        assert!(COMMAND.get_arguments().next().is_some());
        for arg in COMMAND.get_arguments() {
            let id = arg.get_id().as_str();
            let long = id
                .strip_prefix("BORGREPORT_")
                .map(|name| name.to_lowercase().replace('_', "-"));
            assert_eq!(arg.get_long(), long.as_deref(), "Arg id '{id}'");
        }
    }
}
