// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ops::Deref;

use crate::borg;
pub use crate::format::Formattable;
use anyhow::Result;

/// Helper to associate data types used in the report
pub trait Component {}
impl Component for Report {}
impl Component for Section<BulletPoint> {}
impl Component for Section<SummaryEntry> {}
impl Component for Section<ChecksEntry> {}
impl Component for Section<CompactsEntry> {}

/// A report contains sections with structured data
pub struct Report {
    /// The error section holds borg error messages and additional errors
    pub errors: Section<BulletPoint>,
    /// The warning section shows borg messages and additional sanity checks
    pub warnings: Section<BulletPoint>,
    /// The summary section shows statistics for the recent backup archives
    pub summary: Section<SummaryEntry>,
    /// The check section shows results from `borg check`
    pub checks: Section<ChecksEntry>,
    /// The compact section shows results from `borg compact`
    pub compacts: Section<CompactsEntry>,
}
impl Report {
    /// Create a new empty `Report`
    pub const fn new() -> Self {
        Self {
            errors: Section::new(),
            warnings: Section::new(),
            summary: Section::new(),
            checks: Section::new(),
            compacts: Section::new(),
        }
    }

    /// Move the other Report into Self
    pub fn append(&mut self, other: Self) {
        let Self {
            errors,
            warnings,
            summary,
            checks,
            compacts,
        } = other;
        self.errors.append(errors.into_inner());
        self.warnings.append(warnings.into_inner());
        self.summary.append(summary.into_inner());
        self.checks.append(checks.into_inner());
        self.compacts.append(compacts.into_inner());
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
    pub const fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns the number of errors
    pub const fn count_errors(&self) -> usize {
        self.errors.inner().len()
    }

    /// Returns True if the list of warnings is not empty
    pub const fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns the number of warnings
    pub const fn count_warnings(&self) -> usize {
        self.warnings.inner().len()
    }

    /// Return `true` if the report contains a warning or error for `repo`
    pub fn has_warning_or_error_for(&self, repo: &str) -> bool {
        self.warnings.iter().any(|w| w.repository.eq(repo))
            || self.errors.iter().any(|e| e.repository.eq(repo))
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
                report.summary.add(Record::new(
                    repo_name,
                    archive_glob,
                    SummaryEntry::default(),
                ));
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
                report.checks.add(Record::new(
                    repo_name,
                    archive_glob,
                    ChecksEntry {
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

    /// Convert a `borg compact` result into a report. When `None` is given an empty entry is made.
    pub fn from_borg_compact_result<O>(repo_name: &str, compact_result: O) -> Self
    where
        O: Into<Option<Result<borg::Compact>>>,
    {
        let mut report = Self::new();
        match compact_result.into() {
            Some(Ok(compact)) => {
                report.compacts.add(Record::new(
                    repo_name,
                    None,
                    CompactsEntry::new(compact.duration, compact.status, compact.freed_bytes),
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
            }
            None => {
                report.compacts.add(Record::new(repo_name, None, None));
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
        }
        report
    }
}
impl Default for Report {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to format a `msg` text with a `repo[archive_glob]:` prefix
fn add_msg_prefix(repository: &str, archive_glob: Option<&str>, msg: impl Into<String>) -> String {
    format!(
        "{repository}{}{}{}",
        archive_glob.map_or_else(String::default, |glob| String::from("[") + glob + "]"),
        if repository.is_empty() && archive_glob.is_none() {
            ""
        } else {
            ": "
        },
        msg.into()
    )
}

/// A data point with reference to its origin
#[derive(Clone, PartialEq, Eq)]
pub struct Record<T>
where
    T: PartialEq + Clone,
{
    pub repository: String,
    pub archive_glob: Option<String>,
    inner: T,
}

impl<T> Record<T>
where
    T: PartialEq + Clone,
{
    pub fn new<S, I>(repository: S, archive_glob: Option<S>, inner: I) -> Self
    where
        S: Into<String>,
        I: Into<T>,
    {
        Self {
            repository: repository.into(),
            archive_glob: archive_glob.map(Into::into),
            inner: inner.into(),
        }
    }

    pub const fn inner(&self) -> &T {
        &self.inner
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
pub struct Section<T>(SectionInner<T>)
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
    const fn new() -> Self {
        Self(Vec::new())
    }

    pub const fn inner(&self) -> &SectionInner<T> {
        &self.0
    }

    pub fn into_inner(self) -> SectionInner<T> {
        self.0
    }

    /// Clone the inner data and remove consecutive repeated entries.
    /// This can be necessary as different borg commands can produce the same output.
    pub fn dedup_inner(&self) -> SectionInner<T> {
        let mut list = self.inner().clone();
        list.dedup();
        list
    }

    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Add a `Record` entry
    /// Example:
    /// ```rust
    /// add(Record::new("repo", None, BulletPoint::from("Text")))
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
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BulletPoint(String);
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
    fn add_str(
        &mut self,
        repository: &str,
        archive_glob: Option<&str>,
        entry: impl Into<BulletPoint>,
    ) {
        self.add(Record::new(repository, archive_glob, entry));
    }
}

/// A single summary entry
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SummaryEntry {
    /// Name of the backup archive
    pub archive: String,
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
    /// Total deduplicated compressed repository size
    pub unique_csize: i64,
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
            self.add(Record::new(
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
                    .map(|a| {
                        Record::new(
                            repo_name,
                            archive_glob,
                            SummaryEntry {
                                archive: a.name.clone(),
                                hostname: a.hostname.clone(),
                                duration: a.duration,
                                start: a.start.clone(),
                                original_size: a.stats.original_size,
                                compressed_size: a.stats.compressed_size,
                                deduplicated_size: a.stats.deduplicated_size,
                                nfiles: a.stats.nfiles,
                                unique_csize: info.cache.stats.unique_csize,
                            },
                        )
                    })
                    .collect(),
            );
        }
    }
}

/// A single check entry (result of `borg check`)
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ChecksEntry {
    pub archive_name: Option<String>,
    pub duration: jiff::SignedDuration,
    pub status: std::process::ExitStatus,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CompactsEntryInner {
    pub duration: jiff::SignedDuration,
    pub status: std::process::ExitStatus,
    /// `None`, if no `freed_bytes` were returned. This happens when remote repositories not preserve
    /// the `SSH_ORIGINAL_COMMAND`, which is needed to forward the `--info` flag to `borg serve`.
    /// <https://borgbackup.readthedocs.io/en/1.4.1/usage/serve.html#examples>
    pub freed_bytes: Option<u64>,
}

/// A single compact entry (result of `borg compact`)
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CompactsEntry {
    /// `None`, if `borg compact` was requested to run but skipped due to previous warnings or errors.
    pub entry: Option<CompactsEntryInner>,
}

impl CompactsEntry {
    const fn new(
        duration: jiff::SignedDuration,
        status: std::process::ExitStatus,
        freed_bytes: Option<u64>,
    ) -> Self {
        Self {
            entry: Some(CompactsEntryInner {
                duration,
                status,
                freed_bytes,
            }),
        }
    }
}

impl From<Option<CompactsEntryInner>> for CompactsEntry {
    fn from(entry: Option<CompactsEntryInner>) -> Self {
        Self { entry }
    }
}
