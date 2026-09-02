// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use crate::{
    borg::{self, Env},
    cli::{self, RepositoryArg},
};
use anyhow::{Context, Result, ensure};

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
    /// True if `borg compact` shall run
    pub run_compact: bool,
    /// List of additional raw `borg compact` options
    pub compact_options: Vec<String>,
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
            .context(format!(
                "ENV file '{}' has no valid filename",
                file.display()
            ))?
            .to_string();

        // This is collected in two steps to raise dotenvy parsing errors properly.
        let env = dotenvy::from_filename_iter(file)
            .context(format!("Cannot open ENV file {}", file.display()))?
            .collect::<Result<Vec<(String, String)>, dotenvy::Error>>()
            .context(format!("Cannot parse the file '{}'", file.display()))?
            .into_iter()
            .collect();

        Self::from_env(repo_name, env)
    }

    /// Construct a `Repository` with a list of `env` vars (BORG_*).
    /// The CLI options and global ENV are evaluated in addition.
    pub fn from_env(repo_name: String, env: borg::Env) -> Result<Self> {
        let args = cli::repository_args();

        // Provide default values
        let borg_binary =
            resolve(&args.borg_binary, &env, &repo_name)?.unwrap_or_else(|| PathBuf::from("borg"));
        let run_check = resolve(&args.check, &env, &repo_name)?.unwrap_or(false);
        let run_compact = resolve(&args.compact, &env, &repo_name)?.unwrap_or(false);
        let max_age_hours = resolve(&args.max_age_hours, &env, &repo_name)?.unwrap_or(24.0);
        let archive_globs = split_whitespace(resolve(&args.glob_archives, &env, &repo_name)?);
        let check_options = split_whitespace(resolve(&args.check_options, &env, &repo_name)?);
        let compact_options = split_whitespace(resolve(&args.compact_options, &env, &repo_name)?);

        ensure!(
            env.get("BORG_REPO").is_some_and(|v| !v.is_empty()),
            "No value for 'BORG_REPO' was provided for repository: '{repo_name}'"
        );

        Ok(Self {
            name: repo_name,
            env,
            borg_binary,
            archive_globs,
            run_check,
            check_options,
            run_compact,
            compact_options,
            max_age_hours,
        })
    }
}

/// Split an optional space separated option list into its parts
fn split_whitespace(value: Option<String>) -> Vec<String> {
    value.map_or_else(Vec::new, |v| {
        v.split_whitespace()
            .map(std::string::String::from)
            .collect()
    })
}

/// Resolve a CLI override for the borg env configuration
/// 1. CLI argument
/// 2. the repository's own `.env` file
/// 3. the process environment
fn resolve<T>(arg: &RepositoryArg<T>, borg_env: &Env, repo_name: &str) -> Result<Option<T>>
where
    T: std::any::Any + Clone + Send + Sync + 'static,
{
    if let Some(v) = arg.value() {
        return Ok(Some(v.clone()));
    }
    borg_env
        .get(arg.id())
        .map_or_else(
            || std::env::var_os(arg.id()).map(|value| arg.parse(value)),
            |value| Some(arg.parse(value)),
        )
        .transpose()
        .context(format!(
            "Cannot parse parameter {} for repo {repo_name}",
            arg.id()
        ))
}
