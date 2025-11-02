use super::{Record, Report};
use crate::borg;
use anyhow::Result;

/// Convert a `borg compact` result into a report. When `None` is given an empty entry is made.
pub fn borg_compact<O>(repo_name: &str, compact_result: O) -> Report
where
    O: Into<Option<Result<borg::Compact>>>,
{
    let mut report = Report::new();
    match compact_result.into() {
        Some(Ok(compact)) => {
            report.compacts.add(Record::new(
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
            report.compacts.add(Record::new(repo_name, None, None));
        }
        None => report.compacts.add(Record::new(repo_name, None, None)),
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
