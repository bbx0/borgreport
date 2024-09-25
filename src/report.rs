// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use comfy_table::{presets::ASCII_MARKDOWN, CellAlignment, ContentArrangement, Table};
use human_repr::{HumanCount, HumanDuration};

use crate::borg;

/// A report contains sections with structured data
pub struct Report {
    /// The error section holds borg error messages and additional errors
    errors: Section<BulletPoint>,
    /// The warning section shows borg messages and additional sanity checks
    warnings: Section<BulletPoint>,
    /// The summary section shows statistics for the recent backup archives
    summary: Section<SummaryEntry>,
    /// The check section shows results from `borg check`
    checks: Section<ChecksEntry>,
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
    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.add_str(msg);
    }

    /// Add a error message to the report
    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.add_str(msg);
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
    pub fn from_borg_info_result(repo_name: &str, info_result: &Result<borg::Info>) -> Self {
        let mut report = Self::new();
        match &info_result {
            Ok(info) => {
                report.summary.add_from_borg_info(repo_name, info);
            }
            Err(e) => {
                // Create an empty summary entry for the repository
                report.summary.append(vec![SummaryEntry {
                    repository: repo_name.to_string(),
                    ..Default::default()
                }]);
                // Add all borg log messages to the error section
                report.add_error(format!("{repo_name}: {e}"));
            }
        }
        report
    }

    /// Convert a `borg check` result into a report
    pub fn from_borg_check_result(
        repo_name: &str,
        archive_name: Option<&str>,
        check_result: &Result<borg::Check>,
    ) -> Self {
        let mut report = Self::new();
        match check_result {
            Ok(check) => {
                report.checks.add(ChecksEntry {
                    repository: repo_name.to_string(),
                    archive_name: archive_name.map(std::string::ToString::to_string),
                    duration: check.duration,
                    status: check.status,
                });
                if !check.stdout.is_empty() {
                    report.add_warning(format!("{}: {}", repo_name, check.stdout));
                }
                if !check.stderr.is_empty() {
                    report.add_error(format!("{}: {}", repo_name, check.stderr));
                }
            }
            Err(e) => {
                // Add all borg log messages to the error section
                report.add_error(format!("{repo_name}: {e}"));
            }
        }
        report
    }

    /// Perform sanity checks on a `borg info` and return as report
    pub fn from_sanity_checks(repo_name: &str, info: &borg::Info, max_age_hours: f64) -> Self {
        let mut report = Self::new();
        // warn if there are no backup archives (skip remaining tests)
        if info.archives.is_empty() {
            report.add_warning(format!("{repo_name}: Repository is empty"));
        } else {
            for a in &info.archives {
                // warn if the backup age is too old
                if let Ok(span) = a
                    .start
                    .until(jiff::Timestamp::now().to_zoned(jiff::tz::TimeZone::UTC))
                    .and_then(|span| span.total(jiff::Unit::Hour))
                {
                    if span > max_age_hours {
                        report.add_warning(format!(
                            "{repo_name} - {}: Last backup is older than {max_age_hours} hours",
                            a.name
                        ));
                    }
                } else {
                    report.add_warning(format!(
                        "{repo_name} - {}: Failed to calculate backup age with start time '{}' ",
                        a.name, a.start
                    ));
                }
                // warn if backup Source is empty
                if a.stats.original_size == 0 {
                    report.add_warning(format!(
                        "{repo_name} - {}: Last backup archive contains no data",
                        a.name
                    ));
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

/// Pretty print the report with its sections
impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now = jiff::Zoned::now();

        // Title
        writeln!(
            f,
            "==== Backup report ({}) ====\n",
            jiff::fmt::strtime::format("%F", &now).unwrap_or_default(),
        )?;

        if !self.errors.is_empty() {
            writeln!(f, "=== Errors ===\n\n{}", self.errors)?;
        }
        if !self.warnings.is_empty() {
            writeln!(f, "=== Warnings ===\n\n{}", self.warnings)?;
        }
        if !self.summary.is_empty() {
            writeln!(f, "=== Summary ===\n\n{}", self.summary)?;
        }
        if !self.checks.is_empty() {
            writeln!(f, "=== `borg check` result ===\n\n{}", self.checks,)?;
        }

        // Footer
        writeln!(
            f,
            "Generated {} ({} {})",
            jiff::fmt::rfc2822::to_string(&now).unwrap_or_default(),
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )
    }
}

/// A section holds a list of content T
struct Section<T>(Vec<T>);
impl<T> Default for Section<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Section<T> {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn inner(&self) -> &Vec<T> {
        &self.0
    }

    fn into_inner(self) -> Vec<T> {
        self.0
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn add(&mut self, entry: T) {
        self.0.push(entry);
    }

    fn append(&mut self, mut entries: Vec<T>) {
        self.0.append(&mut entries);
    }
}

/// Pretty print a section with a list of `BulletPoint` as list
impl std::fmt::Display for Section<BulletPoint> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Different commands can produce the same output. Remove consecutive repeated entries.
        let mut list = self.inner().clone();
        list.dedup();

        // Print all lines of the section entry and add a bullet point to its first line
        for entry in list {
            let mut lines = entry.trim().lines();
            if let Some(line) = lines.next() {
                writeln!(f, " * {line}")?;
            }
            for line in lines {
                writeln!(f, "   {line}")?;
            }
        }
        Ok(())
    }
}

/// Pretty print a section with a list of `SummaryEntry` as table
impl std::fmt::Display for Section<SummaryEntry> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        table
            .load_preset(ASCII_MARKDOWN)
            .set_content_arrangement(ContentArrangement::Disabled)
            .set_header(vec![
                "Repository",
                "Hostname",
                "Last archive",
                "Start",
                "Duration",
                "Source",
                "Δ Archive",
                "∑ Repository",
            ]);
        for e in self.inner() {
            table.add_row(vec![
                format!("{}", e.repository),
                format!("{}", e.hostname),
                format!("{}", e.archive),
                jiff::fmt::strtime::format("%F", e.start).unwrap_or_else(|_| String::default()),
                format!("{}", e.duration.human_duration()),
                format!("{}", e.original_size.human_count_bytes()),
                format!("{}", e.deduplicated_size.human_count_bytes()),
                format!("{}", e.unique_csize.human_count_bytes()),
            ]);
        }
        //the columns 4,5,6,7 are aligned right
        for i in 4..=7 {
            if let Some(c) = table.column_mut(i) {
                c.set_cell_alignment(CellAlignment::Right);
            }
        }
        writeln!(f, "{table}")
    }
}

/// Pretty print a section with a list of `CheckEntry` as table
impl std::fmt::Display for Section<ChecksEntry> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        table
            .load_preset(ASCII_MARKDOWN)
            .set_content_arrangement(ContentArrangement::Disabled)
            .set_header(vec!["Repository", "Archive", "Duration", "Okay"]);
        for e in self.inner() {
            table.add_row(vec![
                format!("{}", e.repository),
                format!("{}", e.archive_name.clone().unwrap_or_default()),
                format!("{}", e.duration.human_duration()),
                format!("{}", if e.status.success() { "yes" } else { "no" }),
            ]);
        }
        //columns 2,3 are aligned right
        for i in 2..=3 {
            if let Some(c) = table.column_mut(i) {
                c.set_cell_alignment(CellAlignment::Right);
            }
        }
        writeln!(f, "{table}")
    }
}

/// An element of an unordered list
#[derive(Clone, PartialEq)]
struct BulletPoint(String);
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
    fn add_str(&mut self, entry: impl Into<String>) {
        self.add(entry.into().into());
    }
}

/// A single summary entry
#[derive(Debug, Default)]
struct SummaryEntry {
    repository: String,
    archive: String,
    hostname: String,
    duration: std::time::Duration,
    start: jiff::civil::DateTime,
    original_size: u64,
    deduplicated_size: u64,
    unique_csize: u64,
}
impl Section<SummaryEntry> {
    /// Extract and add summary entries from a borg info response
    fn add_from_borg_info(&mut self, repo_name: &str, info: &borg::Info) {
        // Add an default entry in case the repository has no archives
        if info.archives.is_empty() {
            self.add(SummaryEntry {
                repository: repo_name.to_string(),
                unique_csize: info.cache.stats.unique_csize,
                ..Default::default()
            });
        // Add a line for each repository in the archive
        } else {
            self.append(
                info.archives
                    .iter()
                    .map(|a| SummaryEntry {
                        repository: repo_name.to_string(),
                        archive: a.name.clone(),
                        hostname: a.hostname.clone(),
                        duration: a.duration,
                        start: a.start,
                        original_size: a.stats.original_size,
                        deduplicated_size: a.stats.deduplicated_size,
                        unique_csize: info.cache.stats.unique_csize,
                    })
                    .collect(),
            );
        }
    }
}

/// A single check entry (result of `borg check`)
#[derive(Debug, Default)]
struct ChecksEntry {
    repository: String,
    archive_name: Option<String>,
    duration: std::time::Duration,
    status: std::process::ExitStatus,
}
