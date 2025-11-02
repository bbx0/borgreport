use super::{Record, Report};
use crate::borg;
use anyhow::Result;

/// Convert a borg info result into a report.
pub fn borg_info(
    repo_name: &str,
    archive_glob: Option<&str>,
    info_result: &Result<borg::Info>,
) -> Report {
    let mut report = Report::new();
    match info_result {
        Ok(info) if info.archives.is_empty() => {
            report.summary.add(Record::new(
                repo_name,
                archive_glob,
                Info {
                    archive: None,
                    repository: RepositoryInfo {
                        unique_csize: info.cache.stats.unique_csize,
                    },
                },
            ));
            // Add a warning in case the repository has no archives
            report.add_warning(
                repo_name,
                archive_glob,
                archive_glob.map_or_else(
                    || "Repository is empty".to_string(),
                    |glob| format!("The glob '{glob}' yields no result!"),
                ),
            );
        }
        // Add a line for each archive in the repository
        Ok(info) => {
            for a in &info.archives {
                report.summary.add(Record::new(
                    repo_name,
                    archive_glob,
                    Info {
                        archive: Some(ArchiveInfo {
                            name: a.name.clone(),
                            hostname: a.hostname.clone(),
                            duration: a.duration,
                            start: a.start.clone(),
                            original_size: a.stats.original_size,
                            compressed_size: a.stats.compressed_size,
                            deduplicated_size: a.stats.deduplicated_size,
                            nfiles: a.stats.nfiles,
                        }),
                        repository: RepositoryInfo {
                            unique_csize: info.cache.stats.unique_csize,
                        },
                    },
                ));
            }
        }
        Err(e) => {
            // Create an empty summary entry for the repository
            report
                .summary
                .add(Record::new(repo_name, archive_glob, None));
            // Add all borg log messages to the error section
            report.add_error(repo_name, archive_glob, e.to_string());
        }
    }
    report
}

/// Perform sanity checks on a `borg info` and return as report
pub fn sanity_check(
    repo_name: &str,
    archive_glob: Option<&str>,
    info: &borg::Info,
    max_age_hours: f64,
) -> Report {
    let mut report = Report::new();
    for a in &info.archives {
        // warn if the backup age is too old
        if let Ok(span) = a
            .start
            .until(&jiff::Zoned::now())
            .and_then(|span| span.total(jiff::Unit::Hour))
        {
            if span > max_age_hours {
                report.add_warning(
                    repo_name,
                    archive_glob,
                    format!("Last backup is older than {max_age_hours} hours"),
                );
            }
        } else {
            report.add_warning(
                repo_name,
                archive_glob,
                format!(
                    "Failed to calculate backup age with start time '{}' for archive: {} ",
                    a.start, a.name,
                ),
            );
        }
        // warn if backup Source is empty
        if a.stats.original_size == 0 {
            report.add_warning(
                repo_name,
                archive_glob,
                format!(
                    "Last backup archive contains no data. Archive {} is empty.",
                    a.name
                ),
            );
        }
    }
    report
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveInfo {
    /// Name of the backup archive
    pub name: String,
    /// Hostname on which the backup was taken
    pub hostname: String,
    /// Duration the backup has taken
    pub duration: jiff::SignedDuration,
    /// Time when backup was started
    pub start: jiff::Zoned,
    /// Total original archive size (size of backup source)
    pub original_size: i64,
    /// Total compressed archive size
    pub compressed_size: i64,
    /// Deduplicated and compressed archive size
    pub deduplicated_size: i64,
    /// Number of files in the archive
    pub nfiles: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryInfo {
    /// Total deduplicated compressed repository size
    pub unique_csize: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    /// `None`, if the borg info query did not return any archives
    pub archive: Option<ArchiveInfo>,
    pub repository: RepositoryInfo,
}

/// A single info entry (result of `borg info`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoRecord {
    /// `None`, if `borg info` returned with an error.
    pub info: Option<Info>,
}

impl From<Option<Info>> for InfoRecord {
    fn from(info: Option<Info>) -> Self {
        Self { info }
    }
}

impl From<Info> for InfoRecord {
    fn from(info: Info) -> Self {
        Self { info: Some(info) }
    }
}
