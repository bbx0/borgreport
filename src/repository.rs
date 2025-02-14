// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use crate::{
    borg::{self, Env},
    cli,
};
use anyhow::{anyhow, ensure, Context, Result};

/// BORGREPORT_* env vars used on `Repository` level
/// These need to match a clap `ArgId` to allow overriding via cli option.
/// These must not have a clap `env` or it will overrule the repo config.
mod args {
    pub(super) use crate::cli::args::{
        BORG_BINARY, CHECK, CHECK_OPTIONS, GLOB_ARCHIVES, MAX_AGE_HOURS,
    };
}

/// A `Repository` describes the access parameters for a borg repository
#[derive(Clone, Debug)]
pub struct Repository {
    /// Name of the repository
    pub name: String,
    /// Collection of BORG_* env vars required to access the Repository
    pub env: borg::Env,
    /// The borg binary path
    pub borg_binary: PathBuf,
    /// list of given archive globs
    pub archive_globs: Vec<String>,
    /// True if `borg check` shall run
    pub run_check: bool,
    /// List of additional raw `borg check` options
    pub check_options: Vec<String>,
    /// Threshold for the sanity check to alert, when an archive is older
    pub max_age_hours: f64,
}
impl Repository {
    /// Parse an env file into a `Repository` configuration.
    /// The file should contain required BORG_* variables to access the repository.
    /// The file can contain BORGREPORT_* variables to change processing of the report.
    pub fn from_env_file(file: &std::path::PathBuf) -> Result<Self> {
        // The repo name is the file name without its extension
        let repo_name = file
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .context("ENV file '{file:?}' has no valid filename")?
            .to_string();

        // This is collected in two steps to raise dotenvy parsing errors properly.
        let env = dotenvy::from_filename_iter(file)
            .context(format!("Cannot open ENV file {file:?}"))?
            .collect::<Result<Vec<(String, String)>, dotenvy::Error>>()
            .context(format!("Cannot parse the file '{file:?}'"))?
            .into_iter()
            .collect();

        Self::from_env(repo_name, env)
    }

    /// Construct a `Repository` with a list of `env` vars (BORG_*).
    /// The CLI options and global ENV are evaluated in addition.
    pub fn from_env(repo_name: String, env: borg::Env) -> Result<Self> {
        let name = repo_name;

        // Get the args with some added error context
        macro_rules! arg_error_context {
            ($arg: path) => {
                arg(&env, $arg)
                    .context(format!("Cannot parse parameter {} for repo {name}", $arg))?
            };
        }

        // Provide default values
        let borg_binary =
            arg_error_context!(args::BORG_BINARY).unwrap_or_else(|| PathBuf::from("borg"));
        let run_check = arg_error_context!(args::CHECK).unwrap_or(false);
        let max_age_hours = arg_error_context!(args::MAX_AGE_HOURS).unwrap_or(24.0);
        let archive_globs =
            arg_error_context!(args::GLOB_ARCHIVES).map_or(Vec::new(), |globs: String| {
                globs
                    .split_whitespace()
                    .map(std::string::String::from)
                    .collect()
            });
        let check_options =
            arg_error_context!(args::CHECK_OPTIONS).map_or(Vec::new(), |opts: String| {
                opts.split_whitespace()
                    .map(std::string::String::from)
                    .collect()
            });

        ensure!(
            env.get("BORG_REPO").is_some_and(|v| !v.is_empty()),
            "No value for 'BORG_REPO' was provided for repository: '{name}'"
        );

        Ok(Self {
            name,
            env,
            borg_binary,
            archive_globs,
            run_check,
            check_options,
            max_age_hours,
        })
    }
}

/// Check the CLI, the global env and the given env (a repo env) for the argument
fn arg<T>(env: &Env, id: &str) -> Result<Option<T>>
where
    T: FromArg<Value = T>,
{
    T::from_repo_arg(env, id)
}

/// Construct a value from a CLI or ENV value
trait FromArg {
    type Value;
    fn from_cli_arg(id: &str) -> Result<Option<Self::Value>>;
    fn from_cli_env(id: &str) -> Result<Option<Self::Value>>;
    fn from_repo_env(env: &Env, id: &str) -> Result<Option<Self::Value>>;

    // 1. Check the command line option
    // 2. Check the local env (the repo config)
    // 3. Check the global env for any provided default
    fn from_repo_arg(env: &Env, id: &str) -> Result<Option<Self::Value>> {
        if let Some(v) = Self::from_cli_arg(id)? {
            return Ok(Some(v));
        }
        if let Some(v) = Self::from_repo_env(env, id)? {
            return Ok(Some(v));
        }
        if let Some(v) = Self::from_cli_env(id)? {
            return Ok(Some(v));
        }
        Ok(None)
    }
}

// This way we can use the `clap::value_parser!`
macro_rules! from_arg_impl {
    ($type: ident) => {
        impl FromArg for $type {
            type Value = Self;
            /// CLI Argument as $type
            fn from_cli_arg(id: &str) -> Result<Option<Self::Value>> {
                Ok(cli::matches().try_get_one::<$type>(id)?.cloned())
            }
            /// CLI Environment as $type parsed via clap
            fn from_cli_env(id: &str) -> Result<Option<Self::Value>> {
                if let Some(value) = std::env::var_os(id) {
                    return Ok(Some(clap_parse::<$type>(
                        id,
                        clap::value_parser!($type),
                        value,
                    )?));
                }
                Ok(None)
            }
            /// Repository Environment as $type parsed via clap
            fn from_repo_env(env: &borg::Env, id: &str) -> Result<Option<Self::Value>> {
                if let Some(value) = env.get(id) {
                    return Ok(Some(clap_parse::<$type>(
                        id,
                        clap::value_parser!($type),
                        value,
                    )?));
                }
                Ok(None)
            }
        }
    };
}
from_arg_impl! {bool}
from_arg_impl! {f64}
from_arg_impl! {String}
from_arg_impl! {PathBuf}

/// Parse the argument `value` with `parser`. Use `id` as argument name in error.
fn clap_parse<T: std::any::Any + Clone + Send + Sync + 'static>(
    id: &str,
    parser: impl clap::builder::IntoResettable<clap::builder::ValueParser>,
    value: impl Into<std::ffi::OsString> + Clone,
) -> Result<T> {
    let matches = clap::Command::new("")
        .no_binary_name(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .allow_hyphen_values(true)
        .arg(clap::Arg::new(id.to_string()).value_parser(parser))
        .try_get_matches_from([value])?;
    matches
        .try_get_one::<T>(id)?
        .cloned()
        .ok_or_else(|| anyhow!("Cannot parse {id}: The parser does not match the type!",))
}
