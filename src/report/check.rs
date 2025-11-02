// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Record, Report};
use crate::borg;
use anyhow::Result;

/// Convert a `borg check` result into a report. When `None` is given an empty entry is made.
pub fn borg_check<O>(
    repo_name: &str,
    archive_glob: Option<&str>,
    archive_name: Option<&str>,
    check_result: O,
) -> Report
where
    O: Into<Option<Result<borg::Check>>>,
{
    let mut report = Report::new();
    match check_result.into() {
        Some(Ok(check)) => {
            report.checks.add(Record::new(
                repo_name,
                archive_glob,
                Check {
                    archive_name: archive_name.map(ToString::to_string),
                    duration: check.duration,
                    status: check.status,
                },
            ));
            if !check.stdout.is_empty() {
                report.add_warning(repo_name, archive_glob, &check.stdout);
            }
            if !check.stderr.is_empty() {
                report.add_error(repo_name, archive_glob, &check.stderr);
            }
        }
        Some(Err(e)) => {
            // Add all borg log messages to the error section
            report.add_error(repo_name, archive_glob, e.to_string());
            report
                .checks
                .add(Record::new(repo_name, archive_glob, None));
        }
        None => report
            .checks
            .add(Record::new(repo_name, archive_glob, None)),
    }
    report
}

/// A single check entry (result of `borg check`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckRecord {
    /// `None`, if `borg check` was requested to run but skipped due to previous errors.
    pub check: Option<Check>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// A check can be done for a whole repository or a single archive
    pub archive_name: Option<String>,
    pub duration: jiff::SignedDuration,
    pub status: std::process::ExitStatus,
}

impl From<Option<Check>> for CheckRecord {
    fn from(inner: Option<Check>) -> Self {
        Self { check: inner }
    }
}
impl From<Check> for CheckRecord {
    fn from(inner: Check) -> Self {
        Self { check: Some(inner) }
    }
}
