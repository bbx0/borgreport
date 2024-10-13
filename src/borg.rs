// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{bail, Context, Result};
use std::{ffi::OsStr, path::PathBuf};

pub use crate::borg_json::*;
use crate::Repository;

/// Required default Borg env vars
const BORG_DEFAULT_ENV: [(&str, &str); 2] = [("LC_ALL", "C.UTF-8"), ("TZ", "UTC")];
/// Required default Borg common args
const BORG_COMMON_ARGS: [&str; 0] = [];
//const BORG_COMMON_ARGS: [&str; 1] = ["--log-json"];

/// Wrapper to hold BORG_* env vars as key=value pairs
pub type Env = std::collections::BTreeMap<String, String>;

/// Represent the process output in unicode
pub struct Output {
    pub status: std::process::ExitStatus,
    pub stdout: String,
    pub stderr: String,
    /// command execution time
    pub duration: std::time::Duration,
}

/// Response from of `borg check` command
pub type Check = Output;

/// Wrapper to call the borg binary on OS level
pub struct Borg<'a> {
    bin: &'a PathBuf,
    env: &'a Env,
}

impl<'a> From<&'a Repository> for Borg<'a> {
    /// Create new borg instance with a scoped environment
    fn from(repo: &'a Repository) -> Self {
        Borg {
            bin: &repo.borg_binary,
            env: &repo.env,
        }
    }
}

impl<'a> Borg<'a> {
    /// Execute borg with given arguments and env scope
    fn exec<I, S>(&self, args: I) -> Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = std::process::Command::new(self.bin);
        // Collect all present BORG_* vars and remove them from process scope
        // remove systemd NOTIFY_SOCKET as precaution since borgreport emits a status by itself
        std::env::vars_os()
            .filter_map(|(k, _)| k.into_string().ok())
            .filter(|k| k.starts_with("BORG_") || k.eq("NOTIFY_SOCKET"))
            .for_each(|k| {
                command.env_remove(k);
            });
        // Run the command and measure the duration
        let now = std::time::Instant::now();
        let output = command
            .envs(BORG_DEFAULT_ENV)
            .envs(self.env)
            .args(BORG_COMMON_ARGS)
            .args(args)
            .output()
            .context(format!("Failed to execute borg binary: `{:?}`", &self.bin))?;
        let duration = now.elapsed();

        // Convert output to unicode
        Ok(Output {
            status: output.status,
            stderr: String::from_utf8(output.stderr)
                .context("Failed to convert borg stderr into an UTF-8 String!")?,
            stdout: String::from_utf8(output.stdout)
                .context("Failed to convert borg stdout into an UTF-8 String!")?,
            duration,
        })
    }

    /// Query borg info command
    pub fn info(&self, archive_glob: &Option<String>) -> Result<Info> {
        let mut args = vec!["--bypass-lock", "info"];
        if let Some(glob) = archive_glob {
            args.extend(["--glob-archives", glob.as_str()]);
        }
        args.extend(["--last", "1", "--json", "::"]);

        let output = self.exec(args)?;

        if output.status.success() {
            let info = serde_json::from_str(&output.stdout)
                .context("Failed to parse JSON response of `borg info` command in serde!")?;
            Ok(info)
        } else {
            bail!(output.stderr);
        }
    }

    /// Check an archive in the repo: `borg check ::<ARCHIVE>` or the whole repo otherwise
    pub fn check(&self, archive_name: Option<&str>) -> Result<Check> {
        self.exec(vec![
            "check",
            &format!("::{}", archive_name.unwrap_or_default()),
        ])
        //self.exec(["check", "--last", "1", "::"])
    }
}
