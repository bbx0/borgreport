// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Context, Result, bail};
use std::{ffi::OsStr, ops::Deref, path::PathBuf};

pub use crate::borg_json::*;
use crate::{Repository, utils};

/// All borg timestamps are UTC
pub const BORG_TZ: &str = "UTC";

/// Required default Borg env vars
#[cfg(target_os = "linux")]
const BORG_DEFAULT_ENV: [(&str, &str); 2] = [("LC_ALL", "C.UTF-8"), ("TZ", BORG_TZ)];
#[cfg(target_os = "macos")]
const BORG_DEFAULT_ENV: [(&str, &str); 3] =
    [("LANG", "C"), ("LC_CTYPE", "C.UTF-8"), ("TZ", BORG_TZ)];

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
    pub duration: jiff::SignedDuration,
}

/// Response from of `borg check` command
pub type Check = Output;

/// Response from of `borg compact` command
pub struct Compact {
    pub output: Output,
    /// Freed bytes after compacting the repository. Borg returns human-friendly numbers (e.g. kB). The value is not precise.
    pub freed_bytes: Option<u64>,
}
impl Deref for Compact {
    type Target = Output;

    fn deref(&self) -> &Self::Target {
        &self.output
    }
}

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

impl Borg<'_> {
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
        let now = jiff::Zoned::now();
        let output = command
            .envs(BORG_DEFAULT_ENV)
            .envs(self.env)
            .args(BORG_COMMON_ARGS)
            .args(args)
            .output()
            .context(format!(
                "Failed to execute borg binary: `{}`",
                &self.bin.display()
            ))?;
        let duration = jiff::Zoned::now().duration_since(&now);

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
    pub fn info(&self, archive_glob: Option<&str>) -> Result<Info> {
        let mut args = vec!["--bypass-lock", "info"];
        if let Some(glob) = archive_glob {
            args.extend(["--glob-archives", glob]);
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
    pub fn check<T>(&self, archive_name: Option<&str>, check_opts: &[T]) -> Result<Check>
    where
        T: AsRef<str>,
    {
        let mut args = vec!["check"];
        args.extend(check_opts.iter().map(AsRef::as_ref));

        let repository_or_archive = format!("::{}", archive_name.unwrap_or_default());
        args.push(repository_or_archive.as_str());

        self.exec(args)
    }

    /// Compact a repository to free space
    pub fn compact<T>(&self, compact_opts: &[T]) -> Result<Compact>
    where
        T: AsRef<str>,
    {
        // --verbose is required to write the freed bytes to stderr as a log message
        let mut args = vec!["compact", "--verbose"];
        args.extend(compact_opts.iter().map(AsRef::as_ref));

        let output = self.exec(args)?;

        // Get the freed bytes from stderr and remove the line.
        let mut freed_bytes = Option::default();
        let mut stderr = String::new();
        for line in output.stderr.lines() {
            if freed_bytes.is_none() {
                if let Some(bytes) = utils::first_typed_bytes(line) {
                    freed_bytes = Some(bytes);
                    continue;
                }
            }
            stderr.push_str(line);
            stderr.push('\n');
        }

        Ok(Compact {
            output: Output {
                status: output.status,
                stdout: output.stdout,
                stderr,
                duration: output.duration,
            },
            freed_bytes,
        })
    }
}
