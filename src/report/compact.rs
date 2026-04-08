// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Record, Report};
use crate::{
    borg,
    report::{CompactSection, Tabular, TabularCellAlignment},
};
use anyhow::Result;
use human_repr::{HumanCount, HumanDuration};

/// Convert a `borg compact` result into a report. When `None` is given an empty entry is made.
pub fn borg_compact<O>(repo_name: &str, compact_result: O) -> Report
where
    O: Into<Option<Result<borg::Compact>>>,
{
    let mut report = Report::new();
    match compact_result.into() {
        Some(Ok(compact)) => {
            report.compacts.push(Record::new(
                repo_name,
                None,
                Compact {
                    duration: compact.duration,
                    status: compact.status,
                    freed_bytes: compact.freed_bytes,
                },
            ));
            if !compact.stdout.is_empty() {
                report.add_warning(repo_name, None, &compact.stdout);
            }
            if !compact.stderr.is_empty() {
                report.add_error(repo_name, None, &compact.stderr);
            }
        }
        Some(Err(e)) => {
            // Add all borg log messages to the error section
            report.add_error(repo_name, None, e.to_string());
            report.compacts.push(Record::new(repo_name, None, None));
        }
        None => report.compacts.push(Record::new(repo_name, None, None)),
    }

    report
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compact {
    pub duration: jiff::SignedDuration,
    pub status: std::process::ExitStatus,
    /// `None`, if no `freed_bytes` were returned. This happens when remote repositories not preserve
    /// the `SSH_ORIGINAL_COMMAND`, which is needed to forward the `--info` flag to `borg serve`.
    /// <https://borgbackup.readthedocs.io/en/1.4.1/usage/serve.html#examples>
    pub freed_bytes: Option<u64>,
}

/// A single compact entry (result of `borg compact`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactRecord {
    /// `None`, if `borg compact` was requested to run but skipped due to previous warnings or errors.
    pub compact: Option<Compact>,
}

impl From<Option<Compact>> for CompactRecord {
    fn from(inner: Option<Compact>) -> Self {
        Self { compact: inner }
    }
}
impl From<Compact> for CompactRecord {
    fn from(inner: Compact) -> Self {
        Self {
            compact: Some(inner),
        }
    }
}

impl Tabular for CompactSection {
    fn table_preface(&self) -> Vec<&'static str> {
        let mut preface = vec![];
        if self.iter().any(|r| r.compact.is_none()) {
            preface.push("Repositories with errors or warnings are not compacted.");
        }

        if self
            .iter()
            .any(|r| r.compact.as_ref().is_some_and(|e| e.freed_bytes.is_none()))
        {
            preface.push(
                "Some remote repositories cannot return the freed bytes. This happens when the SSH_ORIGINAL_COMMAND is not passed to borg serve."
            );
        }

        preface
    }

    fn table_header() -> Vec<&'static str> {
        vec!["Repository", "Duration", "Freed space"]
    }

    fn table_alignment() -> Vec<TabularCellAlignment> {
        use TabularCellAlignment::{Left, Right};
        vec![Left, Right, Right]
    }

    fn table_row_iter(&self) -> impl Iterator<Item = Vec<String>> {
        self.iter().map(|r| {
            r.compact.as_ref().map_or_else(
                || vec![r.repository.clone(), "-".to_string(), "-".to_string()],
                |compact| {
                    vec![
                        r.repository.clone(),
                        compact.duration.as_secs_f64().human_duration().to_string(),
                        compact
                            .freed_bytes
                            .map_or_else(String::new, |b| b.human_count_bytes().to_string()),
                    ]
                },
            )
        })
    }
}
