// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

mod check;
mod compact;
mod info;

pub use check::borg_check;
pub use compact::borg_compact;
pub use info::borg_info;
pub use info::sanity_check;
use std::ops::Deref;

// Declare the Report components formattable
pub use crate::format::Formattable;
impl<T> Formattable for Section<T> where T: PartialEq + Clone {}
impl Formattable for Report {}

// Variants of the section types
pub type BulletPointSection = Section<BulletPoint>;
pub type CheckSection = Section<check::CheckRecord>;
pub type CompactSection = Section<compact::CompactRecord>;
pub type InfoSection = Section<info::InfoRecord>;

/// A report contains sections with structured data
pub struct Report {
    /// The error section holds borg error messages and additional errors
    pub errors: BulletPointSection,
    /// The warning section shows borg messages and additional sanity checks
    pub warnings: BulletPointSection,
    /// The summary section shows statistics for the recent backup archives
    pub summary: InfoSection,
    /// The check section shows results from `borg check`
    pub checks: CheckSection,
    /// The compact section shows results from `borg compact`
    pub compacts: CompactSection,
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
        self.errors.append(errors);
        self.warnings.append(warnings);
        self.summary.append(summary);
        self.checks.append(checks);
        self.compacts.append(compacts);
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
        self.errors.content().len()
    }

    /// Returns True if the list of warnings is not empty
    pub const fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns the number of warnings
    pub const fn count_warnings(&self) -> usize {
        self.warnings.content().len()
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
        archive_glob.map_or_else(String::new, |glob| String::from("[") + glob + "]"),
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
    fn new<S, I>(repository: S, archive_glob: Option<S>, inner: I) -> Self
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
}

impl<T> Deref for Record<T>
where
    T: PartialEq + Clone,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A section holds a list of content `T` attributed to repository / archive
pub type SectionContent<T> = Vec<Record<T>>;

/// A section holds a list of content T
pub struct Section<T>(SectionContent<T>)
where
    T: PartialEq + Clone;
impl<T> Section<T>
where
    T: PartialEq + Clone,
{
    const fn new() -> Self {
        Self(Vec::new())
    }

    pub const fn content(&self) -> &SectionContent<T> {
        &self.0
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

    fn append(&mut self, mut other: Self) {
        self.0.append(&mut other.0);
    }
}

impl<T> Deref for Section<T>
where
    T: PartialEq + Clone,
{
    type Target = SectionContent<T>;

    fn deref(&self) -> &Self::Target {
        self.content()
    }
}

impl<T> From<Section<T>> for SectionContent<T>
where
    T: PartialEq + Clone,
{
    fn from(value: Section<T>) -> Self {
        value.0
    }
}

/// An element of an unordered list
#[derive(Debug, Clone, PartialEq, Eq)]
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

// A `Section` with a list of `BulletPoints`
impl BulletPointSection {
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
