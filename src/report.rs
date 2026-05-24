// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

mod check;
mod compact;
mod info;

use crate::utils::with_brackets_or;
pub use check::borg_check;
pub use compact::borg_compact;
pub use info::borg_info;
pub use info::sanity_check;

// Variants of the section types
pub type ErrorSection = Vec<Record<BulletPoint>>;
pub type WarningSection = Vec<Record<BulletPoint>>;
pub type CheckSection = Vec<Record<check::Check>>;
pub type CompactSection = Vec<Record<compact::Compact>>;
pub type SummarySection = Vec<Record<info::Info>>;

/// An element of an unordered list
type BulletPoint = String;

/// A report contains sections with structured data
pub struct Report {
    /// The error section holds borg error messages and additional errors
    pub errors: ErrorSection,
    /// The warning section shows borg messages and additional sanity checks
    pub warnings: WarningSection,
    /// The summary section shows statistics for the recent backup archives
    pub summary: SummarySection,
    /// The check section shows results from `borg check`
    pub checks: CheckSection,
    /// The compact section shows results from `borg compact`
    pub compacts: CompactSection,
}
impl Report {
    /// Create a new empty `Report`
    pub const fn new() -> Self {
        Self {
            errors: ErrorSection::new(),
            warnings: WarningSection::new(),
            summary: SummarySection::new(),
            checks: CheckSection::new(),
            compacts: CompactSection::new(),
        }
    }

    /// Move the other Report into Self
    pub fn append(&mut self, other: Self) {
        let Self {
            mut errors,
            mut warnings,
            mut summary,
            mut checks,
            mut compacts,
        } = other;
        self.errors.append(&mut errors);
        self.warnings.append(&mut warnings);
        self.summary.append(&mut summary);
        self.checks.append(&mut checks);
        self.compacts.append(&mut compacts);
    }

    /// Add a warning message to the report
    pub fn add_warning(
        &mut self,
        repository: &str,
        archive_glob: Option<&str>,
        msg: impl Into<String>,
    ) {
        self.warnings.push(Record::new(
            repository,
            archive_glob,
            add_msg_prefix(repository, archive_glob, msg),
        ));
    }

    /// Add a error message to the report
    pub fn add_error(
        &mut self,
        repository: &str,
        archive_glob: Option<&str>,
        msg: impl Into<String>,
    ) {
        self.errors.push(Record::new(
            repository,
            archive_glob,
            add_msg_prefix(repository, archive_glob, msg),
        ));
    }

    /// Returns True if the list of errors is not empty
    pub const fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns the number of errors
    pub const fn count_errors(&self) -> usize {
        self.errors.len()
    }

    /// Returns True if the list of warnings is not empty
    pub const fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns the number of warnings
    pub const fn count_warnings(&self) -> usize {
        self.warnings.len()
    }

    /// Return `true` if the report contains a warning or error for `repo`
    pub fn has_warning_or_error_for(&self, repo: &str) -> bool {
        self.warnings.iter().any(|w| w.repository.eq(repo))
            || self.errors.iter().any(|e| e.repository.eq(repo))
    }
}

/// Helper to format a `msg` text with a `repo[archive_glob]:` prefix
fn add_msg_prefix(repository: &str, archive_glob: Option<&str>, msg: impl Into<String>) -> String {
    format!(
        "{repository}{}{}{}",
        with_brackets_or(archive_glob, ""),
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
pub struct Record<T> {
    pub repository: String,
    pub archive_glob: Option<String>,
    /// A `None` value indicates a query which was skipped or resulted in an error.
    inner: Option<T>,
}

impl<T> Record<T> {
    fn new<S, I>(repository: S, archive_glob: Option<S>, inner: I) -> Self
    where
        S: Into<String>,
        I: Into<Option<T>>,
    {
        Self {
            repository: repository.into(),
            archive_glob: archive_glob.map(Into::into),
            inner: inner.into(),
        }
    }

    pub const fn value(&self) -> Option<&T> {
        self.inner.as_ref()
    }
}

/// Represent a part of the Report as an unordered list
pub trait Listed {
    fn list_iter(&self) -> impl Iterator<Item = String>;
}

impl Listed for Vec<Record<BulletPoint>> {
    fn list_iter(&self) -> impl Iterator<Item = String> {
        // Entries with no inner value are not printed.
        self.iter().filter_map(|r| r.value().cloned())
    }
}

/// Cell alignment for columns in a table
pub enum TabularCellAlignment {
    Left,
    Right,
}

/// Represent a part of the Report as a table
pub trait Tabular {
    /// Text to display before a table explaining the content or to add notes
    fn table_preface(&self) -> Vec<&'static str>;
    fn table_header() -> Vec<&'static str>;
    fn table_alignment() -> Vec<TabularCellAlignment>;
    fn table_row_iter(&self) -> impl Iterator<Item = Vec<String>>;
}
