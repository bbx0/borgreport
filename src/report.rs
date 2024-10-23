// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ops::Deref;

use anyhow::Result;

use crate::borg;
pub(crate) use crate::format::Formattable;

/// Helper to associate data types used in the report
pub(crate) trait Component {}
impl Component for Report {}
impl Component for Section<BulletPoint> {}
impl Component for Section<SummaryEntry> {}
impl Component for Section<ChecksEntry> {}

/// A report contains sections with structured data
pub(crate) struct Report {
    /// The error section holds borg error messages and additional errors
    pub(crate) errors: Section<BulletPoint>,
    /// The warning section shows borg messages and additional sanity checks
    pub(crate) warnings: Section<BulletPoint>,
    /// The summary section shows statistics for the recent backup archives
    pub(crate) summary: Section<SummaryEntry>,
    /// The check section shows results from `borg check`
    pub(crate) checks: Section<ChecksEntry>,
}
impl Report {
    /// Create a new empty `Report`
    pub fn new() -> Self {
        Self {
            errors: Section::new(),
            warnings: Section::new(),
            summary: Section::new(),
            checks: Section::new(),
        }
    }

    /// Move the other Report into Self
    pub fn append(&mut self, other: Self) {
        let Self {
            errors,
            warnings,
            summary,
            checks,
        } = other;
        self.errors.append(errors.into_inner());
        self.warnings.append(warnings.into_inner());
        self.summary.append(summary.into_inner());
        self.checks.append(checks.into_inner());
    }

    /// Add a warning message to the report
    pub fn add_warning(
        &mut self,
        repository: &str,
        archive_glob: Option<&str>,
        msg: impl Into<String>,
    ) {
        self.warnings.add_str(
            repository,
            archive_glob,
            add_msg_prefix(repository, archive_glob, msg),
        );
    }

    /// Add a error message to the report
    pub fn add_error(
        &mut self,
        repository: &str,
        archive_glob: Option<&str>,
        msg: impl Into<String>,
    ) {
        self.errors.add_str(
            repository,
            archive_glob,
            add_msg_prefix(repository, archive_glob, msg),
        );
    }

    /// Returns True if the list of errors is not empty
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns the number of errors
    pub fn count_errors(&self) -> usize {
        self.errors.inner().len()
    }

    /// Returns True if the list of warnings is not empty
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns the number of warnings
    pub fn count_warnings(&self) -> usize {
        self.warnings.inner().len()
    }

    /// Convert a `borg info` result into a report
    pub fn from_borg_info_result(
        repo_name: &str,
        archive_glob: Option<&str>,
        info_result: &Result<borg::Info>,
    ) -> Self {
        let mut report = Self::new();
        match &info_result {
            Ok(info) => {
                report
                    .summary
                    .add_from_borg_info(repo_name, archive_glob, info);
            }
            Err(e) => {
                // Create an empty summary entry for the repository
                report.summary.append(vec![Record {
                    repository: repo_name.to_string(),
                    archive_glob: archive_glob.map(ToString::to_string),
                    inner: SummaryEntry {
                        ..Default::default()
                    },
                }]);
                // Add all borg log messages to the error section
                report.add_error(repo_name, archive_glob, e.to_string());
            }
        }
        report
    }

    /// Convert a `borg check` result into a report
    pub fn from_borg_check_result(
        repo_name: &str,
        archive_glob: Option<&str>,
        archive_name: Option<&str>,
        check_result: &Result<borg::Check>,
    ) -> Self {
        let mut report = Self::new();
        match check_result {
            Ok(check) => {
                report.checks.add((
                    repo_name,
                    archive_glob,
                    ChecksEntry {
                        repository: repo_name.to_string(),
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
            Err(e) => {
                // Add all borg log messages to the error section
                report.add_error(repo_name, archive_glob, e.to_string());
            }
        }
        report
    }

    /// Perform sanity checks on a `borg info` and return as report
    pub fn from_sanity_checks(
        repo_name: &str,
        archive_glob: Option<&str>,
        info: &borg::Info,
        max_age_hours: f64,
    ) -> Self {
        let mut report = Self::new();
        // warn if there are no backup archives (skip remaining tests)
        if info.archives.is_empty() {
            report.add_warning(repo_name, archive_glob, "Repository is empty");
        } else {
            for a in &info.archives {
                // warn if the backup age is too old
                if let Ok(span) = a
                    .start
                    .until(jiff::Timestamp::now().to_zoned(jiff::tz::TimeZone::UTC))
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
        }
        report
    }
}
impl Default for Report {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to format a `msg` text with a "`repo`\[`archive_glob`\] :" prefix
fn add_msg_prefix(repository: &str, archive_glob: Option<&str>, msg: impl Into<String>) -> String {
    format!(
        "{repository}{}{}{}",
        archive_glob.map_or(String::default(), |glob| "[".to_string() + glob + "]"),
        if repository.is_empty() && archive_glob.is_none() {
            ""
        } else {
            ": "
        },
        msg.into()
    )
}

/// A data point with reference to its origin
#[derive(Clone, PartialEq)]
pub(crate) struct Record<T>
where
    T: PartialEq + Clone,
{
    pub(crate) repository: String,
    pub(crate) archive_glob: Option<String>,
    inner: T,
}

impl<T> Record<T>
where
    T: PartialEq + Clone,
{
    pub(crate) fn inner(&self) -> &T {
        &self.inner
    }
}

impl<R, A, T> From<(R, Option<A>, T)> for Record<T>
where
    R: Into<String>,
    A: Into<String>,
    T: PartialEq + Clone,
{
    fn from(value: (R, Option<A>, T)) -> Self {
        let (repository, archive_glob, record) = value;
        Record {
            repository: repository.into(),
            archive_glob: archive_glob.map(Into::into),
            inner: record,
        }
    }
}

impl<T> Deref for Record<T>
where
    T: PartialEq + Clone,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

/// A section holds a list of content `T` attributed to repository / archive
pub type SectionInner<T> = Vec<Record<T>>;

/// A section holds a list of content T
pub(crate) struct Section<T>(SectionInner<T>)
where
    T: PartialEq + Clone;
impl<T> Default for Section<T>
where
    T: PartialEq + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Section<T>
where
    T: PartialEq + Clone,
{
    fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn inner(&self) -> &SectionInner<T> {
        &self.0
    }

    pub(crate) fn into_inner(self) -> SectionInner<T> {
        self.0
    }

    /// Clone the inner data and remove consecutive repeated entries.
    /// This can be necessary as different borg commands can produce the same output.
    pub(crate) fn dedup_inner(&self) -> SectionInner<T> {
        let mut list = self.inner().clone();
        list.dedup();
        list
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Add a `Record` entry
    /// The record can be added as triplet: (Into<String>,&Option<String>,<T>)
    /// Example:
    /// ```rust
    /// add(("repo", None, BulletPoint::from("Text")))
    /// ```
    fn add<R>(&mut self, record: R)
    where
        R: Into<Record<T>>,
    {
        self.0.push(record.into());
    }

    fn append(&mut self, mut records: SectionInner<T>) {
        self.0.append(&mut records);
    }
}

impl<T> Deref for Section<T>
where
    T: PartialEq + Clone,
{
    type Target = SectionInner<T>;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

/// An element of an unordered list
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct BulletPoint(String);
impl std::ops::Deref for BulletPoint {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<String> for BulletPoint {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// A `Section` with a list of `BulletPoints`
impl Section<BulletPoint> {
    /// Add a String value as new `BulletPoint`
    fn add_str(&mut self, repository: &str, archive_glob: Option<&str>, entry: impl Into<String>) {
        self.add((repository, archive_glob, entry.into().into()));
    }
}

/// A single summary entry
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct SummaryEntry {
    /// Name of the backup archive
    pub(crate) archive: String,
    /// Hostname on which the backup was taken
    pub(crate) hostname: String,
    /// Duration the backup has taken
    pub(crate) duration: std::time::Duration,
    /// Time when backup was started
    pub(crate) start: jiff::civil::DateTime,
    /// Total original archive size (size of backup source)
    pub(crate) original_size: i64,
    /// Total compressed archive size
    pub(crate) compressed_size: i64,
    /// Deduplicated and compressed archive size
    pub(crate) deduplicated_size: i64,
    /// Number of files in the archive
    pub(crate) nfiles: i64,
    /// Total deduplicated compressed repository size
    pub(crate) unique_csize: i64,
}
impl Section<SummaryEntry> {
    /// Extract and add summary entries from a borg info response
    fn add_from_borg_info(
        &mut self,
        repo_name: &str,
        archive_glob: Option<&str>,
        info: &borg::Info,
    ) {
        // Add an default entry in case the repository has no archives
        if info.archives.is_empty() {
            self.add((
                repo_name,
                archive_glob,
                SummaryEntry {
                    unique_csize: info.cache.stats.unique_csize,
                    ..Default::default()
                },
            ));
        // Add a line for each repository in the archive
        } else {
            self.append(
                info.archives
                    .iter()
                    .map(|a| Record {
                        repository: repo_name.to_string(),
                        archive_glob: archive_glob.map(ToString::to_string),
                        inner: SummaryEntry {
                            archive: a.name.clone(),
                            hostname: a.hostname.clone(),
                            duration: a.duration,
                            start: a.start,
                            original_size: a.stats.original_size,
                            compressed_size: a.stats.compressed_size,
                            deduplicated_size: a.stats.deduplicated_size,
                            nfiles: a.stats.nfiles,
                            unique_csize: info.cache.stats.unique_csize,
                        },
                    })
                    .collect(),
            );
        }
    }
}

/// A single check entry (result of `borg check`)
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct ChecksEntry {
    pub(crate) repository: String,
    pub(crate) archive_name: Option<String>,
    pub(crate) duration: std::time::Duration,
    pub(crate) status: std::process::ExitStatus,
}
